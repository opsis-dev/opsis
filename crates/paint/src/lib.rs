mod boxes;
mod canvas;
mod clips;
pub mod color;
mod effects;
mod filters;
mod geometry;
mod gradients;
pub mod lottie;
mod resources;
mod svg;
mod text;

use std::collections::HashMap;

use opsis_layout::Viewport;
use opsis_layout::computed::{Context, ROOT_FONT_SIZE};
use opsis_layout::output::{Laid, Placed, PlacedKind};
use opsis_scene::node::{Element, Kind, Node};
use opsis_scene::prop;
use opsis_scene::paint::{BlendMode, Filter};
use opsis_scene::style::{Overflow, Position, Style, Visibility};
use opsis_scene::values::Length;
use opsis_skia::{Matrix44, Paint, RectF, Rgba, Sampling, Surface};

use crate::boxes::{Areas, draw_box, draw_replaced};
use crate::geometry::{is_rounded, matrix44, rect_f, round_rect};
pub use crate::resources::Resources;

pub(crate) struct Painter<'s, 'r, 'l> {
    pub root: &'l Node,
    pub surface: &'s mut Surface,
    pub resources: &'s mut Resources<'r>,
    pub laid: &'l Laid,
    pub index: &'l HashMap<Vec<usize>, usize>,
    pub context: Context,
    pub offscreen: bool,
}

#[derive(Debug, Clone)]
enum Entry<'n> {
    Box {
        element: &'n Element,
        path: Vec<usize>,
        clips: Vec<usize>,
    },
    Text {
        path: Vec<usize>,
        clips: Vec<usize>,
    },
    Pseudo {
        element: &'n Element,
        path: Vec<usize>,
        clips: Vec<usize>,
        flow: Vec<Entry<'n>>,
    },
}

