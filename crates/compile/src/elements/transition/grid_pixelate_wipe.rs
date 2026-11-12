use opsis_macros::{choice, settings};
use opsis_scene::easing::Easing;
use opsis_scene::node::Node;
use opsis_scene::paint::{AssetRef, Image, ImageLayer, LayerSize, MaskLayer, MaskMode, Repeat};
use opsis_scene::random::js_number_text;
use opsis_scene::style::Style;
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;
use opsis_types::numbers::whole;

use crate::elements::anim::clamped_eased;
use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::transition::present::{encoded_svg, filled, transition_phase};
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum PixelateOrder {
    FromLeft,
    FromTop,
    Diagonal,
    Center,
}

#[choice(export)]
pub enum PixelShape {
    Square,
    Dot,
}

#[settings(export)]
pub struct GridPixelateWipe {
    #[setting(
        "Columns",
        default = 12,
        range(min = 1, max = 48),
        step = 1,
        help = "Grid column count."
    )]
    pub cols: u32,
    #[setting(
        "Rows",
        default = 8,
        range(min = 1, max = 48),
        step = 1,
        help = "Grid row count."
    )]
    pub rows: u32,
    #[setting(
        "Order",
        default = PixelateOrder::FromLeft,
        help = "Which axis or point the cells light up from."
    )]
    pub order: PixelateOrder,
    #[setting(
        "Shape",
        default = PixelShape::Square,
        help = "Cells fill as blocks, or as points that grow with their own reveal."
    )]
    pub shape: PixelShape,
    #[setting(
        "Stagger",
        default = 0.82,
        range(min = 0, max = 1),
        step = 0.01,
        help = "0 pops every cell together; 1 spreads them across the whole window.",
        advanced
    )]
    pub stagger: f64,
    #[setting("Reverse order", advanced)]
    pub reverse: bool,
}

pub fn definition() -> ElementMeta {
    meta::<GridPixelateWipe>(
        element(
            ElementKind::Presentation,
            "grid-pixelate-wipe",
            "Pixel grid wipe",
            "The next scene appears one grid cell at a time.",
            ElementCategory::Transition,
            26,
        )
        .look(NodeLook::of("lucide:grid-2x2")),
    )
}

const MIN_OPACITY: f64 = 0.004;

fn cell_rank(order: PixelateOrder, col: f64, row: f64, cols: f64, rows: f64) -> f64 {
    let u = if cols <= 1.0 { 0.0 } else { col / (cols - 1.0) };
    let v = if rows <= 1.0 { 0.0 } else { row / (rows - 1.0) };
    match order {
        PixelateOrder::FromLeft => u,
        PixelateOrder::FromTop => v,
        PixelateOrder::Diagonal => f64::midpoint(u, v),
        PixelateOrder::Center => ((u - 0.5).hypot(v - 0.5) / std::f64::consts::FRAC_1_SQRT_2).min(1.0),
    }
}

fn cell_opacity(progress: f64, rank: f64, stagger: f64) -> Result<f64, DrawFailure> {
    let reveal = rank * stagger;
    let span = (1.0 - stagger).max(0.001);
    let raw = ((progress - reveal) / span).clamp(0.0, 1.0);
    clamped_eased(raw, [0.0, 1.0], [0.0, 1.0], Easing::ease_out(Easing::Cubic))
}

fn grid_mask(props: &GridPixelateWipe, progress: f64, rows: u32, aspect: f64) -> Result<String, DrawFailure> {
    let (cols, rows_f) = (f64::from(props.cols), f64::from(rows));
    let (cell_w, cell_h) = (100.0 / cols, 100.0 / rows_f);
    let n = js_number_text;
    let mut cells = String::new();
    for row in 0..rows {
        for col in 0..props.cols {
            let (c, r) = (f64::from(col), f64::from(row));
            let rank = cell_rank(props.order, c, r, cols, rows_f);
            let opacity = cell_opacity(progress, if props.reverse { 1.0 - rank } else { rank }, props.stagger)?;
            if opacity < MIN_OPACITY {
                continue;
            }
            let alpha = format!("{opacity:.3}");
            if props.shape == PixelShape::Dot {
                let radius = (cell_w * aspect).hypot(cell_h) / 2.0 * opacity;
                cells.push_str(&format!(
                    "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{:.3}\" ry=\"{radius:.3}\" fill=\"white\" fill-opacity=\"{alpha}\"/>",
                    n((c + 0.5) * cell_w),
                    n((r + 0.5) * cell_h),
                    radius / aspect
                ));
                continue;
            }
            cells.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"white\" fill-opacity=\"{alpha}\"/>",
                n(c * cell_w),
                n(r * cell_h),
                n(cell_w),
                n(cell_h)
            ));
        }
    }
    Ok(encoded_svg(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 100\" preserveAspectRatio=\"none\">{cells}</svg>"
    )))
}

pub fn present(props: &GridPixelateWipe, side: Transition, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let phase = transition_phase(side.progress, side.entering, 1.0, false)?;
    let filled_share = 1.0 - phase.displace;
    if !phase.entering || filled_share >= 1.0 {
        return Ok(filled(Style::default(), side.children));
    }
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let rows = whole(f64::from(props.rows) * (height / width) * 16.0 / 9.0).max(1);
    let mut layer = ImageLayer::of(Image::Url(AssetRef(grid_mask(props, filled_share, rows, width / height)?)));
    layer.size = LayerSize::Explicit(Length::percent(100.0), Length::percent(100.0));
    layer.repeat = (Repeat::NoRepeat, Repeat::NoRepeat);
    Ok(filled(
        Style::default().mask(vec![MaskLayer {
            layer,
            mode: MaskMode::Alpha,
        }]),
        side.children,
    ))
}
