use std::borrow::Cow;

use opsis_scene::node::Node;
use opsis_scene::style::{
    FontStyle as SceneFontStyle, OverflowWrap as SceneOverflowWrap, TextAlign, TextRendering,
    TextTransform, VerticalAlign as SceneVerticalAlign, WhiteSpace, WordBreak as SceneWordBreak,
};
use opsis_types::numbers::{exact_eq, narrow};
use parley::{
    Alignment, AlignmentOptions, CHROMIUM_LINE_BREAK_OVERRIDE, FontFamily, FontFamilyName,
    FontStyle, FontWeight, InlineBox, InlineBoxKind, Layout as Paragraph, LineHeight, OverflowWrap,
    PositionedLayoutItem, StyleProperty, TextWrapMode, VerticalAlign, WhiteSpaceCollapse,
    WordBreak, YieldData,
};
use taffy::prelude::*;
use taffy::util::ResolveOrZero;
use taffy::{
    Baselines, LayoutInput, LayoutOutput, LayoutPartialTree, RequestedAxis, RunMode, SizingMode,
};

use crate::computed::{LineSpacing, TextStyle};
use crate::tree::{Images, Tree, is_inline_level};

const FALLBACK_FAMILY: &str = "df-main";

#[derive(Debug, Clone)]
pub enum InlineItem {
    Text {
        path: Vec<usize>,
        text: String,
        style: TextStyle,
        shift: f32,
    },
    SpanStart {
        path: Vec<usize>,
        style: TextStyle,
        edges: SpanEdges,
        offset: (f64, f64),
    },
    Edge {
        width: f32,
        skip_start: f32,
        skip_end: f32,
        shift: f32,
    },
    SpanEnd,
    Box {
        node: usize,
        placement: Placement,
    },
    Tab {
        path: Vec<usize>,
        style: TextStyle,
        shift: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Placement {
    Shift(f32),
    Middle { base: f32, half_x: f32 },
    TextTop { base: f32, ascent: f32 },
    TextBottom { base: f32, descent: f32 },
    LineTop,
    LineBottom,
}

impl Placement {
    pub fn shift(self, above: f32, below: f32) -> Option<f32> {
        match self {
            Self::Shift(shift) => Some(shift),
            Self::Middle { base, half_x } => Some(base + (above - below) / 2.0 - half_x),
            Self::TextTop { base, ascent } => Some(base + above - ascent),
            Self::TextBottom { base, descent } => Some(base + descent - below),
            Self::LineTop | Self::LineBottom => None,
        }
    }
}

const SUB_DIVISOR: f64 = 5.0;
const SUPER_DIVISOR: f64 = 3.0;

fn placement(
    tree: &mut Tree<'_>,
    align: Option<&SceneVerticalAlign>,
    base: f32,
    parent: &TextStyle,
    own: &TextStyle,
) -> Placement {
    let Some(align) = align else {
        return Placement::Shift(base);
    };
    let font_size = layout_unit_floor(parent.font_size);
    match align {
        SceneVerticalAlign::Baseline => Placement::Shift(base),
        SceneVerticalAlign::Sub => {
            Placement::Shift(base + layout_unit_floor(f64::from(font_size) / SUB_DIVISOR) + 1.0)
        }
        SceneVerticalAlign::Super => {
            Placement::Shift(base - layout_unit_floor(f64::from(font_size) / SUPER_DIVISOR) - 1.0)
        }
        SceneVerticalAlign::Length(length) => {
            let line_height = own.line_height_px();
            let raise = crate::computed::resolve(length, &own.units(), &tree.context)
                .px_or(line_height, 0.0);
            Placement::Shift(base - narrow(raise))
        }
        SceneVerticalAlign::Middle => Placement::Middle {
            base,
            half_x: (font_extent(tree, parent).x_height / 2.0).round(),
        },
        SceneVerticalAlign::TextTop => Placement::TextTop {
            base,
            ascent: font_extent(tree, parent).rounded_ascent(),
        },
        SceneVerticalAlign::TextBottom => Placement::TextBottom {
            base,
            descent: font_extent(tree, parent).rounded_descent(),
        },
        SceneVerticalAlign::Top => Placement::LineTop,
        SceneVerticalAlign::Bottom => Placement::LineBottom,
    }
}

fn span_shift(tree: &mut Tree<'_>, placement: Placement, own: &TextStyle) -> f32 {
    let font = font_extent(tree, own);
    let extent = text_extent(own, font);
    placement
        .shift(extent.above, extent.below)
        .unwrap_or(match placement {
            Placement::Shift(base)
            | Placement::Middle { base, .. }
            | Placement::TextTop { base, .. }
            | Placement::TextBottom { base, .. } => base,
            Placement::LineTop | Placement::LineBottom => 0.0,
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SpanEdges {
    pub start_margin: f32,
    pub start: f32,
    pub end: f32,
    pub end_margin: f32,
    pub top: f32,
    pub bottom: f32,
}

fn span_geometry(
    style: &opsis_scene::style::Style,
    text: &TextStyle,
    context: &crate::computed::Context,
) -> (SpanEdges, (f64, f64)) {
    let units = text.units();
    let px = |length: &opsis_scene::values::Length| {
        crate::computed::resolve(length, &units, context).px_or(0.0, 0.0)
    };
    let side = |side: &opsis_scene::style::BorderSide| {
        if side.style == opsis_scene::style::BorderStyle::None {
            0.0
        } else {
            crate::style::device_border(px(&side.width))
        }
    };
    let padding = opsis_scene::prop!(style, Padding);
    let margin = opsis_scene::prop!(style, Margin);
    let border = opsis_scene::prop!(style, Border);
    let pad = |pick: fn(
        &opsis_scene::values::Edges<opsis_scene::values::Length>,
    ) -> &opsis_scene::values::Length| { padding.map_or(0.0, |p| px(pick(p))) };
    let gap = |pick: fn(
        &opsis_scene::values::Edges<opsis_scene::values::Length>,
    ) -> &opsis_scene::values::Length| { margin.map_or(0.0, |m| px(pick(m))) };
    let rule = |pick: fn(
        &opsis_scene::values::Edges<opsis_scene::style::BorderSide>,
    ) -> &opsis_scene::style::BorderSide| { border.map_or(0.0, |b| side(pick(b))) };
    let start_margin = gap(|e| &e.left);
    let end_margin = gap(|e| &e.right);
    let edges = SpanEdges {
        start_margin: narrow(start_margin),
        start: narrow(start_margin + rule(|e| &e.left) + pad(|e| &e.left)),
        end: narrow(end_margin + rule(|e| &e.right) + pad(|e| &e.right)),
        end_margin: narrow(end_margin),
        top: narrow(rule(|e| &e.top) + pad(|e| &e.top)),
        bottom: narrow(rule(|e| &e.bottom) + pad(|e| &e.bottom)),
    };
    let relative =
        opsis_scene::prop!(style, Position) == Some(&opsis_scene::style::Position::Relative);
    let offset = if relative {
        let x = opsis_scene::prop!(style, Left)
            .map(px)
            .or_else(|| opsis_scene::prop!(style, Right).map(|r| -px(r)))
            .unwrap_or(0.0);
        let y = opsis_scene::prop!(style, Top)
            .map(px)
            .or_else(|| opsis_scene::prop!(style, Bottom).map(|b| -px(b)))
            .unwrap_or(0.0);
        (x, y)
    } else {
        (0.0, 0.0)
    };
    (edges, offset)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxSize {
    pub width: f32,
    pub height: f32,
    pub baseline: Option<f32>,
    pub margin: taffy::Rect<f32>,
}

impl BoxSize {
    pub const EMPTY: Self = Self {
        width: 0.0,
        height: 0.0,
        baseline: None,
        margin: taffy::Rect {
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        },
    };

    pub fn outer_width(&self) -> f32 {
        self.margin.left + self.width + self.margin.right
    }

    pub fn outer_height(&self) -> f32 {
        self.margin.top + self.height + self.margin.bottom
    }

    pub fn outer_baseline(&self) -> f32 {
        self.baseline
            .map_or_else(|| self.outer_height(), |b| b + self.margin.top)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineBox {
    pub top: f32,
    pub baseline: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontExtent {
    pub ascent: f32,
    pub descent: f32,
    pub gap: f32,
    pub space: f32,
    pub hyphen: f32,
    pub x_height: f32,
    pub zero: f32,
    pub subpixel: bool,
}

impl FontExtent {
    const fn lends_ascent(self) -> bool {
        self.subpixel && self.descent.round() < self.descent && self.ascent.round() >= 1.0
    }

    pub const fn rounded_ascent(self) -> f32 {
        if self.lends_ascent() {
            self.ascent.round() - 1.0
        } else {
            self.ascent.round()
        }
    }

    pub const fn rounded_descent(self) -> f32 {
        if self.lends_ascent() {
            self.descent.round() + 1.0
        } else {
            self.descent.round()
        }
    }
}

#[derive(Debug)]
pub struct InlineContent {
    pub items: Vec<InlineItem>,
    pub root: TextStyle,
    pub paragraph: Option<Paragraph<u32>>,
    pub box_sizes: Vec<BoxSize>,
    pub tab_widths: Vec<f32>,
    pub hyphens: Vec<Hyphen>,
    pub lines: Vec<LineBox>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hyphen {
    pub line: usize,
    pub item: usize,
    pub width: f32,
}

pub const fn preserves_spaces(style: &TextStyle) -> bool {
    matches!(
        style.white_space,
        WhiteSpace::Pre | WhiteSpace::PreWrap | WhiteSpace::BreakSpaces
    )
}

fn transformed(text: &str, transform: TextTransform) -> String {
    match transform {
        TextTransform::None => text.to_owned(),
        TextTransform::Uppercase => text.to_uppercase(),
        TextTransform::Lowercase => text.to_lowercase(),
        TextTransform::Capitalize => {
            let mut out = String::with_capacity(text.len());
            let mut start = true;
            for c in text.chars() {
                if start && c.is_alphanumeric() {
                    out.extend(c.to_uppercase());
                } else {
                    out.push(c);
                }
                start = c.is_whitespace();
            }
            out
        }
    }
}

pub const fn is_css_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{c}')
}

fn collapsed(text: &str, keep_breaks: bool) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_space = false;
    for c in text.chars() {
        if c == '\n' && keep_breaks {
            while out.ends_with(' ') {
                out.pop();
            }
            out.push('\n');
            in_space = true;
        } else if is_css_space(c) {
            if !in_space {
                out.push(' ');
            }
            in_space = true;
        } else {
            out.push(c);
            in_space = false;
        }
    }
    out
}

fn segment_breaks(text: &str, white_space: WhiteSpace) -> String {
    match white_space {
        WhiteSpace::Normal | WhiteSpace::Nowrap => collapsed(text, false),
        WhiteSpace::PreLine => collapsed(text, true),
        WhiteSpace::Pre | WhiteSpace::PreWrap | WhiteSpace::BreakSpaces => text.to_owned(),
    }
}

impl InlineContent {
    pub fn collect(
        tree: &mut Tree<'_>,
        nodes: &[(Vec<usize>, &Node)],
        root: &TextStyle,
        images: &Images<'_>,
    ) -> Self {
        let mut items = Vec::new();
        for (path, node) in nodes {
            push_node(tree, &mut items, path, node, root, images, 0.0);
        }
        Self {
            items,
            root: root.clone(),
            paragraph: None,
            box_sizes: Vec::new(),
            tab_widths: Vec::new(),
            hyphens: Vec::new(),
            lines: Vec::new(),
        }
    }
}

fn push_node(
    tree: &mut Tree<'_>,
    items: &mut Vec<InlineItem>,
    path: &[usize],
    node: &Node,
    style: &TextStyle,
    images: &Images<'_>,
    base: f32,
) {
    match node {
        Node::Text(text) => {
            let shaped =
                segment_breaks(&transformed(text, style.text_transform), style.white_space);
            for (i, part) in shaped.split('\t').enumerate() {
                if i > 0 {
                    items.push(InlineItem::Tab {
                        path: path.to_vec(),
                        style: style.clone(),
                        shift: base,
                    });
                }
                if !part.is_empty() {
                    items.push(InlineItem::Text {
                        path: path.to_vec(),
                        text: part.to_owned(),
                        style: style.clone(),
                        shift: base,
                    });
                }
            }
        }
        Node::Element(element) => {
            let display = opsis_scene::prop!(element.style, Display).copied();
            let is_span = matches!(element.kind, opsis_scene::node::Kind::Span)
                && matches!(display, None | Some(opsis_scene::style::Display::Inline));
            if is_span && is_inline_level(node) {
                let inner = tree.text_child(style, &element.style);
                let (edges, offset) = span_geometry(&element.style, &inner, &tree.context);
                let align = opsis_scene::prop!(element.style, VerticalAlign);
                let placed = placement(tree, align, base, style, &inner);
                let shift = span_shift(tree, placed, &inner);
                items.push(InlineItem::SpanStart {
                    path: path.to_vec(),
                    style: inner.clone(),
                    edges,
                    offset,
                });
                if edges.start > 0.0 {
                    items.push(InlineItem::Edge {
                        width: edges.start,
                        skip_start: edges.start_margin,
                        skip_end: 0.0,
                        shift,
                    });
                }
                for (i, child) in element.children.iter().enumerate() {
                    let mut child_path = path.to_vec();
                    child_path.push(i);
                    if is_inline_level(child) {
                        push_node(tree, items, &child_path, child, &inner, images, shift);
                    } else if let Node::Element(block) = child
                        && let Some(id) =
                            tree.build_element(block, &child_path, &inner, images, false)
                    {
                        items.push(InlineItem::Box {
                            node: id,
                            placement: Placement::Shift(shift),
                        });
                    }
                }
                if edges.end > 0.0 {
                    items.push(InlineItem::Edge {
                        width: edges.end,
                        skip_start: 0.0,
                        skip_end: edges.end_margin,
                        shift,
                    });
                }
                items.push(InlineItem::SpanEnd);
            } else if let Some(id) = tree.build_element(element, path, style, images, false) {
                let own = tree.text_child(style, &element.style);
                let align = opsis_scene::prop!(element.style, VerticalAlign);
                let placement = placement(tree, align, base, style, &own);
                items.push(InlineItem::Box {
                    node: id,
                    placement,
                });
            }
        }
    }
}

fn tag_text(bytes: [u8; 4]) -> String {
    String::from_utf8_lossy(&bytes).into_owned()
}

fn features(style: &TextStyle) -> String {
    let numeric = style.numeric;
    let named = [
        (numeric.tabular, "tnum"),
        (numeric.proportional, "pnum"),
        (numeric.slashed_zero, "zero"),
        (numeric.oldstyle, "onum"),
        (numeric.lining, "lnum"),
    ]
    .into_iter()
    .filter(|(on, _)| *on)
    .map(|(_, name)| format!("\"{name}\" 1"));
    let set = style
        .features
        .iter()
        .map(|(t, value)| format!("\"{}\" {value}", tag_text(t.0)));
    named.chain(set).collect::<Vec<_>>().join(", ")
}

fn variations(style: &TextStyle) -> String {
    style
        .variations
        .iter()
        .map(|(t, v)| format!("\"{}\" {}", tag_text(t.0), v.0))
        .collect::<Vec<_>>()
        .join(", ")
}

fn families(style: &TextStyle, fallbacks: &[String]) -> Vec<FontFamilyName<'static>> {
    style
        .families
        .iter()
        .map(|name| FontFamilyName::Named(Cow::Owned(name.clone())))
        .chain([FontFamilyName::Named(Cow::Borrowed(FALLBACK_FAMILY))])
        .chain(
            fallbacks
                .iter()
                .map(|name| FontFamilyName::Named(Cow::Owned(name.clone()))),
        )
        .collect()
}

const fn line_height(style: &TextStyle) -> LineHeight {
    match style.line_height {
        LineSpacing::Normal => LineHeight::MetricsRelative(1.0),
        LineSpacing::Number(n) => LineHeight::FontSizeRelative(narrow(n)),
        LineSpacing::Px(px) => LineHeight::Absolute(narrow(px)),
    }
}

const fn white_space(style: &TextStyle) -> (WhiteSpaceCollapse, TextWrapMode) {
    match style.white_space {
        WhiteSpace::Normal => (WhiteSpaceCollapse::Collapse, TextWrapMode::Wrap),
        WhiteSpace::Nowrap => (WhiteSpaceCollapse::Collapse, TextWrapMode::NoWrap),
        WhiteSpace::Pre => (WhiteSpaceCollapse::Preserve, TextWrapMode::NoWrap),
        WhiteSpace::PreWrap => (WhiteSpaceCollapse::Preserve, TextWrapMode::Wrap),
        WhiteSpace::PreLine => (WhiteSpaceCollapse::PreserveBreaks, TextWrapMode::Wrap),
        WhiteSpace::BreakSpaces => (WhiteSpaceCollapse::BreakSpaces, TextWrapMode::Wrap),
    }
}

fn style_props(
    style: &TextStyle,
    brush: u32,
    fallbacks: &[String],
) -> Vec<StyleProperty<'static, u32>> {
    let (collapse, wrap) = white_space(style);
    let feature_list = features(style);
    let variation_list = variations(style);
    let family_list = families(style, fallbacks);
    vec![
        StyleProperty::FontFamily(FontFamily::List(Cow::Owned(family_list))),
        StyleProperty::FontSize(narrow(style.font_size)),
        StyleProperty::FontWeight(FontWeight::new(narrow(style.font_weight))),
        StyleProperty::FontStyle(match style.font_style {
            SceneFontStyle::Normal => FontStyle::Normal,
            SceneFontStyle::Italic => FontStyle::Italic,
            SceneFontStyle::Oblique => FontStyle::Oblique(None),
        }),
        StyleProperty::LineHeight(line_height(style)),
        StyleProperty::LetterSpacing(narrow(style.letter_spacing)),
        StyleProperty::WordSpacing(narrow(style.word_spacing)),
        StyleProperty::WhiteSpaceCollapse(collapse),
        StyleProperty::TextWrapMode(wrap),
        StyleProperty::WordBreak(match style.word_break {
            SceneWordBreak::Normal => WordBreak::Normal,
            SceneWordBreak::BreakAll => WordBreak::BreakAll,
            SceneWordBreak::KeepAll => WordBreak::KeepAll,
        }),
        StyleProperty::OverflowWrap(match style.overflow_wrap {
            SceneOverflowWrap::Normal => OverflowWrap::Normal,
            SceneOverflowWrap::Anywhere => OverflowWrap::Anywhere,
            SceneOverflowWrap::BreakWord => OverflowWrap::BreakWord,
        }),
        StyleProperty::FontFeatures(parley::FontFeatures::Source(Cow::Owned(feature_list))),
        StyleProperty::FontVariations(parley::FontVariations::Source(Cow::Owned(variation_list))),
        StyleProperty::Brush(brush),
    ]
}

fn push_style(
    builder: &mut parley::RangedBuilder<'_, u32>,
    style: &TextStyle,
    range: std::ops::Range<usize>,
    (brush, fallbacks): (u32, &[String]),
) {
    for prop in style_props(style, brush, fallbacks) {
        builder.push(prop, range.clone());
    }
}

fn push_root(
    builder: &mut parley::RangedBuilder<'_, u32>,
    style: &TextStyle,
    fallbacks: &[String],
) {
    for prop in style_props(style, u32::MAX, fallbacks) {
        builder.push_default(prop);
    }
}

const SPACED_PROBE: &str = "x x";
const JOINED_PROBE: &str = "xx";
const HYPHEN_PROBE: &str = "-";
const SOFT_HYPHEN: char = '\u{ad}';
const INLINE_OBJECT: char = '\u{fffc}';
const HYPHEN_ROUNDS: usize = 3;
const X_HEIGHT_SHARE: f64 = 0.5;
const ZERO_PROBE: &str = "0";

pub fn probe(tree: &mut Tree<'_>, style: &TextStyle, text: &str) -> Paragraph<u32> {
    let fallbacks = tree.fonts.fallbacks().to_vec();
    let fonts = &mut tree.fonts.context;
    let mut builder = tree.text_layouts.ranged_builder(fonts, text, 1.0, false);
    builder.set_line_break_override(Some(CHROMIUM_LINE_BREAK_OVERRIDE));
    push_root(&mut builder, style, &fallbacks);
    let mut paragraph: Paragraph<u32> = builder.build(text);
    paragraph.break_all_lines(None);
    paragraph
}

pub fn font_extent(tree: &mut Tree<'_>, style: &TextStyle) -> FontExtent {
    let key = format!(
        "{:?}|{}|{}|{:?}|{}|{}|{:?}",
        style.families,
        style.font_size,
        style.font_weight,
        style.font_style,
        style.letter_spacing,
        style.word_spacing,
        style.text_rendering
    );
    if let Some(found) = tree.extents.get(&key) {
        return *found;
    }
    let spaced = probe(tree, style, SPACED_PROBE);
    let joined = probe(tree, style, JOINED_PROBE);
    let hyphen = probe(tree, style, HYPHEN_PROBE).width();
    let zero = probe(tree, style, ZERO_PROBE).width();
    let mut extent = FontExtent {
        ascent: narrow(style.font_size * 0.8),
        descent: narrow(style.font_size * 0.2),
        gap: 0.0,
        space: spaced.width() - joined.width(),
        hyphen,
        x_height: narrow(style.font_size * X_HEIGHT_SHARE),
        zero,
        subpixel: style.text_rendering == TextRendering::GeometricPrecision,
    };
    if let Some(line) = joined.lines().next() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(run) = item {
                let metrics = run.run().font_metrics();
                extent.ascent = metrics.ascent;
                extent.descent = metrics.descent;
                extent.gap = metrics.leading;
                if let Some(x_height) = metrics.x_height {
                    extent.x_height = x_height;
                }
            }
        }
    }
    tree.extents.insert(key, extent);
    extent
}

type LineContent = (Vec<(TextStyle, FontExtent, f32)>, Vec<(BoxSize, Placement)>);

#[derive(Debug, Clone, Copy)]
struct Extent {
    above: f32,
    below: f32,
}

const LAYOUT_UNITS_PER_PX: f64 = 64.0;

fn layout_unit_floor(px: f64) -> f32 {
    narrow((px * LAYOUT_UNITS_PER_PX).floor() / LAYOUT_UNITS_PER_PX)
}

fn layout_unit_round(px: f64) -> f32 {
    narrow((px * LAYOUT_UNITS_PER_PX).round() / LAYOUT_UNITS_PER_PX)
}

fn text_extent(style: &TextStyle, font: FontExtent) -> Extent {
    let (ascent, descent) = (font.rounded_ascent(), font.rounded_descent());
    let line_height = match style.line_height {
        LineSpacing::Normal => ascent + descent + font.gap.round(),
        LineSpacing::Number(_) | LineSpacing::Px(_) => layout_unit_round(style.line_height_px()),
    };
    let leading = line_height - (ascent + descent);
    let top = (leading / 2.0).floor();
    Extent {
        above: ascent + top,
        below: descent + (leading - top),
    }
}

fn line_boxes(tree: &mut Tree<'_>, id: usize) -> Vec<LineBox> {
    let crate::tree::Content::Inline(content) = &tree.nodes[id].content else {
        return Vec::new();
    };
    let Some(paragraph) = &content.paragraph else {
        return Vec::new();
    };
    let items = content.items.clone();
    let root = content.root.clone();
    let sizes = content.box_sizes.clone();
    let mut per_line: Vec<LineContent> = Vec::new();
    let box_items: Vec<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| matches!(item, InlineItem::Box { .. }))
        .map(|(i, _)| i)
        .collect();
    for line in paragraph.lines() {
        let mut texts = Vec::new();
        let mut boxes = Vec::new();
        for item in line.items() {
            match item {
                PositionedLayoutItem::GlyphRun(run) => {
                    let index = usize::try_from(run.style().brush).unwrap_or(usize::MAX);
                    if let Some(InlineItem::Text { style, shift, .. }) = items.get(index) {
                        let metrics = run.run().font_metrics();
                        texts.push((
                            style.clone(),
                            FontExtent {
                                ascent: metrics.ascent,
                                descent: metrics.descent,
                                gap: metrics.leading,
                                space: 0.0,
                                hyphen: 0.0,
                                x_height: 0.0,
                                zero: 0.0,
                                subpixel: style.text_rendering == TextRendering::GeometricPrecision,
                            },
                            *shift,
                        ));
                    }
                }
                PositionedLayoutItem::InlineBox(placed) => {
                    let index = usize::try_from(placed.id).unwrap_or(usize::MAX);
                    if let Some(position) = box_items.iter().position(|&i| i == index)
                        && let Some(size) = sizes.get(position)
                        && let Some(InlineItem::Box { placement, .. }) = items.get(index)
                    {
                        boxes.push((*size, *placement));
                    }
                }
            }
        }
        per_line.push((texts, boxes));
    }
    let strut_font = font_extent(tree, &root);
    let strut = text_extent(&root, strut_font);
    let mut top = 0.0;
    let mut out = Vec::new();
    for (texts, boxes) in per_line {
        let mut above = strut.above;
        let mut below = strut.below;
        for (style, font, shift) in &texts {
            let extent = text_extent(style, *font);
            above = above.max(extent.above - shift);
            below = below.max(extent.below + shift);
        }
        for (size, placement) in &boxes {
            let box_above = size.outer_baseline();
            let box_below = size.outer_height() - box_above;
            if let Some(shift) = placement.shift(box_above, box_below) {
                above = above.max(box_above - shift);
                below = below.max(box_below + shift);
            }
        }
        for (size, placement) in &boxes {
            let spare = size.outer_height() - (above + below);
            if spare > 0.0 {
                match placement {
                    Placement::LineTop => below += spare,
                    Placement::LineBottom => above += spare,
                    _ => {}
                }
            }
        }
        let height = above + below;
        out.push(LineBox {
            top,
            baseline: top + above,
            height,
        });
        top += height;
    }
    out
}

const fn alignment(style: &TextStyle) -> Alignment {
    match style.text_align {
        TextAlign::Start => Alignment::Start,
        TextAlign::End => Alignment::End,
        TextAlign::Left => Alignment::Left,
        TextAlign::Right => Alignment::Right,
        TextAlign::Center => Alignment::Center,
        TextAlign::Justify => Alignment::Justify,
    }
}

pub fn text_of(items: &[InlineItem]) -> (String, Vec<(usize, usize)>) {
    let mut text = String::new();
    let mut ranges = Vec::new();
    for item in items {
        let start = text.len();
        match item {
            InlineItem::Text { text: t, .. } => text.push_str(t),
            InlineItem::Box { .. } | InlineItem::Tab { .. } | InlineItem::Edge { .. } => {
                text.push(INLINE_OBJECT);
            }
            InlineItem::SpanStart { .. } | InlineItem::SpanEnd => {}
        }
        ranges.push((start, text.len()));
    }
    (text, ranges)
}

fn paragraph_key(
    items: &[InlineItem],
    root: &TextStyle,
    sizes: &[BoxSize],
    tabs: &[f32],
    fallbacks: &[String],
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    format!("{items:?}|{root:?}|{sizes:?}|{tabs:?}|{fallbacks:?}").hash(&mut hasher);
    hasher.finish()
}

fn build_paragraph(tree: &mut Tree<'_>, id: usize) -> Option<Paragraph<u32>> {
    let crate::tree::Content::Inline(content) = &tree.nodes[id].content else {
        return None;
    };
    let items = content.items.clone();
    let root = content.root.clone();
    let sizes = content.box_sizes.clone();
    let tabs = content.tab_widths.clone();
    let fallbacks = tree.fonts.fallbacks().to_vec();
    let key = paragraph_key(&items, &root, &sizes, &tabs, &fallbacks);
    if let Some(paragraph) = tree.fonts.shaped(key) {
        return Some(paragraph);
    }
    let paragraph = shaped_paragraph(tree, &items, &root, (&sizes, &tabs), &fallbacks);
    tree.fonts.keep_shaped(key, &paragraph);
    Some(paragraph)
}

fn shaped_paragraph(
    tree: &mut Tree<'_>,
    items: &[InlineItem],
    root: &TextStyle,
    (sizes, tabs): (&[BoxSize], &[f32]),
    fallbacks: &[String],
) -> Paragraph<u32> {
    let (text, ranges) = text_of(items);
    let fonts = &mut tree.fonts.context;
    let mut builder = tree.text_layouts.ranged_builder(fonts, &text, 1.0, false);
    if !breaks_all(root, items) {
        builder.set_line_break_override(Some(CHROMIUM_LINE_BREAK_OVERRIDE));
    }
    push_root(&mut builder, root, fallbacks);
    let mut box_index = 0;
    let mut tab_index = 0;
    for (index, (item, (start, end))) in items.iter().zip(&ranges).enumerate() {
        if matches!(
            item,
            InlineItem::Box { .. } | InlineItem::Tab { .. } | InlineItem::Edge { .. }
        ) {
            builder.push(StyleProperty::LetterSpacing(0.0), *start..*end);
        }
        match item {
            InlineItem::Text { style, .. } => {
                push_style(
                    &mut builder,
                    style,
                    *start..*end,
                    (u32::try_from(index).unwrap_or(u32::MAX), fallbacks),
                );
            }
            InlineItem::Box { .. } => {
                let size = sizes.get(box_index).copied().unwrap_or(BoxSize::EMPTY);
                builder.push_inline_box(InlineBox {
                    id: u64::try_from(index).unwrap_or(u64::MAX),
                    kind: InlineBoxKind::InFlow,
                    index: *start,
                    width: size.outer_width(),
                    height: size.outer_height(),
                    baseline: Some(size.outer_baseline()),
                    vertical_align: VerticalAlign::BASELINE,
                });
                box_index += 1;
            }
            InlineItem::Tab { .. } => {
                builder.push_inline_box(InlineBox {
                    id: u64::try_from(index).unwrap_or(u64::MAX),
                    kind: InlineBoxKind::InFlow,
                    index: *start,
                    width: tabs.get(tab_index).copied().unwrap_or(0.0),
                    height: 0.0,
                    baseline: None,
                    vertical_align: VerticalAlign::BASELINE,
                });
                tab_index += 1;
            }
            InlineItem::Edge { width, .. } => {
                builder.push_inline_box(InlineBox {
                    id: u64::try_from(index).unwrap_or(u64::MAX),
                    kind: InlineBoxKind::InFlow,
                    index: *start,
                    width: *width,
                    height: 0.0,
                    baseline: None,
                    vertical_align: VerticalAlign::BASELINE,
                });
            }
            InlineItem::SpanStart { .. } | InlineItem::SpanEnd => {}
        }
    }
    builder.build(&text)
}

fn breaks_all(root: &TextStyle, items: &[InlineItem]) -> bool {
    root.word_break == SceneWordBreak::BreakAll
        || items.iter().any(|item| {
            matches!(item, InlineItem::Text { style, .. } if style.word_break == SceneWordBreak::BreakAll)
        })
}

const TAB_ROUNDS: usize = 3;
const NARROWEST_TAB: f32 = 0.5;

fn tab_widths(tree: &mut Tree<'_>, id: usize) -> Vec<f32> {
    let crate::tree::Content::Inline(content) = &tree.nodes[id].content else {
        return Vec::new();
    };
    let items = content.items.clone();
    let mut positions: Vec<(usize, f32)> = Vec::new();
    if let Some(paragraph) = &content.paragraph {
        for line in paragraph.lines() {
            let start = line.metrics().offset;
            for item in line.items() {
                if let PositionedLayoutItem::InlineBox(placed) = item
                    && let Ok(index) = usize::try_from(placed.id)
                    && matches!(items.get(index), Some(InlineItem::Tab { .. }))
                {
                    positions.push((index, placed.x - start));
                }
            }
        }
    }
    positions.sort_by_key(|(index, _)| *index);
    positions
        .into_iter()
        .map(|(index, x)| {
            let Some(InlineItem::Tab { style, .. }) = items.get(index) else {
                return 0.0;
            };
            let space = font_extent(tree, style).space;
            let interval = narrow(style.tab_size) * space;
            if interval <= 0.0 {
                return 0.0;
            }
            let mut stop = ((x / interval).floor() + 1.0) * interval;
            if stop - x < NARROWEST_TAB * space {
                stop += interval;
            }
            stop - x
        })
        .collect()
}

fn same_widths(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| exact_eq(f64::from(*x), f64::from(*y)))
}

fn has_tabs(tree: &Tree<'_>, id: usize) -> bool {
    match &tree.nodes[id].content {
        crate::tree::Content::Inline(content) => content
            .items
            .iter()
            .any(|item| matches!(item, InlineItem::Tab { .. })),
        _ => false,
    }
}

fn measure_box(tree: &mut Tree<'_>, node: usize, available: AvailableSpace) -> BoxSize {
    let input = |width: AvailableSpace, known: Option<f32>| LayoutInput {
        known_dimensions: Size {
            width: known,
            height: None,
        },
        known_dimensions_are_definite: Size {
            width: known.is_some(),
            height: false,
        },
        parent_size: Size {
            width: available.into_option(),
            height: None,
        },
        available_space: Size {
            width,
            height: AvailableSpace::MaxContent,
        },
        run_mode: RunMode::ComputeSize,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        vertical_margins_are_collapsible: Line::FALSE,
    };
    let id = NodeId::from(node);
    let fixed_width = tree.nodes[node].style.size.width.into_option();
    let width = if fixed_width.is_some() {
        None
    } else {
        let min = tree
            .compute_child_layout(id, input(AvailableSpace::MinContent, None))
            .size
            .width;
        let max = tree
            .compute_child_layout(id, input(AvailableSpace::MaxContent, None))
            .size
            .width;
        Some(match available {
            AvailableSpace::Definite(w) => max.min(min.max(w)),
            AvailableSpace::MinContent => min,
            AvailableSpace::MaxContent => max,
        })
    };
    let output = tree.compute_child_layout(id, input(available, width));
    let baseline = output
        .baselines
        .first
        .or_else(|| synthesized_baseline(tree, node, output.size));
    let basis = available.into_option();
    let sides = tree.nodes[node].style.margin;
    let margin = taffy::Rect {
        left: sides
            .left
            .resolve_or_zero(basis, |t, b| tree.resolve_calc_value(t, b)),
        right: sides
            .right
            .resolve_or_zero(basis, |t, b| tree.resolve_calc_value(t, b)),
        top: sides
            .top
            .resolve_or_zero(basis, |t, b| tree.resolve_calc_value(t, b)),
        bottom: sides
            .bottom
            .resolve_or_zero(basis, |t, b| tree.resolve_calc_value(t, b)),
    };
    BoxSize {
        width: output.size.width,
        height: output.size.height,
        baseline,
        margin,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Which {
    First,
    Last,
}

fn line_baseline(tree: &Tree<'_>, node: usize, which: Which) -> Option<f32> {
    let current = &tree.nodes[node];
    if let crate::tree::Content::Inline(content) = &current.content {
        let line = match which {
            Which::First => content.lines.first(),
            Which::Last => content.lines.last(),
        };
        return line.map(|l| l.baseline);
    }
    if which == Which::Last
        && (current.style.overflow.x != taffy::Overflow::Visible
            || current.style.overflow.y != taffy::Overflow::Visible)
    {
        return None;
    }
    let in_flow = current
        .children
        .iter()
        .copied()
        .filter(|&c| tree.nodes[c].style.position != Position::Absolute);
    let ordered: Vec<usize> = match which {
        Which::First => in_flow.collect(),
        Which::Last => in_flow.rev().collect(),
    };
    ordered.into_iter().find_map(|child| {
        line_baseline(tree, child, which).map(|b| tree.nodes[child].layout.location.y + b)
    })
}

fn synthesized_baseline(tree: &mut Tree<'_>, node: usize, size: Size<f32>) -> Option<f32> {
    let flex_or_grid = matches!(
        tree.nodes[node].style.display,
        Display::Flex | Display::Grid
    );
    let known = Size {
        width: Some(size.width),
        height: Some(size.height),
    };
    tree.compute_child_layout(
        NodeId::from(node),
        LayoutInput {
            known_dimensions: known,
            known_dimensions_are_definite: Size {
                width: true,
                height: true,
            },
            parent_size: known,
            available_space: Size {
                width: AvailableSpace::Definite(size.width),
                height: AvailableSpace::Definite(size.height),
            },
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            vertical_margins_are_collapsible: Line::FALSE,
        },
    );
    if !flex_or_grid {
        return line_baseline(tree, node, Which::Last);
    }
    let first = tree.nodes[node]
        .children
        .iter()
        .copied()
        .find(|&c| tree.nodes[c].style.position != Position::Absolute)?;
    let layout = tree.nodes[first].layout;
    Some(
        line_baseline(tree, first, Which::First)
            .map_or(layout.location.y + layout.size.height, |b| {
                layout.location.y + b
            }),
    )
}

fn shape_paragraph(
    tree: &mut Tree<'_>,
    id: usize,
    width: Option<f32>,
    available: AvailableSpace,
) -> Option<f32> {
    let mut paragraph = build_paragraph(tree, id)?;
    let root_align = match &tree.nodes[id].content {
        crate::tree::Content::Inline(content) => alignment(&content.root),
        _ => Alignment::Start,
    };
    let max = match available {
        AvailableSpace::MinContent => Some(paragraph.calculate_content_widths().min),
        AvailableSpace::MaxContent => None,
        AvailableSpace::Definite(w) => Some(w),
    };
    let limit = width.or(max);
    paragraph.set_alignment(
        root_align,
        AlignmentOptions {
            align_when_overflowing: false,
            last_line_alignment: None,
        },
    );
    let mut narrowed: Vec<(usize, f32)> = Vec::new();
    for _ in 0..HYPHEN_ROUNDS {
        break_lines(&mut paragraph, limit, &narrowed);
        let Some(limit) = limit else {
            break;
        };
        let overflowing: Vec<(usize, f32)> = hyphenated_lines(tree, id, &paragraph)
            .into_iter()
            .filter(|hyphen| {
                !narrowed.iter().any(|(n, _)| *n == hyphen.line)
                    && paragraph
                        .lines()
                        .nth(hyphen.line)
                        .is_some_and(|l| l.metrics().advance + hyphen.width > limit)
            })
            .map(|hyphen| (hyphen.line, hyphen.width))
            .collect();
        if overflowing.is_empty() {
            break;
        }
        narrowed.extend(overflowing);
    }
    let line_width = width.unwrap_or_else(|| paragraph.width());
    let hyphens = hyphenated_lines(tree, id, &paragraph);
    if let crate::tree::Content::Inline(content) = &mut tree.nodes[id].content {
        content.paragraph = Some(paragraph);
        content.hyphens = hyphens;
    }
    Some(line_width)
}

fn break_lines(paragraph: &mut Paragraph<u32>, limit: Option<f32>, narrowed: &[(usize, f32)]) {
    let limit = limit.unwrap_or(f32::MAX);
    let mut lines = paragraph.break_lines();
    let mut index = 0;
    loop {
        let advance = narrowed
            .iter()
            .find(|(line, _)| *line == index)
            .map_or(limit, |(_, hyphen)| limit - hyphen);
        lines.state_mut().set_layout_max_advance(advance);
        lines.state_mut().set_line_max_advance(advance);
        match lines.break_next() {
            None => break,
            Some(YieldData::InlineBoxBreak(_)) => {
                let x = lines.state().line_x();
                lines.state_mut().append_inline_box_to_line(x, 0.0, 0.0);
            }
            Some(YieldData::LineBreak(_)) => index += 1,
            Some(_) => {}
        }
    }
    lines.finish();
}

fn hyphenated_lines(tree: &mut Tree<'_>, id: usize, paragraph: &Paragraph<u32>) -> Vec<Hyphen> {
    let crate::tree::Content::Inline(content) = &tree.nodes[id].content else {
        return Vec::new();
    };
    let items = content.items.clone();
    let (text, ranges) = text_of(&items);
    paragraph
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let end = line.text_range().end;
            text.get(..end)
                .is_some_and(|t| t.ends_with(SOFT_HYPHEN))
                .then(|| (index, end - SOFT_HYPHEN.len_utf8()))
        })
        .filter_map(|(line, at)| {
            let item = ranges
                .iter()
                .position(|(start, end)| (*start..*end).contains(&at))?;
            let InlineItem::Text { style, .. } = &items[item] else {
                return None;
            };
            Some(Hyphen {
                line,
                item,
                width: font_extent(tree, style).hyphen,
            })
        })
        .collect()
}

pub fn lay_out_paragraph(
    tree: &mut Tree<'_>,
    id: usize,
    width: Option<f32>,
    available: AvailableSpace,
) -> (f32, f32, Option<f32>) {
    let boxes = tree.inline_box_ids(id);
    let sizes: Vec<BoxSize> = boxes
        .iter()
        .map(|&b| measure_box(tree, b, available))
        .collect();
    if let crate::tree::Content::Inline(content) = &mut tree.nodes[id].content {
        content.box_sizes = sizes;
        content.tab_widths.clear();
    }
    let Some(mut line_width) = shape_paragraph(tree, id, width, available) else {
        return (0.0, 0.0, None);
    };
    if has_tabs(tree, id) {
        for _ in 0..TAB_ROUNDS {
            let wanted = tab_widths(tree, id);
            let settled = match &tree.nodes[id].content {
                crate::tree::Content::Inline(content) => same_widths(&content.tab_widths, &wanted),
                _ => true,
            };
            if settled {
                break;
            }
            if let crate::tree::Content::Inline(content) = &mut tree.nodes[id].content {
                content.tab_widths = wanted;
            }
            line_width = shape_paragraph(tree, id, width, available).unwrap_or(line_width);
        }
    }
    let lines = line_boxes(tree, id);
    let height = lines.last().map_or(0.0, |l| l.top + l.height);
    let baseline = lines.first().map(|l| l.baseline);
    if let crate::tree::Content::Inline(content) = &mut tree.nodes[id].content {
        content.lines = lines;
    }
    (line_width, height, baseline)
}

pub fn compute_inline_layout(tree: &mut Tree<'_>, id: usize, inputs: LayoutInput) -> LayoutOutput {
    let available = match inputs.known_dimensions.width {
        Some(width) => AvailableSpace::Definite(width),
        None => inputs.available_space.width,
    };
    let (width, height, baseline) =
        lay_out_paragraph(tree, id, inputs.known_dimensions.width, available);
    let size = Size {
        width: inputs.known_dimensions.width.unwrap_or(width),
        height: inputs.known_dimensions.height.unwrap_or(height),
    };
    LayoutOutput {
        baselines: Baselines::from_first(baseline),
        ..LayoutOutput::from_outer_size(size)
    }
}
