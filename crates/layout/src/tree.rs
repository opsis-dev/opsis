use std::borrow::Cow;
use std::collections::HashMap;

use opsis_scene::node::{Kind, Node};
use opsis_scene::prop;
use opsis_scene::style::{ContainerType, Display as SceneDisplay, Style as SceneStyle};
use opsis_scene::values::Length as SceneLength;
use opsis_types::numbers::narrow;
use taffy::prelude::*;
use taffy::util::MaybeResolve;
use taffy::{
    Cache, CacheTree, LayoutBlockContainer, LayoutContainingBlock, LayoutFlexboxContainer,
    LayoutGridContainer, LayoutInput, LayoutOutput, LayoutPartialTree, RunMode,
    TraversePartialTree, compute_block_layout, compute_cached_layout, compute_flexbox_layout,
    compute_grid_layout, compute_hidden_layout, compute_leaf_layout, compute_oof_layout,
};

use crate::computed::{Container, Context, Formula, TextStyle};
use crate::fonts::Fonts;
use crate::inline::{InlineContent, InlineItem};
use crate::style::{SVG_DEFAULT_HEIGHT, SVG_DEFAULT_WIDTH, taffy_style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Div,
    Span,
    Image,
    Svg,
    Canvas,
    Leaf,
}

#[derive(Debug)]
pub enum Content {
    Element,
    Replaced { width: f64, height: f64 },
    Inline(Box<InlineContent>),
}

#[derive(Debug)]
pub struct LayoutNode {
    pub path: Vec<usize>,
    pub tag: Tag,
    pub scene: Option<SceneStyle>,
    pub style: Style,
    pub text: TextStyle,
    pub children: Vec<usize>,
    pub content: Content,
    pub cache: Cache,
    pub layout: Layout,
    pub hoisted: Vec<NodeId>,
    pub drawn: Vec<(usize, Vec<crate::output::TextRun>)>,
}

fn replaced_size(known: Size<Option<f32>>, width: f32, height: f32) -> Size<f32> {
    match (known.width, known.height) {
        (Some(w), Some(h)) => Size {
            width: w,
            height: h,
        },
        (Some(w), None) if width > 0.0 => Size {
            width: w,
            height: w * height / width,
        },
        (None, Some(h)) if height > 0.0 => Size {
            width: h * width / height,
            height: h,
        },
        (w, h) => Size {
            width: w.unwrap_or(width),
            height: h.unwrap_or(height),
        },
    }
}

pub struct Images<'a> {
    pub size_of: &'a dyn Fn(&str) -> Option<(f64, f64)>,
}

impl std::fmt::Debug for Images<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Images").finish_non_exhaustive()
    }
}

pub struct Tree<'a> {
    pub nodes: Vec<LayoutNode>,
    pub formulas: Vec<Formula>,
    pub fonts: &'a mut Fonts,
    pub context: Context,
    pub text_layouts: parley::LayoutContext<u32>,
    pub extents: HashMap<String, crate::inline::FontExtent>,
    pub containers: HashMap<Vec<usize>, Container>,
}

impl std::fmt::Debug for Tree<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tree")
            .field("nodes", &self.nodes.len())
            .finish_non_exhaustive()
    }
}

const fn kind_tag(kind: &Kind) -> Tag {
    match kind {
        Kind::Div => Tag::Div,
        Kind::Span => Tag::Span,
        Kind::Image(_) => Tag::Image,
        Kind::Svg(_) => Tag::Svg,
        Kind::Drawing(_) => Tag::Canvas,
        Kind::Footage(_) | Kind::Lottie(_) | Kind::Effect(_) | Kind::Slot(_) => Tag::Leaf,
    }
}

fn scene_display(style: &SceneStyle, tag: Tag) -> SceneDisplay {
    prop!(style, Display).copied().unwrap_or(match tag {
        Tag::Span => SceneDisplay::Inline,
        Tag::Div | Tag::Image | Tag::Svg | Tag::Canvas | Tag::Leaf => SceneDisplay::Block,
    })
}

fn is_out_of_flow(style: &SceneStyle) -> bool {
    matches!(
        prop!(style, Position),
        Some(opsis_scene::style::Position::Absolute | opsis_scene::style::Position::Fixed)
    )
}

pub fn is_inline_level(node: &Node) -> bool {
    match node {
        Node::Text(_) => true,
        Node::Element(element) => {
            let tag = kind_tag(&element.kind);
            !is_out_of_flow(&element.style)
                && matches!(
                    scene_display(&element.style, tag),
                    SceneDisplay::Inline
                        | SceneDisplay::InlineBlock
                        | SceneDisplay::InlineFlex
                        | SceneDisplay::InlineGrid
                )
        }
    }
}

