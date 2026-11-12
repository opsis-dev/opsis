use opsis_scene::style::{Style as SceneStyle, WhiteSpace};
use parley::PositionedLayoutItem;
use taffy::prelude::*;
use taffy::{LayoutInput, LayoutPartialTree, RequestedAxis, RunMode, SizingMode};

use crate::computed::TextStyle;
use crate::inline::{
    BoxSize, FontExtent, InlineItem, Placement, SpanEdges, font_extent, lay_out_paragraph,
};
use crate::transform::{Matrix, local, perspective_of};
use crate::tree::{Content, Tag, Tree};

#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Self {
            x,
            y,
            width: (self.x + self.width).max(other.x + other.width) - x,
            height: (self.y + self.height).max(other.y + other.height) - y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacedGlyph {
    pub id: u16,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub struct TextRun {
    pub font: parley::FontData,
    pub size: f32,
    pub embolden: bool,
    pub skew: f32,
    pub variations: Vec<([u8; 4], f32)>,
    pub glyphs: Vec<PlacedGlyph>,
    pub baseline: f64,
    pub start: f64,
    pub advance: f64,
    pub underline: (f32, f32),
    pub strikethrough: (f32, f32),
    pub ascent: f32,
}

impl PartialEq for TextRun {
    fn eq(&self, other: &Self) -> bool {
        self.font.data.id() == other.font.data.id()
            && self.font.index == other.font.index
            && self.size.to_bits() == other.size.to_bits()
            && self.embolden == other.embolden
            && self.skew.to_bits() == other.skew.to_bits()
            && self.variations == other.variations
            && self.glyphs == other.glyphs
            && self.baseline.to_bits() == other.baseline.to_bits()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlacedKind {
    Element(Tag),
    Text {
        lines: Vec<Rect>,
        runs: Vec<TextRun>,
    },
}

pub fn run_of(
    glyph_run: &parley::GlyphRun<'_, u32>,
    style: &TextStyle,
    origin: (f64, f64),
    baseline: f32,
) -> TextRun {
    let run = glyph_run.run();
    let synthesis = run.synthesis();
    let mut variations: Vec<([u8; 4], f32)> = synthesis
        .variation_settings()
        .iter()
        .map(|(tag, value)| (tag.to_be_bytes(), *value))
        .collect();
    for (tag, value) in &style.variations {
        let value = opsis_types::numbers::narrow(value.0);
        match variations.iter_mut().find(|(t, _)| *t == tag.0) {
            Some(found) => found.1 = value,
            None => variations.push((tag.0, value)),
        }
    }
    let run_baseline = glyph_run.baseline();
    let metrics = run.font_metrics();
    TextRun {
        baseline: origin.1 + to_f64(baseline),
        start: origin.0 + to_f64(glyph_run.offset()),
        advance: to_f64(glyph_run.advance()),
        underline: (metrics.underline_offset, metrics.underline_size),
        strikethrough: (metrics.strikethrough_offset, metrics.strikethrough_size),
        ascent: metrics.ascent,
        font: run.font().clone(),
        size: run.font_size(),
        embolden: synthesis.embolden(),
        skew: synthesis.skew().unwrap_or(0.0),
        variations,
        glyphs: glyph_run
            .positioned_glyphs()
            .filter_map(|glyph| {
                Some(PlacedGlyph {
                    id: u16::try_from(glyph.id).ok()?,
                    x: origin.0 + to_f64(glyph.x),
                    y: origin.1 + to_f64(baseline + glyph.y - run_baseline),
                })
            })
            .collect(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Sides {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Sides {
    fn of(rect: taffy::Rect<f32>) -> Self {
        Self {
            top: to_f64(rect.top),
            right: to_f64(rect.right),
            bottom: to_f64(rect.bottom),
            left: to_f64(rect.left),
        }
    }

    pub fn inset(&self, rect: Rect) -> Rect {
        Rect {
            x: rect.x + self.left,
            y: rect.y + self.top,
            width: (rect.width - self.left - self.right).max(0.0),
            height: (rect.height - self.top - self.bottom).max(0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Placed {
    pub drawn: Vec<(usize, Vec<TextRun>)>,
    pub path: Vec<usize>,
    pub kind: PlacedKind,
    pub border_box: Rect,
    pub border: Sides,
    pub padding: Sides,
    pub matrix: Matrix,
    pub client: Rect,
    pub style: Option<SceneStyle>,
    pub text: TextStyle,
}

#[derive(Debug, Clone, Default)]
pub struct Laid {
    pub boxes: Vec<Placed>,
}

impl Laid {
    pub fn at(&self, path: &[usize]) -> Option<&Placed> {
        self.boxes.iter().find(|b| b.path == path)
    }
}

fn perform(tree: &mut Tree<'_>, node: usize, width: f32, height: f32) {
    let size = Size {
        width: Some(width),
        height: Some(height),
    };
    tree.compute_child_layout(
        NodeId::from(node),
        LayoutInput {
            known_dimensions: size,
            known_dimensions_are_definite: Size {
                width: true,
                height: true,
            },
            parent_size: size,
            available_space: Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(height),
            },
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            vertical_margins_are_collapsible: Line::FALSE,
        },
    );
}

fn to_f64(value: f32) -> f64 {
    f64::from(value)
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Frame {
    pub origin: (f64, f64),
    pub matrix: Matrix,
    pub perspective: Option<Matrix>,
}

impl Frame {
    pub(crate) const ROOT: Self = Self {
        origin: (0.0, 0.0),
        matrix: Matrix::IDENTITY,
        perspective: None,
    };

    fn inherited(&self) -> Matrix {
        self.perspective
            .map_or(self.matrix, |p| self.matrix.then(&p))
    }
}

pub(crate) struct Walk<'t, 'f> {
    pub tree: &'t mut Tree<'f>,
    pub out: Vec<Placed>,
}

impl Walk<'_, '_> {
    pub(crate) fn node(&mut self, id: usize, frame: &Frame) {
        let layout = self.tree.nodes[id].layout;
        let x = frame.origin.0 + to_f64(layout.location.x);
        let y = frame.origin.1 + to_f64(layout.location.y);
        let border_box = Rect {
            x,
            y,
            width: to_f64(layout.size.width),
            height: to_f64(layout.size.height),
        };
        if matches!(self.tree.nodes[id].content, Content::Inline(_)) {
            self.paragraph(
                id,
                &Frame {
                    origin: (x, y),
                    ..*frame
                },
                layout.size.width,
            );
            return;
        }
        let node = &self.tree.nodes[id];
        let context = self.tree.context;
        let (matrix, perspective) = node.scene.as_ref().map_or_else(
            || (frame.inherited(), None),
            |style| {
                (
                    frame
                        .inherited()
                        .then(&local(style, &node.text, &context, &border_box)),
                    perspective_of(style, &node.text, &context, &border_box),
                )
            },
        );
        self.out.push(Placed {
            drawn: node.drawn.clone(),
            path: node.path.clone(),
            kind: PlacedKind::Element(node.tag),
            border_box,
            border: Sides::of(layout.border),
            padding: Sides::of(layout.padding),
            matrix,
            client: matrix.bounds(border_box),
            style: node.scene.clone(),
            text: node.text.clone(),
        });
        let inner = Frame {
            origin: (x, y),
            matrix,
            perspective,
        };
        let children = node.children.clone();
        let hoisted: Vec<usize> = node.hoisted.iter().map(|h| usize::from(*h)).collect();
        for child in children {
            if self.tree.nodes[child].style.position == Position::Absolute {
                continue;
            }
            self.node(child, &inner);
        }
        for child in hoisted {
            self.node(child, &inner);
        }
    }

    fn paragraph(&mut self, id: usize, frame: &Frame, width: f32) {
        lay_out_paragraph(self.tree, id, Some(width), AvailableSpace::Definite(width));
        let items = match &self.tree.nodes[id].content {
            Content::Inline(content) => content.items.clone(),
            _ => return,
        };
        let mut enclosing: Vec<TextStyle> = Vec::new();
        let mut tab_fonts: Vec<Option<FontExtent>> = Vec::with_capacity(items.len());
        for item in &items {
            tab_fonts.push(match item {
                InlineItem::Tab { style, .. } => Some(font_extent(self.tree, style)),
                InlineItem::Edge { .. } => enclosing
                    .last()
                    .cloned()
                    .map(|style| font_extent(self.tree, &style)),
                InlineItem::SpanStart { style, .. } => {
                    enclosing.push(style.clone());
                    None
                }
                InlineItem::SpanEnd => {
                    enclosing.pop();
                    None
                }
                InlineItem::Text { .. } | InlineItem::Box { .. } => None,
            });
        }
        let Content::Inline(content) = &self.tree.nodes[id].content else {
            return;
        };
        let origin = frame.origin;
        let matrix = frame.inherited();
        let line_boxes = content.lines.clone();
        let box_items: Vec<usize> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| matches!(item, InlineItem::Box { .. }))
            .map(|(i, _)| i)
            .collect();
        let sizes = content.box_sizes.clone();
        let hyphens = content.hyphens.clone();
        let mut lines: Vec<Vec<(usize, Rect)>> = vec![Vec::new(); items.len()];
        let mut runs: Vec<Vec<TextRun>> = vec![Vec::new(); items.len()];
        let mut boxes: Vec<(usize, Rect)> = Vec::new();
        if let Some(paragraph) = &content.paragraph {
            for (line_index, line) in paragraph.lines().enumerate() {
                let baseline = line_boxes
                    .get(line_index)
                    .map_or_else(|| line.metrics().baseline, |l| l.baseline);
                let (line_top, line_height) = line_boxes
                    .get(line_index)
                    .map_or((0.0, 0.0), |l| (l.top, l.height));
                let mut on_line: Vec<Option<Rect>> = vec![None; items.len()];
                let last_run = line
                    .items()
                    .enumerate()
                    .filter(|(_, item)| matches!(item, PositionedLayoutItem::GlyphRun(_)))
                    .map(|(i, _)| i)
                    .last();
                let hanging = line.metrics().hanging_advance;
                for (position, item) in line.items().enumerate() {
                    match item {
                        PositionedLayoutItem::GlyphRun(run) => {
                            let Ok(index) = usize::try_from(run.style().brush) else {
                                continue;
                            };
                            let metrics = run.run().font_metrics();
                            let (ascent, descent) =
                                (metrics.ascent.round(), metrics.descent.round());
                            let keeps_spaces = matches!(
                                items.get(index),
                                Some(InlineItem::Text { style, .. })
                                    if style.white_space == WhiteSpace::PreWrap
                            );
                            let advance = if Some(position) == last_run && !keeps_spaces {
                                (run.advance() - hanging).max(0.0)
                            } else {
                                run.advance()
                            };
                            let shift = match items.get(index) {
                                Some(InlineItem::Text { shift, .. }) => *shift,
                                _ => 0.0,
                            };
                            if let Some(InlineItem::Text { style, .. }) = items.get(index) {
                                runs[index].push(run_of(&run, style, origin, baseline + shift));
                            }
                            let rect = Rect {
                                x: origin.0 + to_f64(run.offset()),
                                y: origin.1 + to_f64(baseline - ascent + shift),
                                width: to_f64(advance),
                                height: to_f64(ascent + descent),
                            };
                            if rect.width <= 0.0 {
                                continue;
                            }
                            if let Some(slot) = on_line.get_mut(index) {
                                *slot = Some(slot.map_or(rect, |r| r.union(rect)));
                            }
                        }
                        PositionedLayoutItem::InlineBox(placed) => {
                            let Ok(index) = usize::try_from(placed.id) else {
                                continue;
                            };
                            if let Some(Some(font)) = tab_fonts.get(index) {
                                let (ascent, descent) =
                                    (font.rounded_ascent(), font.rounded_descent());
                                let (skip_start, skip_end, shift) = match items.get(index) {
                                    Some(InlineItem::Edge {
                                        skip_start,
                                        skip_end,
                                        shift,
                                        ..
                                    }) => (*skip_start, *skip_end, *shift),
                                    Some(InlineItem::Tab { shift, .. }) => (0.0, 0.0, *shift),
                                    _ => (0.0, 0.0, 0.0),
                                };
                                let rect = Rect {
                                    x: origin.0 + to_f64(placed.x + skip_start),
                                    y: origin.1 + to_f64(baseline - ascent + shift),
                                    width: to_f64(placed.width - skip_start - skip_end),
                                    height: to_f64(ascent + descent),
                                };
                                if rect.width > 0.0
                                    && let Some(slot) = on_line.get_mut(index)
                                {
                                    *slot = Some(slot.map_or(rect, |r| r.union(rect)));
                                }
                                continue;
                            }
                            let size = box_items
                                .iter()
                                .position(|&i| i == index)
                                .and_then(|p| sizes.get(p))
                                .copied()
                                .unwrap_or(BoxSize {
                                    width: placed.width,
                                    height: placed.height,
                                    ..BoxSize::EMPTY
                                });
                            let placement = match items.get(index) {
                                Some(InlineItem::Box { placement, .. }) => *placement,
                                _ => Placement::Shift(0.0),
                            };
                            let above = size.outer_baseline();
                            let below = size.outer_height() - above;
                            let margin_top = match placement.shift(above, below) {
                                Some(shift) => baseline + shift - above,
                                None if placement == Placement::LineTop => line_top,
                                None => line_top + line_height - size.outer_height(),
                            };
                            boxes.push((
                                index,
                                Rect {
                                    x: origin.0 + to_f64(placed.x + size.margin.left),
                                    y: origin.1 + to_f64(margin_top + size.margin.top),
                                    width: to_f64(size.width),
                                    height: to_f64(size.height),
                                },
                            ));
                        }
                    }
                }
                for hyphen in hyphens.iter().filter(|h| h.line == line_index) {
                    if let Some(Some(rect)) = on_line.get_mut(hyphen.item) {
                        rect.width += to_f64(hyphen.width);
                    }
                }
                for (index, rect) in on_line.into_iter().enumerate() {
                    if let Some(rect) = rect {
                        lines[index].push((line_index, rect));
                    }
                }
            }
        }
        let mut spans: Vec<OpenSpan> = Vec::new();
        let mut pending: Option<PendingText> = None;
        for (index, item) in items.iter().enumerate() {
            let shift = spans.last().map_or((0.0, 0.0), |span| span.shift);
            for (_, rect) in &mut lines[index] {
                rect.x += shift.0;
                rect.y += shift.1;
            }
            for run in &mut runs[index] {
                run.baseline += shift.1;
                run.start += shift.0;
                for glyph in &mut run.glyphs {
                    glyph.x += shift.0;
                    glyph.y += shift.1;
                }
            }
            match item {
                InlineItem::Edge { .. } => {
                    for span in &mut spans {
                        span.rects.extend(lines[index].iter().map(|(_, r)| *r));
                    }
                }
                InlineItem::Text { path, style, .. } | InlineItem::Tab { path, style, .. } => {
                    for span in &mut spans {
                        span.rects.extend(lines[index].iter().map(|(_, r)| *r));
                    }
                    let item_runs = std::mem::take(&mut runs[index]);
                    match &mut pending {
                        Some(text) if text.path == *path => {
                            text.add(&lines[index]);
                            text.runs.extend(item_runs);
                        }
                        _ => {
                            self.flush(pending.take(), matrix);
                            let mut text = PendingText {
                                path: path.clone(),
                                style: style.clone(),
                                lines: Vec::new(),
                                runs: item_runs,
                            };
                            text.add(&lines[index]);
                            pending = Some(text);
                        }
                    }
                }
                InlineItem::SpanStart {
                    path,
                    style,
                    edges,
                    offset,
                } => {
                    self.flush(pending.take(), matrix);
                    spans.push(OpenSpan {
                        path: path.clone(),
                        style: style.clone(),
                        edges: *edges,
                        shift: (shift.0 + offset.0, shift.1 + offset.1),
                        rects: Vec::new(),
                    });
                }
                InlineItem::SpanEnd => {
                    self.flush(pending.take(), matrix);
                    if let Some(span) = spans.pop() {
                        let content = span
                            .rects
                            .iter()
                            .copied()
                            .reduce(Rect::union)
                            .unwrap_or_default();
                        let border_box = Rect {
                            y: content.y - to_f64(span.edges.top),
                            height: content.height + to_f64(span.edges.top + span.edges.bottom),
                            ..content
                        };
                        if let Some(outer) = spans.last_mut() {
                            outer.rects.extend(span.rects.iter().copied());
                        }
                        self.out.push(Placed {
                            drawn: Vec::new(),
                            path: span.path,
                            kind: PlacedKind::Element(Tag::Span),
                            border_box,
                            border: Sides::default(),
                            padding: Sides::default(),
                            matrix,
                            client: matrix.bounds(border_box),
                            style: None,
                            text: span.style,
                        });
                    }
                }
                InlineItem::Box { node, .. } => {
                    self.flush(pending.take(), matrix);
                    let Some((_, placed)) = boxes.iter().find(|(i, _)| *i == index).copied() else {
                        continue;
                    };
                    let rect = Rect {
                        x: placed.x + shift.0,
                        y: placed.y + shift.1,
                        ..placed
                    };
                    for span in &mut spans {
                        span.rects.push(rect);
                    }
                    perform(
                        self.tree,
                        *node,
                        narrow32(rect.width),
                        narrow32(rect.height),
                    );
                    self.tree.nodes[*node].layout.location = taffy::Point { x: 0.0, y: 0.0 };
                    self.tree.nodes[*node].layout.size = Size {
                        width: narrow32(rect.width),
                        height: narrow32(rect.height),
                    };
                    self.node(
                        *node,
                        &Frame {
                            origin: (rect.x, rect.y),
                            ..*frame
                        },
                    );
                }
            }
        }
        self.flush(pending, matrix);
    }

    fn flush(&mut self, text: Option<PendingText>, matrix: Matrix) {
        let Some(text) = text else {
            return;
        };
        let rects: Vec<Rect> = text.lines.into_iter().map(|(_, r)| r).collect();
        let border_box = rects
            .iter()
            .copied()
            .reduce(Rect::union)
            .unwrap_or_default();
        let client = rects
            .iter()
            .map(|r| matrix.bounds(*r))
            .reduce(Rect::union)
            .unwrap_or_default();
        self.out.push(Placed {
            drawn: Vec::new(),
            path: text.path,
            kind: PlacedKind::Text {
                lines: rects,
                runs: text.runs,
            },
            border_box,
            border: Sides::default(),
            padding: Sides::default(),
            matrix,
            client,
            style: None,
            text: text.style,
        });
    }
}

#[derive(Debug)]
struct OpenSpan {
    path: Vec<usize>,
    style: TextStyle,
    edges: SpanEdges,
    shift: (f64, f64),
    rects: Vec<Rect>,
}

#[derive(Debug)]
struct PendingText {
    path: Vec<usize>,
    style: TextStyle,
    lines: Vec<(usize, Rect)>,
    runs: Vec<TextRun>,
}

impl PendingText {
    fn add(&mut self, rects: &[(usize, Rect)]) {
        for (line, rect) in rects {
            match self.lines.iter_mut().find(|(l, _)| l == line) {
                Some((_, found)) => *found = found.union(*rect),
                None => self.lines.push((*line, *rect)),
            }
        }
    }
}

const fn narrow32(value: f64) -> f32 {
    opsis_types::numbers::narrow(value)
}