#[derive(Debug, Default)]
struct Layers<'n> {
    negative: Vec<(i32, Entry<'n>)>,
    zero: Vec<Entry<'n>>,
    positive: Vec<(i32, Entry<'n>)>,
}

fn is_positioned(style: &Style) -> bool {
    prop!(style, Position).is_some_and(|p| *p != Position::Static)
}

fn has_transform(style: &Style) -> bool {
    prop!(style, Transform).is_some_and(|t| !t.is_empty())
        || prop!(style, Translate).is_some()
        || prop!(style, Rotate).is_some()
        || prop!(style, Scale).is_some()
}

fn makes_context(style: &Style) -> bool {
    (is_positioned(style) && prop!(style, ZIndex).is_some())
        || effects::needs_layer(style)
        || effects::has_mask(style)
        || has_transform(style)
        || prop!(style, BackdropFilter).is_some_and(|f| !f.is_empty())
        || prop!(style, ClipPath).is_some()
        || prop!(style, Mask).is_some_and(|m| !m.is_empty())
        || prop!(style, Perspective).is_some()
}

fn clips_overflow(style: &Style) -> bool {
    let hidden = |o: Option<&Overflow>| o.is_some_and(|o| *o != Overflow::Visible);
    hidden(prop!(style, OverflowX)) || hidden(prop!(style, OverflowY))
}

impl<'n> Painter<'_, '_, '_> {
    fn collect(
        &self,
        element: &'n Element,
        path: &[usize],
        clips: &[usize],
        into: &mut Layers<'n>,
        flow: &mut Vec<Entry<'n>>,
    ) {
        let mut inner = clips.to_vec();
        if clips_overflow(&element.style)
            && let Some(&i) = self.index.get(path)
        {
            inner.push(i);
        }
        for (i, child) in element.children.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(i);
            match child {
                Node::Text(_) => {
                    if self.index.contains_key(&child_path) {
                        flow.push(Entry::Text {
                            path: child_path,
                            clips: inner.clone(),
                        });
                    }
                }
                Node::Element(el) => {
                    if !self.index.contains_key(&child_path) {
                        self.collect(el, &child_path, &inner, into, flow);
                        continue;
                    }
                    let entry = Entry::Box {
                        element: el,
                        path: child_path.clone(),
                        clips: inner.clone(),
                    };
                    let style = &el.style;
                    let z = prop!(style, ZIndex).copied();
                    let context = makes_context(style);
                    match z {
                        Some(z) if z < 0 && context => into.negative.push((z, entry)),
                        Some(z) if z > 0 && context => into.positive.push((z, entry)),
                        _ if context => into.zero.push(entry),
                        _ if is_positioned(style) => {
                            let at = into.zero.len();
                            into.zero.push(Entry::Pseudo {
                                element: el,
                                path: child_path.clone(),
                                clips: inner.clone(),
                                flow: Vec::new(),
                            });
                            let mut own = Vec::new();
                            self.collect(el, &child_path, &inner, into, &mut own);
                            if let Some(Entry::Pseudo { flow, .. }) = into.zero.get_mut(at) {
                                *flow = own;
                            }
                        }
                        _ => {
                            flow.push(entry);
                            self.collect(el, &child_path, &inner, into, flow);
                        }
                    }
                }
            }
        }
    }

    fn apply_clips(&mut self, clips: &[usize]) {
        for &i in clips {
            let placed = &self.laid.boxes[i];
            let Some(style) = placed.style.as_ref() else {
                continue;
            };
            let areas = Areas::of(placed, style, self);
            self.surface.reset_matrix();
            self.surface.concat(matrix44(&placed.matrix));
            if is_rounded(&areas.inner) {
                self.surface
                    .clip_round_rect(round_rect(areas.padding, &areas.inner), true);
            } else {
                self.surface.clip_rect(rect_f(areas.padding), true);
            }
        }
    }

    fn with_matrix(&mut self, placed: &Placed) {
        self.surface.reset_matrix();
        self.surface.concat(matrix44(&placed.matrix));
    }

    fn draw_own(&mut self, element: &Element, path: &[usize]) {
        let Some(&i) = self.index.get(path) else {
            return;
        };
        let laid = self.laid;
        let placed = &laid.boxes[i];
        if placed.text.visibility == Visibility::Hidden {
            return;
        }
        self.with_matrix(placed);
        draw_box(self, placed, &element.style);
        match &element.kind {
            Kind::Image(asset) => draw_replaced(self, placed, &element.style, &asset.0),
            Kind::Svg(svg) => self.draw_svg(placed, &element.style, svg),
            Kind::Drawing(drawing) => canvas::draw(self, placed, &element.style, drawing),
            Kind::Lottie(frame) => boxes::draw_lottie(self, placed, &element.style, frame),
            _ => {}
        }
    }

    fn draw_svg(&mut self, placed: &Placed, style: &Style, svg: &opsis_scene::node::SvgElement) {
        let areas = Areas::of(placed, style, self);
        let content = areas.content;
        let mut rooted = svg.clone();
        if !rooted.attributes.iter().any(|(k, _)| k == "color") {
            let [r, g, b, a] = color::resolve(&placed.text.color, &placed.text.color).srgb();
            rooted.attributes.push((
                "color".to_owned(),
                format!(
                    "rgba({}, {}, {}, {})",
                    f64::from(r.clamp(0.0, 1.0)) * 255.0,
                    f64::from(g.clamp(0.0, 1.0)) * 255.0,
                    f64::from(b.clamp(0.0, 1.0)) * 255.0,
                    f64::from(a.clamp(0.0, 1.0))
                ),
            ));
        }
        let document = clips::markup(&rooted, Some((content.width, content.height)));
        let base = placed
            .matrix
            .then(&opsis_layout::transform::Matrix::translation(
                content.x, content.y, 0.0,
            ));
        let Painter {
            surface, resources, ..
        } = self;
        let visible = prop!(style, OverflowX) == Some(&Overflow::Visible)
            || svg
                .attributes
                .iter()
                .any(|(k, v)| k == "overflow" && v == "visible");
        let Some(tree) = resources.markup(document) else {
            return;
        };
        let saved = surface.save();
        if !visible {
            surface.clip_rect(geometry::rect_f(content), true);
        }
        let mut decode = |bytes: &[u8]| opsis_skia::Image::decode(bytes);
        svg::SvgPainter {
            surface,
            decode: &mut decode,
        }
        .group(tree.root(), &base);
        surface.restore_to(saved);
    }

    fn draw_text(&mut self, path: &[usize]) {
        let Some(&i) = self.index.get(path) else {
            return;
        };
        let laid = self.laid;
        let placed = &laid.boxes[i];
        if matches!(placed.kind, PlacedKind::Text { .. }) {
            self.with_matrix(placed);
            text::draw_text(self, placed);
        }
    }

    fn paint_entry(&mut self, entry: &Entry<'n>) {
        match entry {
            Entry::Text { path, clips } => {
                let saved = self.surface.save();
                self.apply_clips(clips);
                self.draw_text(path);
                self.surface.restore_to(saved);
            }
            Entry::Box {
                element,
                path,
                clips,
            } => {
                let saved = self.surface.save();
                self.apply_clips(clips);
                self.draw_own(element, path);
                self.surface.restore_to(saved);
            }
            Entry::Pseudo {
                element,
                path,
                clips,
                flow,
            } => {
                let saved = self.surface.save();
                self.apply_clips(clips);
                self.draw_own(element, path);
                self.surface.restore_to(saved);
                for inner in flow {
                    self.paint_entry(inner);
                }
            }
        }
    }

    fn faces_back(&self, element: &Element, path: &[usize]) -> bool {
        prop!(&element.style, BackfaceVisibility) == Some(&Visibility::Hidden)
            && self
                .index
                .get(path)
                .is_some_and(|&i| self.laid.boxes[i].matrix.faces_back(self.laid.boxes[i].border_box))
    }

    fn filter_reach(&self, element: &Element, path: &[usize]) -> f64 {
        let style = &element.style;
        if prop!(style, MixBlendMode).is_some_and(|b| *b != BlendMode::Normal)
            || prop!(style, BackdropFilter).is_some_and(|f| !f.is_empty())
        {
            return 0.0;
        }
        let (Some(filters), Some(&i)) = (prop!(style, Filter), self.index.get(path)) else {
            return 0.0;
        };
        let text = &self.laid.boxes[i].text;
        let length = |l: &Length| geometry::px(l, text, &self.context, 0.0);
        filters
            .iter()
            .map(|filter| match filter {
                Filter::Blur(radius) => 3.0 * length(radius),
                Filter::DropShadow(shadow) => {
                    3.0f64.mul_add(length(&shadow.blur), length(&shadow.x).abs().max(length(&shadow.y).abs()))
                }
                _ => 0.0,
            })
            .sum()
    }

    fn paint_offscreen(&mut self, element: &'n Element, path: &[usize], clips: &[usize], reach: f64) {
        let margin = reach.ceil();
        let grow: u32 = opsis_types::numbers::whole(margin * 2.0);
        let (width, height) = (self.surface.width() + grow, self.surface.height() + grow);
        let Ok(surface) = Surface::new(width, height) else {
            return;
        };
        let shift = opsis_types::numbers::narrow(margin);
        let mut surface = surface.with_base(Matrix44::translation(shift, shift));
        surface.clear(Rgba::TRANSPARENT);
        {
            let mut sub = Painter {
                root: self.root,
                surface: &mut surface,
                resources: self.resources,
                laid: self.laid,
                index: self.index,
                context: self.context,
                offscreen: true,
            };
            sub.paint_context(element, path, &[]);
        }
        let Some(image) = surface.snapshot() else {
            return;
        };
        let saved = self.surface.save();
        self.apply_clips(clips);
        self.surface.reset_matrix();
        let (w, h) = (
            opsis_types::numbers::narrow(f64::from(width)),
            opsis_types::numbers::narrow(f64::from(height)),
        );
        self.surface.draw_image_rect(
            &image,
            (RectF::new(0.0, 0.0, w, h), RectF::new(-shift, -shift, w, h)),
            Sampling::Nearest,
            &Paint::new(),
        );
        self.surface.restore_to(saved);
    }

    fn paint_context(&mut self, element: &'n Element, path: &[usize], clips: &[usize]) {
        if self.faces_back(element, path) {
            return;
        }
        if !self.offscreen {
            let reach = self.filter_reach(element, path);
            if reach > 0.0 {
                self.paint_offscreen(element, path, clips, reach);
                return;
            }
        }
        let was_offscreen = self.offscreen;
        self.offscreen = false;
        self.paint_context_here(element, path, clips);
        self.offscreen = was_offscreen;
    }

    fn paint_context_here(&mut self, element: &'n Element, path: &[usize], clips: &[usize]) {
        let saved = self.surface.save();
        self.apply_clips(clips);
        let laid = self.laid;
        let root = self.root;
        let opened = match self.index.get(path).copied() {
            Some(i) => {
                let placed = &laid.boxes[i];
                self.with_matrix(placed);
                Some((
                    placed,
                    effects::open_layer(self, placed, &element.style, root),
                ))
            }
            None => None,
        };
        self.draw_own(element, path);
        let mut layers = Layers::default();
        let mut flow = Vec::new();
        self.collect(element, path, &[], &mut layers, &mut flow);
        layers.negative.sort_by_key(|(z, _)| *z);
        layers.positive.sort_by_key(|(z, _)| *z);
        for (_, entry) in &layers.negative {
            self.paint_nested(entry);
        }
        for entry in &flow {
            self.paint_entry(entry);
        }
        for entry in &layers.zero {
            self.paint_nested(entry);
        }
        for (_, entry) in &layers.positive {
            self.paint_nested(entry);
        }
        if let Some((placed, opened)) = opened {
            effects::close_layer(self, placed, &element.style, opened);
        }
        self.surface.restore_to(saved);
    }

    fn paint_nested(&mut self, entry: &Entry<'n>) {
        match entry {
            Entry::Box {
                element,
                path,
                clips,
            } => self.paint_context(element, path, clips),
            Entry::Text { .. } | Entry::Pseudo { .. } => self.paint_entry(entry),
        }
    }
}

pub fn paint(
    root: &Node,
    laid: &Laid,
    viewport: Viewport,
    surface: &mut Surface,
    resources: &mut Resources<'_>,
) {
    let Node::Element(element) = root else {
        return;
    };
    let index = laid
        .boxes
        .iter()
        .enumerate()
        .map(|(i, placed)| (placed.path.clone(), i))
        .collect();
    let mut painter = Painter {
        root,
        surface,
        resources,
        laid,
        index: &index,
        offscreen: false,
        context: Context {
            viewport_width: viewport.width,
            viewport_height: viewport.height,
            root_font_size: ROOT_FONT_SIZE,
            container: None,
        },
    };
    painter.paint_context(element, &[], &[]);
}