fn attribute_px(svg: &opsis_scene::node::SvgElement, name: &str) -> Option<f64> {
    svg.attributes
        .iter()
        .find(|(key, _)| key == name)
        .and_then(|(_, value)| value.trim_end_matches("px").trim().parse().ok())
}

fn attribute_length(svg: &opsis_scene::node::SvgElement, name: &str) -> Option<SceneLength> {
    let (_, value) = svg.attributes.iter().find(|(key, _)| key == name)?;
    let value = value.trim();
    opsis_scene::css::length(value).or_else(|| value.parse::<f64>().ok().map(SceneLength::px))
}

fn sized_style(element: &opsis_scene::node::Element) -> Cow<'_, SceneStyle> {
    let Kind::Svg(root) = &element.kind else {
        return Cow::Borrowed(&element.style);
    };
    let mut base = SceneStyle::default();
    if let Some(width) = attribute_length(root, "width") {
        base = base.width(width);
    }
    if let Some(height) = attribute_length(root, "height") {
        base = base.height(height);
    }
    Cow::Owned(base.merged(&element.style))
}

impl Tree<'_> {
    pub fn push(&mut self, node: LayoutNode) -> usize {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    pub fn build_element(
        &mut self,
        element: &opsis_scene::node::Element,
        path: &[usize],
        parent_text: &TextStyle,
        images: &Images<'_>,
        blockify: bool,
    ) -> Option<usize> {
        let tag = kind_tag(&element.kind);
        let display = scene_display(&element.style, tag);
        if display == SceneDisplay::None {
            return None;
        }
        let text = self.text_child(parent_text, &element.style);
        let sized = sized_style(element);
        let style = taffy_style(
            &sized,
            display,
            blockify,
            &text,
            &self.context,
            &mut self.formulas,
        );
        let content = match &element.kind {
            Kind::Image(asset) => {
                let (width, height) = (images.size_of)(&asset.0).unwrap_or((0.0, 0.0));
                Content::Replaced { width, height }
            }
            Kind::Svg(root) => Content::Replaced {
                width: attribute_px(root, "width").unwrap_or(SVG_DEFAULT_WIDTH),
                height: attribute_px(root, "height").unwrap_or(SVG_DEFAULT_HEIGHT),
            },
            Kind::Drawing(canvas) => Content::Replaced {
                width: f64::from(canvas.width),
                height: f64::from(canvas.height),
            },
            _ => Content::Element,
        };
        let drawn = match &element.kind {
            Kind::Drawing(canvas) => crate::drawing::shaped(self, canvas),
            _ => Vec::new(),
        };
        let id = self.push(LayoutNode {
            path: path.to_vec(),
            tag,
            scene: Some(element.style.clone()),
            style,
            text: text.clone(),
            children: Vec::new(),
            content,
            cache: Cache::new(),
            layout: Layout::with_order(0),
            hoisted: Vec::new(),
            drawn,
        });
        if matches!(self.nodes[id].content, Content::Element) {
            let outer = self.context.container;
            self.enter_container(&element.style, path);
            let children = self.build_children(&element.children, path, &text, images, display);
            self.context.container = outer;
            self.nodes[id].children = children;
        }
        Some(id)
    }

    fn build_children(
        &mut self,
        children: &[Node],
        path: &[usize],
        text: &TextStyle,
        images: &Images<'_>,
        display: SceneDisplay,
    ) -> Vec<usize> {
        let flat = flatten_contents(children, path);
        let in_flex_or_grid = matches!(
            display,
            SceneDisplay::Flex
                | SceneDisplay::InlineFlex
                | SceneDisplay::Grid
                | SceneDisplay::InlineGrid
        );
        let mut out = Vec::new();
        let mut run: Vec<(Vec<usize>, &Node)> = Vec::new();
        for (child_path, child) in flat {
            let inline = !in_flex_or_grid && is_inline_level(child);
            let flex_text = in_flex_or_grid && matches!(child, Node::Text(_));
            if inline || flex_text {
                run.push((child_path, child));
                continue;
            }
            self.flush_run(&mut run, path, text, images, &mut out);
            if let Node::Element(element) = child {
                out.extend(self.build_element(element, &child_path, text, images, in_flex_or_grid));
            }
        }
        self.flush_run(&mut run, path, text, images, &mut out);
        out
    }

    fn flush_run(
        &mut self,
        run: &mut Vec<(Vec<usize>, &Node)>,
        path: &[usize],
        text: &TextStyle,
        images: &Images<'_>,
        out: &mut Vec<usize>,
    ) {
        if run.is_empty() {
            return;
        }
        let items: Vec<(Vec<usize>, &Node)> = std::mem::take(run);
        let all_collapsible = items.iter().all(|(_, n)| match n {
            Node::Text(t) => t.chars().all(crate::inline::is_css_space),
            Node::Element(_) => false,
        });
        if all_collapsible && !crate::inline::preserves_spaces(text) {
            return;
        }
        let content = InlineContent::collect(self, &items, text, images);
        if content.items.is_empty() {
            return;
        }
        let id = self.push(LayoutNode {
            path: path.to_vec(),
            tag: Tag::Div,
            scene: None,
            style: Style {
                display: Display::Block,
                ..Style::default()
            },
            text: text.clone(),
            children: Vec::new(),
            content: Content::Inline(Box::new(content)),
            cache: Cache::new(),
            layout: Layout::with_order(0),
            hoisted: Vec::new(),
            drawn: Vec::new(),
        });
        out.push(id);
    }

    fn node(&self, id: NodeId) -> &LayoutNode {
        &self.nodes[usize::from(id)]
    }

    fn node_mut(&mut self, id: NodeId) -> &mut LayoutNode {
        &mut self.nodes[usize::from(id)]
    }

    fn enter_container(&mut self, style: &SceneStyle, path: &[usize]) {
        let kind = prop!(style, ContainerType).copied().unwrap_or_default();
        if kind == ContainerType::Normal {
            return;
        }
        let fallback = self.context.container.unwrap_or(Container {
            width: self.context.viewport_width,
            height: self.context.viewport_height,
        });
        let size = self.containers.get(path).copied().unwrap_or(fallback);
        self.context.container = Some(match kind {
            ContainerType::InlineSize => Container {
                width: size.width,
                height: fallback.height,
            },
            ContainerType::Size | ContainerType::Normal => size,
        });
    }

    pub fn measured_containers(&self) -> HashMap<Vec<usize>, Container> {
        self.nodes
            .iter()
            .filter(|node| {
                node.scene.as_ref().is_some_and(|style| {
                    prop!(style, ContainerType).is_some_and(|kind| *kind != ContainerType::Normal)
                })
            })
            .map(|node| {
                let layout = &node.layout;
                let across = layout.padding.left
                    + layout.padding.right
                    + layout.border.left
                    + layout.border.right;
                let down = layout.padding.top
                    + layout.padding.bottom
                    + layout.border.top
                    + layout.border.bottom;
                (
                    node.path.clone(),
                    Container {
                        width: f64::from(layout.size.width - across),
                        height: f64::from(layout.size.height - down),
                    },
                )
            })
            .collect()
    }

    pub fn text_child(
        &mut self,
        parent: &TextStyle,
        style: &opsis_scene::style::Style,
    ) -> TextStyle {
        let mut text = parent.child(style, &self.context);
        let font = crate::inline::font_extent(self, &text);
        text.ch = f64::from(font.zero);
        text.ex = f64::from(font.x_height);
        text
    }

    pub fn inline_box_ids(&self, id: usize) -> Vec<usize> {
        match &self.nodes[id].content {
            Content::Inline(content) => content
                .items
                .iter()
                .filter_map(|item| match item {
                    InlineItem::Box { node, .. } => Some(*node),
                    InlineItem::Text { .. }
                    | InlineItem::Tab { .. }
                    | InlineItem::SpanStart { .. }
                    | InlineItem::Edge { .. }
                    | InlineItem::SpanEnd => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }
}

pub fn flatten_contents<'n>(children: &'n [Node], path: &[usize]) -> Vec<(Vec<usize>, &'n Node)> {
    let mut out = Vec::new();
    for (i, child) in children.iter().enumerate() {
        let mut child_path = path.to_vec();
        child_path.push(i);
        if let Node::Element(element) = child
            && scene_display(&element.style, kind_tag(&element.kind)) == SceneDisplay::Contents
        {
            out.extend(flatten_contents(&element.children, &child_path));
            continue;
        }
        out.push((child_path, child));
    }
    out
}

pub struct ChildIter<'a>(std::slice::Iter<'a, usize>);

impl Iterator for ChildIter<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied().map(NodeId::from)
    }
}

