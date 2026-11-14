use opsis_scene::clock::js_round;
use opsis_scene::color::{Color, ColorSpace, Interpolation};
use opsis_scene::drawing::Font;
use opsis_scene::measure::Measure;
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::paint::{BlendMode, Filter, Transform};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{
    BorderSide, BorderStyle, BoxSizing, DecorationLine, DecorationStyle, Display, FontStyle,
    LineHeight, Overflow, Position, Style, Tag, TextAlign, TextDecoration, WhiteSpace,
};
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::numbers::float;
use opsis_types::terminal::{TerminalCursor, TerminalLook};
use opsis_types::terminal_frames::{FrameRecord, Frames, Redaction, Run, flag};

use crate::elements::draw::css_color;
use crate::elements::terminal::sprites::{Cell, Shape, sprite_for, stroke_for};
use crate::terminal::TERMINAL_FONTS;

const CELL_WIDTH_EM: f64 = 0.6;
const BLUR_SHARE: f64 = 0.35;
const PIXEL_SHADES: [f64; 4] = [0.55, 0.7, 0.45, 0.62];
const PIXELS_PER_CELL: f64 = 2.0;
const PIXEL_ROWS: u32 = 2;
const WHITE: u32 = 0x00ff_ffff;
const BOLD_WEIGHT: f64 = 700.0;
const REGULAR_WEIGHT: f64 = 400.0;
const LIGATURES_OFF: [&[u8; 4]; 4] = [b"kern", b"liga", b"clig", b"calt"];

