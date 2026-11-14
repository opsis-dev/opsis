use opsis_scene::node::{Node, SvgElement, svg};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{Overflow, Position, Style};
use opsis_scene::values::Length;
use opsis_types::numbers::narrow;
use opsis_types::recording::HighlightLook;
use roughr::core::{Drawable, Op, OpType, Options, OptionsBuilder};
use roughr::generator::Generator;

const MAX_OFFSET: f32 = 5.0;
const BOWING: f32 = 1.0;
const CURVE_TIGHTNESS: f32 = 0.0;
const CURVE_FITTING: f32 = 0.95;
const CURVE_STEPS: f32 = 9.0;
const MARKER_ROUGHNESS: f64 = 3.0;
const LINE_ROUGHNESS: f64 = 1.5;
const MARK_SVG_SIZE: f64 = 100.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkShape {
    pub look: HighlightLook,
    pub seed: u32,
    pub strokes: u32,
    pub roughness: Option<f64>,
}

fn options(seed: u32, marker: bool, roughness: Option<f64>) -> Option<Options> {
    let roughness = roughness.unwrap_or(if marker {
        MARKER_ROUGHNESS
    } else {
        LINE_ROUGHNESS
    });
    OptionsBuilder::default()
        .max_randomness_offset(MAX_OFFSET)
        .roughness(narrow(roughness))
        .bowing(BOWING)
        .curve_tightness(CURVE_TIGHTNESS)
        .curve_fitting(CURVE_FITTING)
        .curve_step_count(CURVE_STEPS)
        .disable_multi_stroke(true)
        .preserve_vertices(false)
        .seed(u64::from(seed))
        .build()
        .ok()
}

type Point = (f64, f64);

fn back_and_forth(
    strokes: u32,
    (from, to): (Point, Point),
    each: &dyn Fn(u32) -> Option<Options>,
) -> Vec<Drawable<f64>> {
    let generator = Generator::default();
    (0..strokes)
        .map(|i| {
            let (start, end) = if i % 2 == 1 { (to, from) } else { (from, to) };
            generator.line(start.0, start.1, end.0, end.1, &each(i))
        })
        .collect()
}

fn drawables(shape: MarkShape, b: MarkBox) -> Vec<Drawable<f64>> {
    let generator = Generator::default();
    let stroke = |i: u32| options(shape.seed + i, false, shape.roughness);
    match shape.look {
        HighlightLook::Clean => Vec::new(),
        HighlightLook::Box => (0..shape.strokes)
            .map(|i| generator.rectangle(b.x, b.y, b.w, b.h, &stroke(i)))
            .collect(),
        HighlightLook::Circle => (0..shape.strokes)
            .map(|i| {
                generator.ellipse(
                    b.x + b.w / 2.0,
                    b.y + b.h / 2.0,
                    b.w / std::f64::consts::SQRT_2 * 2.0,
                    b.h / std::f64::consts::SQRT_2 * 2.0,
                    &stroke(i),
                )
            })
            .collect(),
        HighlightLook::Underline => back_and_forth(
            shape.strokes,
            ((b.x, b.y + b.h), (b.x + b.w, b.y + b.h)),
            &stroke,
        ),
        HighlightLook::Marker => back_and_forth(
            shape.strokes,
            ((b.x, b.y + b.h / 2.0), (b.x + b.w, b.y + b.h / 2.0)),
            &|i| options(shape.seed + i, true, shape.roughness),
        ),
    }
}

fn op_text(op: &Op<f64>) -> String {
    let n = |i: usize| js_number_text(op.data.get(i).copied().unwrap_or(0.0));
    match op.op {
        OpType::Move => format!("M{} {}", n(0), n(1)),
        OpType::LineTo => format!("L{} {}", n(0), n(1)),
        OpType::BCurveTo => format!("C{} {}, {} {}, {} {}", n(0), n(1), n(2), n(3), n(4), n(5)),
    }
}

fn subpaths(ops: &[Op<f64>]) -> Vec<String> {
    let mut runs: Vec<Vec<&Op<f64>>> = Vec::new();
    for op in ops {
        match runs.last_mut() {
            Some(run) if op.op != OpType::Move => run.push(op),
            _ => runs.push(vec![op]),
        }
    }
    runs.iter()
        .map(|run| {
            run.iter()
                .map(|op| op_text(op))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

pub fn mark_paths(shape: MarkShape, b: MarkBox) -> (Vec<String>, Option<f64>) {
    if b.w == 0.0 || b.h == 0.0 {
        return (Vec::new(), None);
    }
    let paths: Vec<String> = drawables(shape, b)
        .iter()
        .flat_map(|drawn| drawn.sets.iter())
        .flat_map(|set| subpaths(&set.ops))
        .collect();
    let width = (shape.look == HighlightLook::Marker).then_some(b.h);
    (paths, width)
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
pub fn stroke_shown(progress: f64, stroke: usize, strokes: usize) -> f64 {
    let raw = progress * opsis_types::numbers::float(strokes) - opsis_types::numbers::float(stroke);
    1.0f64.min(0.0f64.max(raw))
}

pub fn hand_drawn_mark(
    shape: MarkShape,
    b: MarkBox,
    progress: f64,
    color: &str,
    width: f64,
) -> Node {
    let (paths, own_width) = mark_paths(shape, b);
    let count = paths.len();
    let stroke_width = own_width.unwrap_or(width);
    let mut root = SvgElement::new("svg")
        .attr("width", MARK_SVG_SIZE)
        .attr("height", MARK_SVG_SIZE);
    for (i, d) in paths.into_iter().enumerate() {
        let shown = stroke_shown(progress, i, count);
        let mut path = SvgElement::new("path")
            .attr("d", d)
            .attr("fill", "none")
            .attr("stroke", color)
            .attr("stroke-width", stroke_width);
        if shown < 1.0 {
            path = path
                .attr("pathLength", "1")
                .attr("stroke-dasharray", "1 1")
                .attr("stroke-dashoffset", 1.0 - shown);
        }
        root = root.child(path);
    }
    svg(
        root,
        Style::default()
            .position(Position::Absolute)
            .top(Length::ZERO)
            .left(Length::ZERO)
            .width(Length::px(MARK_SVG_SIZE))
            .height(Length::px(MARK_SVG_SIZE))
            .overflow(Overflow::Visible),
    )
    .into()
}