impl TraversePartialTree for Tree<'_> {
    type ChildIter<'a>
        = ChildIter<'a>
    where
        Self: 'a;

    fn child_ids(&self, parent: NodeId) -> Self::ChildIter<'_> {
        ChildIter(self.node(parent).children.iter())
    }

    fn child_count(&self, parent: NodeId) -> usize {
        self.node(parent).children.len()
    }

    fn get_child_id(&self, parent: NodeId, index: usize) -> NodeId {
        NodeId::from(self.node(parent).children[index])
    }
}

impl taffy::TraverseTree for Tree<'_> {}

pub const fn calc_token(index: usize) -> *const () {
    std::ptr::without_provenance((index + 1) << 3)
}

fn calc_index(token: *const ()) -> usize {
    (token.addr() >> 3).saturating_sub(1)
}

impl LayoutPartialTree for Tree<'_> {
    type CustomIdent = String;

    type CoreContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    fn get_core_container_style(&self, node: NodeId) -> Self::CoreContainerStyle<'_> {
        &self.node(node).style
    }

    fn resolve_calc_value(&self, token: *const (), basis: f32) -> f32 {
        self.formulas
            .get(calc_index(token))
            .map_or(0.0, |formula| narrow(formula.at(f64::from(basis))))
    }

    fn set_unrounded_layout(&mut self, node: NodeId, layout: &Layout) {
        self.node_mut(node).layout = *layout;
    }

    fn compute_child_layout(&mut self, node: NodeId, inputs: LayoutInput) -> LayoutOutput {
        compute_cached_layout(self, node, inputs, |tree, node, inputs| {
            let id = usize::from(node);
            if tree.nodes[id].style.display == Display::None {
                return compute_hidden_layout(tree, node);
            }
            let mut output = match &tree.nodes[id].content {
                Content::Inline(_) => crate::inline::compute_inline_layout(tree, id, inputs),
                Content::Replaced { width, height } => {
                    let (width, height) = (narrow(*width), narrow(*height));
                    let style = &tree.nodes[id].style;
                    let formulas = &tree.formulas;
                    let calc = |token: *const (), basis: f32| {
                        formulas
                            .get(calc_index(token))
                            .map_or(0.0, |f| narrow(f.at(f64::from(basis))))
                    };
                    let styled = inputs
                        .known_dimensions
                        .or(style.size.maybe_resolve(inputs.parent_size, calc));
                    compute_leaf_layout(inputs, style, calc, |known, _| {
                        replaced_size(known.or(styled), width, height)
                    })
                }
                Content::Element => match tree.nodes[id].style.display {
                    Display::Flex => compute_flexbox_layout(tree, node, inputs),
                    Display::Grid => compute_grid_layout(tree, node, inputs),
                    _ => compute_block_layout(tree, node, inputs, None),
                },
            };
            if inputs.run_mode == RunMode::PerformLayout {
                compute_oof_layout(tree, node, &mut output);
            }
            output
        })
    }
}