const CURSOR_BAR: u8 = 0;
const CURSOR_BLOCK: u8 = 1;
const CURSOR_UNDERLINE: u8 = 2;
const CURSOR_HOLLOW: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellBox {
    pub width: f64,
    pub height: f64,
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
pub fn cell_box(look: &TerminalLook) -> CellBox {
    let size = f64::from(look.font_size);
    CellBox {
        width: size * CELL_WIDTH_EM + look.letter_spacing,
        height: js_round(size * look.line_height),
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
pub fn screen_size(look: &TerminalLook, cols: u16, rows: u16) -> CellBox {
    let cell = cell_box(look);
    let padding = f64::from(look.padding);
    CellBox {
        width: js_round(f64::from(cols) * cell.width + 2.0 * padding),
        height: js_round(f64::from(rows) * cell.height + 2.0 * padding),
    }
}

pub fn hex_colour(colour: u32) -> String {
    format!("#{colour:06x}")
}

fn decoration(run: &Run) -> Option<TextDecoration> {
    let mut lines = Vec::new();
    if run.underline != 0 {
        lines.push(DecorationLine::Underline);
    }
    if run.flags & flag::STRIKETHROUGH != 0 {
        lines.push(DecorationLine::LineThrough);
    }
    if run.flags & flag::OVERLINE != 0 {
        lines.push(DecorationLine::Overline);
    }
    if lines.is_empty() {
        return None;
    }
    let style = match run.underline {
        2 => DecorationStyle::Double,
        3 => DecorationStyle::Wavy,
        4 => DecorationStyle::Dotted,
        5 => DecorationStyle::Dashed,
        _ => DecorationStyle::Solid,
    };
    Some(TextDecoration {
        lines,
        style,
        color: Some(css_color(&hex_colour(run.underline_colour))),
        thickness: None,
    })
}

fn single_char(text: &str) -> Option<char> {
    let mut chars = text.chars();
    let first = chars.next()?;
    chars.next().is_none().then_some(first)
}

fn sprite(run: &Run, cell: CellBox, thickness: f64) -> Option<Vec<Shape>> {
    let ch = single_char(&run.text)?;
    sprite_for(
        u32::from(ch),
        Cell {
            width: cell.width * f64::from(run.cells),
            height: cell.height,
            thickness,
        },
    )
}

#[derive(Debug, Clone)]
struct Placed {
    col: u16,
    cells: u16,
    fg: u32,
    y: f64,
    shapes: Vec<Shape>,
}

fn rect_d(x: f64, y: f64, width: f64, height: f64) -> String {
    let n = js_number_text;
    format!(
        "M{} {}h{}v{}h{}Z",
        n(x),
        n(y),
        n(width),
        n(height),
        n(-width)
    )
}

fn shape_d(shape: &Shape, dx: f64, dy: f64) -> String {
    match shape {
        Shape::Rect {
            x,
            y,
            width,
            height,
            ..
        } => rect_d(x + dx, y + dy, *width, *height),
        Shape::Inverted(_) => String::new(),
        Shape::Fill { path, .. } | Shape::Stroke { path, .. } => path.moved(dx, dy).d(),
    }
}

fn fill_opacity(shape: &Shape) -> f64 {
    match shape {
        Shape::Rect { opacity, .. } => *opacity,
        Shape::Fill { opacity, .. } => opacity.unwrap_or(1.0),
        _ => 1.0,
    }
}

const fn boxed_shape(shape: &Shape) -> bool {
    match shape {
        Shape::Inverted(_) => true,
        Shape::Rect { .. } => false,
        Shape::Fill { clip, .. } => *clip,
        Shape::Stroke { clip, inner, .. } => *clip || *inner,
    }
}

#[derive(Debug, Clone, Copy)]
struct Spot {
    dx: f64,
    dy: f64,
}

fn shape_element(shape: &Shape, at: Spot, colour: &str, inside: Option<&str>) -> Vec<SvgElement> {
    let d = shape_d(shape, at.dx, at.dy);
    match shape {
        Shape::Stroke { width, inner, .. } if *inner && inside.is_some() => {
            let id = inside.unwrap_or_default();
            vec![
                SvgElement::new("clipPath")
                    .attr("id", id)
                    .child(SvgElement::new("path").attr("d", d.clone())),
                SvgElement::new("path")
                    .attr("d", d)
                    .attr("fill", "none")
                    .attr("stroke", colour)
                    .attr("stroke-width", width * 2.0)
                    .attr("clip-path", format!("url(#{id})")),
            ]
        }
        Shape::Stroke { width, .. } => vec![
            SvgElement::new("path")
                .attr("d", d)
                .attr("fill", "none")
                .attr("stroke", colour)
                .attr("stroke-width", *width),
        ],
        Shape::Inverted(_) => Vec::new(),
        _ => vec![
            SvgElement::new("path")
                .attr("d", d)
                .attr("fill", colour)
                .attr("fill-opacity", fill_opacity(shape)),
        ],
    }
}

#[derive(Debug, Clone)]
struct Boxed {
    shapes: Vec<Shape>,
    at: Spot,
    width: f64,
    height: f64,
    colour: String,
}

fn cell_rect(b: &Boxed) -> SvgElement {
    SvgElement::new("rect")
        .attr("x", b.at.dx)
        .attr("y", b.at.dy)
        .attr("width", b.width)
        .attr("height", b.height)
}

fn boxed_shapes(id: &str, b: &Boxed) -> Vec<SvgElement> {
    let inverted: Vec<&Shape> = b
        .shapes
        .iter()
        .flat_map(|s| match s {
            Shape::Inverted(inner) => inner.iter().collect(),
            _ => Vec::new(),
        })
        .collect();
    let clipped: Vec<&Shape> = b
        .shapes
        .iter()
        .filter(|s| !matches!(s, Shape::Inverted(_)))
        .collect();
    let mut out = vec![
        SvgElement::new("clipPath")
            .attr("id", format!("{id}-clip"))
            .child(cell_rect(b)),
    ];
    if !clipped.is_empty() {
        let mut group = SvgElement::new("g").attr("clip-path", format!("url(#{id}-clip)"));
        for (i, shape) in clipped.iter().enumerate() {
            let inside = format!("{id}-inner-{i}");
            for element in shape_element(shape, b.at, &b.colour, Some(&inside)) {
                group = group.child(element);
            }
        }
        out.push(group);
    }
    if !inverted.is_empty() {
        let mut mask = SvgElement::new("mask")
            .attr("id", format!("{id}-mask"))
            .attr("maskUnits", "userSpaceOnUse")
            .attr("x", b.at.dx)
            .attr("y", b.at.dy)
            .attr("width", b.width)
            .attr("height", b.height)
            .child(cell_rect(b).attr("fill", hex_colour(WHITE)));
        for shape in inverted {
            for element in shape_element(shape, b.at, &hex_colour(0), None) {
                mask = mask.child(element);
            }
        }
        out.push(mask);
        out.push(
            cell_rect(b)
                .attr("fill", b.colour.as_str())
                .attr("mask", format!("url(#{id}-mask)")),
        );
    }
    out
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
fn sprite_layer(placed: &[Placed], look: &TerminalLook, size: CellBox, id: &str) -> Node {
    let cell = cell_box(look);
    let padding = f64::from(look.padding);
    let mut fills: Vec<(String, f64, String)> = Vec::new();
    let mut strokes: Vec<(String, String, f64)> = Vec::new();
    let mut boxed: Vec<Boxed> = Vec::new();
    for p in placed {
        let at = Spot {
            dx: padding + f64::from(p.col) * cell.width,
            dy: padding + p.y * cell.height,
        };
        let colour = hex_colour(p.fg);
        let own: Vec<Shape> = p
            .shapes
            .iter()
            .filter(|s| boxed_shape(s))
            .cloned()
            .collect();
        if !own.is_empty() {
            boxed.push(Boxed {
                shapes: own,
                at,
                width: cell.width * f64::from(p.cells),
                height: cell.height,
                colour: colour.clone(),
            });
        }
        for shape in &p.shapes {
            if boxed_shape(shape) {
                continue;
            }
            match shape {
                Shape::Stroke { path, width, .. } => {
                    strokes.push((path.moved(at.dx, at.dy).d(), colour.clone(), *width));
                }
                Shape::Inverted(_) => {}
                _ => {
                    let opacity = fill_opacity(shape);
                    let d = shape_d(shape, at.dx, at.dy);
                    let key = js_number_text(opacity);
                    match fills
                        .iter_mut()
                        .find(|(c, o, _)| *c == colour && js_number_text(*o) == key)
                    {
                        Some(entry) => entry.2.push_str(&d),
                        None => fills.push((colour.clone(), opacity, d)),
                    }
                }
            }
        }
    }
    let mut root = SvgElement::new("svg")
        .attr("width", size.width)
        .attr("height", size.height)
        .attr(
            "viewBox",
            format!(
                "0 0 {} {}",
                js_number_text(size.width),
                js_number_text(size.height)
            ),
        );
    for (colour, opacity, d) in fills {
        root = root.child(
            SvgElement::new("path")
                .attr("d", d)
                .attr("fill", colour)
                .attr("fill-opacity", opacity),
        );
    }
    for (d, colour, width) in strokes {
        root = root.child(
            SvgElement::new("path")
                .attr("d", d)
                .attr("fill", "none")
                .attr("stroke", colour)
                .attr("stroke-width", width),
        );
    }
    for (i, b) in boxed.iter().enumerate() {
        for element in boxed_shapes(&format!("{id}-{i}"), b) {
            root = root.child(element);
        }
    }
    svg(
        root,
        Style::default()
            .position(Position::Absolute)
            .left(Length::ZERO)
            .top(Length::ZERO)
            .width(Length::px(size.width))
            .height(Length::px(size.height)),
    )
    .into()
}

fn absolute_box(left: f64, top: f64, width: f64, height: f64) -> Style {
    Style::default()
        .position(Position::Absolute)
        .left(Length::px(left))
        .top(Length::px(top))
        .width(Length::px(width))
        .height(Length::px(height))
}

fn masked(run: &Run, cell: CellBox, bg: &str) -> Vec<Node> {
    let fill = Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO);
    match run.redaction {
        Redaction::Solid => {
            vec![div(fill.background_color(css_color(&hex_colour(run.fg)))).into()]
        }
        Redaction::Blur => vec![
            div(fill
                .background_color(Color::mix(
                    Interpolation {
                        space: ColorSpace::Oklch,
                        ..Interpolation::DEFAULT
                    },
                    (css_color(bg), None),
                    (Color::CurrentColor, Some(BLUR_SHARE)),
                ))
                .filter(vec![Filter::Blur(Length::px(cell.height / 4.0))]))
            .into(),
        ],
        _ => {
            let side = cell.height / PIXELS_PER_CELL;
            let across = opsis_types::numbers::whole((cell.width / side).ceil());
            let mut blocks = Vec::new();
            for y in 0..PIXEL_ROWS {
                for x in 0..across {
                    let index = usize::try_from(x + y * 3 + u32::from(run.col)).unwrap_or(0);
                    let shade = PIXEL_SHADES[index % PIXEL_SHADES.len()];
                    blocks.push(
                        div(
                            absolute_box(f64::from(x) * side, f64::from(y) * side, side, side)
                                .background_color(css_color(bg))
                                .filter(vec![Filter::Brightness(Num(shade))]),
                        )
                        .into(),
                    );
                }
            }
            blocks
        }
    }
}

fn terminal_font(size: f64, bold: bool) -> Font {
    Font {
        families: TERMINAL_FONTS.iter().map(|f| (*f).to_owned()).collect(),
        size: Num(size),
        weight: Num(if bold { BOLD_WEIGHT } else { REGULAR_WEIGHT }),
        italic: false,
    }
}

fn fitted(run: &Run, width: f64, size: f64, measure: &dyn Measure) -> Option<Vec<Transform>> {
    single_char(&run.text)?;
    let measured = measure.text_width(
        &run.text,
        &terminal_font(size, run.flags & flag::BOLD != 0),
        0.0,
    );
    if measured <= 0.0 || measured <= width {
        return None;
    }
    Some(vec![Transform::Scale(
        Num(width / measured),
        Num(1.0),
        Num(1.0),
    )])
}

#[derive(Clone, Copy)]
struct RunAt<'a> {
    y: f64,
    screen_bg: u32,
    thickness: f64,
    look: &'a TerminalLook,
    measure: &'a dyn Measure,
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
fn run_view(run: &Run, at: &RunAt<'_>) -> Option<Node> {
    let cell = cell_box(at.look);
    let padding = f64::from(at.look.padding);
    let width = cell.width * f64::from(run.cells);
    let mut place = absolute_box(
        padding + f64::from(run.col) * cell.width,
        padding + at.y * cell.height,
        width,
        cell.height,
    );
    if run.bg != at.screen_bg {
        place = place.background_color(css_color(&hex_colour(run.bg)));
    }
    if run.redaction != Redaction::None {
        let bg = hex_colour(run.bg);
        return Some(
            div(place
                .color(css_color(&hex_colour(run.fg)))
                .overflow(Overflow::Hidden))
            .children(masked(
                run,
                CellBox {
                    width,
                    height: cell.height,
                },
                &bg,
            ))
            .into(),
        );
    }
    if sprite(run, cell, at.thickness).is_some() {
        return (run.bg != at.screen_bg).then(|| div(place).into());
    }
    let single = single_char(&run.text).is_some();
    let mut style = place
        .color(css_color(&hex_colour(run.fg)))
        .white_space(WhiteSpace::Pre)
        .line_height(LineHeight::Length(Length::px(cell.height)))
        .letter_spacing(Length::px(at.look.letter_spacing))
        .font_weight(Num(if run.flags & flag::BOLD != 0 {
            BOLD_WEIGHT
        } else {
            REGULAR_WEIGHT
        }))
        .font_style(if run.flags & flag::ITALIC != 0 {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        })
        .text_align(if single {
            TextAlign::Center
        } else {
            TextAlign::Left
        });
    if let Some(lines) = decoration(run) {
        style = style.text_decoration(lines);
    }
    let mut inner = Style::default().display(Display::InlineBlock);
    if let Some(scale) = fitted(run, width, f64::from(at.look.font_size), at.measure) {
        inner = inner.transform(scale);
    }
    Some(div(style).child(span(inner).child(run.text.clone())).into())
}

const fn cursor_shape(look: &TerminalLook, frame: &FrameRecord) -> u8 {
    match look.cursor {
        TerminalCursor::Block => CURSOR_BLOCK,
        TerminalCursor::Bar => CURSOR_BAR,
        TerminalCursor::Underline => CURSOR_UNDERLINE,
        TerminalCursor::Program => frame.cursor.shape,
    }
}

fn run_at(runs: &[Run], x: u16) -> Option<&Run> {
    runs.iter()
        .find(|r| x >= r.col && u32::from(x) < u32::from(r.col) + u32::from(r.cells))
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
fn cursor(frame: &FrameRecord, runs: &[Run], look: &TerminalLook, thickness: f64) -> Node {
    let cell = cell_box(look);
    let padding = f64::from(look.padding);
    let c = frame.cursor;
    let under = run_at(runs, c.x);
    let wide = under.filter(|r| r.col == c.x).map_or(1, |r| r.cells);
    let left = padding + f64::from(c.x) * cell.width;
    let top = padding + f64::from(c.y) * cell.height;
    let width = cell.width * f64::from(wide);
    let paint = css_color(&hex_colour(c.colour));
    match cursor_shape(look, frame) {
        CURSOR_BAR => {
            div(absolute_box(left, top, thickness * 2.0, cell.height).background_color(paint))
                .into()
        }
        CURSOR_UNDERLINE => div(absolute_box(
            left,
            top + cell.height - thickness * 2.0,
            width,
            thickness * 2.0,
        )
        .background_color(paint))
        .into(),
        CURSOR_HOLLOW => div(absolute_box(left, top, width, cell.height)
            .box_sizing(BoxSizing::BorderBox)
            .border(Edges::all(BorderSide {
                width: Length::px(thickness),
                style: BorderStyle::Solid,
                color: Some(paint),
            })))
        .into(),
        _ => {
            let text: String = under
                .and_then(|r| r.text.chars().nth(usize::from(c.x.saturating_sub(r.col))))
                .map_or_else(String::new, String::from);
            let bold = under.is_some_and(|r| r.flags & flag::BOLD != 0);
            let mut block = div(absolute_box(left, top, width, cell.height)
                .background_color(paint)
                .color(css_color(&hex_colour(frame.bg)))
                .white_space(WhiteSpace::Pre)
                .line_height(LineHeight::Length(Length::px(cell.height)))
                .text_align(TextAlign::Center)
                .font_weight(Num(if bold { BOLD_WEIGHT } else { REGULAR_WEIGHT })));
            if !text.trim().is_empty() {
                block = block.child(text);
            }
            block.into()
        }
    }
}

#[derive(Clone, Copy)]
pub struct View<'a> {
    pub frames: &'a Frames,
    pub frame: &'a FrameRecord,
    pub look: &'a TerminalLook,
    pub cursor_on: bool,
    pub measure: &'a dyn Measure,
    pub id: &'a str,
}

impl std::fmt::Debug for View<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("View")
            .field("frame", &self.frame)
            .field("look", &self.look)
            .field("cursor_on", &self.cursor_on)
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

fn rows_of<'a>(
    frames: &'a Frames,
    frame: &'a FrameRecord,
) -> impl Iterator<Item = (f64, &'a [Run])> {
    frame.row_indices.iter().enumerate().map(|(y, index)| {
        let runs = usize::try_from(*index)
            .ok()
            .and_then(|i| frames.row_table.get(i))
            .map_or(&[][..], Vec::as_slice);
        (float(y), runs)
    })
}

pub fn terminal_view(view: &View<'_>) -> Node {
    let size = screen_size(view.look, view.frame.cols, view.frame.rows);
    let thickness = stroke_for(f64::from(view.look.font_size));
    let cell = cell_box(view.look);
    let cursor_runs = view
        .frame
        .row_indices
        .get(usize::from(view.frame.cursor.y))
        .and_then(|i| usize::try_from(*i).ok())
        .and_then(|i| view.frames.row_table.get(i))
        .map_or(&[][..], Vec::as_slice);
    let mut placed = Vec::new();
    let mut children: Vec<Node> = Vec::new();
    let at = |y: f64| RunAt {
        y,
        screen_bg: view.frame.bg,
        thickness,
        look: view.look,
        measure: view.measure,
    };
    for (y, runs) in rows_of(view.frames, view.frame) {
        for run in runs {
            if let Some(shapes) = sprite(run, cell, thickness) {
                placed.push(Placed {
                    col: run.col,
                    cells: run.cells,
                    fg: run.fg,
                    y,
                    shapes,
                });
            }
            children.extend(run_view(run, &at(y)));
        }
    }
    children.push(sprite_layer(&placed, view.look, size, view.id));
    if view.frame.cursor.visible && view.cursor_on {
        children.push(cursor(view.frame, cursor_runs, view.look, thickness));
    }
    div(Style::default()
        .position(Position::Relative)
        .width(Length::px(size.width))
        .height(Length::px(size.height))
        .overflow(Overflow::Hidden)
        .background_color(css_color(&hex_colour(view.frame.bg)))
        .font_family(
            TERMINAL_FONTS
                .iter()
                .map(|f| (*f).to_owned())
                .collect::<Vec<_>>(),
        )
        .font_size(Length::px(f64::from(view.look.font_size)))
        .font_feature_settings(
            LIGATURES_OFF
                .iter()
                .map(|t| (Tag(**t), 0))
                .collect::<Vec<_>>(),
        ))
    .children(children)
    .into()
}

pub fn lighten_layer(opacity: f64, child: Node) -> Node {
    div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .opacity(opacity)
        .mix_blend_mode(BlendMode::Lighten))
    .child(child)
    .into()
}