impl LayoutContainingBlock for Tree<'_> {
    type OofItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    fn get_oof_item_style(&self, node: NodeId) -> Self::OofItemStyle<'_> {
        &self.node(node).style
    }

    fn clear_hoisted_children(&mut self, node: NodeId) {
        self.node_mut(node).hoisted.clear();
    }

    fn add_hoisted_children(&mut self, node: NodeId, hoisted: &[NodeId]) {
        self.node_mut(node).hoisted.extend_from_slice(hoisted);
    }
}

impl CacheTree for Tree<'_> {
    fn cache_get(&mut self, node: NodeId, inputs: &LayoutInput) -> Option<LayoutOutput> {
        self.node_mut(node).cache.get(inputs)
    }

    fn cache_store(&mut self, node: NodeId, inputs: &LayoutInput, output: LayoutOutput) {
        self.node_mut(node).cache.store(inputs, output);
    }

    fn cache_clear(&mut self, node: NodeId) {
        self.node_mut(node).cache.clear();
    }
}

impl LayoutFlexboxContainer for Tree<'_> {
    type FlexboxContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    type FlexboxItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    fn get_flexbox_container_style(&self, node: NodeId) -> Self::FlexboxContainerStyle<'_> {
        &self.node(node).style
    }

    fn get_flexbox_child_style(&self, child: NodeId) -> Self::FlexboxItemStyle<'_> {
        &self.node(child).style
    }
}

impl LayoutGridContainer for Tree<'_> {
    type GridContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    type GridItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    fn get_grid_container_style(&self, node: NodeId) -> Self::GridContainerStyle<'_> {
        &self.node(node).style
    }

    fn get_grid_child_style(&self, child: NodeId) -> Self::GridItemStyle<'_> {
        &self.node(child).style
    }
}

impl LayoutBlockContainer for Tree<'_> {
    type BlockContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    type BlockItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    fn get_block_container_style(&self, node: NodeId) -> Self::BlockContainerStyle<'_> {
        &self.node(node).style
    }

    fn get_block_child_style(&self, child: NodeId) -> Self::BlockItemStyle<'_> {
        &self.node(child).style
    }
}
