use opsis_compile::color::{ColorFormat, Hsva, format_color, parse_color};
use opsis_compile::gradient::{
    GradientEdit, GradientKind, GradientView, edit_gradient, gradient_view,
};
use opsis_qt::{ColorView, GradientInfo, GradientStopView};

pub fn view(text: &str) -> ColorView {
    let format = ColorFormat::of(text).id().to_owned();
    parse_color(text).map_or_else(
        || ColorView {
            valid: false,
            h: 0.0,
            s: 0.0,
            v: 0.0,
            a: 1.0,
            format: format.clone(),
        },
        |c| ColorView {
            valid: true,
            h: c.h,
            s: c.s,
            v: c.v,
            a: c.a,
            format: format.clone(),
        },
    )
}

pub fn text(h: f64, s: f64, v: f64, a: f64, format: &str) -> String {
    let color = Hsva {
        h: h.rem_euclid(360.0),
        s: s.clamp(0.0, 1.0),
        v: v.clamp(0.0, 1.0),
        a: a.clamp(0.0, 1.0),
    };
    format_color(
        color,
        ColorFormat::from_id(format).unwrap_or(ColorFormat::Hex),
    )
}

fn info(view: Option<GradientView>) -> GradientInfo {
    view.map_or_else(
        || GradientInfo {
            valid: false,
            linear: true,
            angle: 0.0,
            stops: Vec::new(),
            css: String::new(),
            strip: String::new(),
            can_add: false,
            can_remove: false,
            selected: -1,
        },
        |v| GradientInfo {
            valid: true,
            linear: v.gradient.kind == GradientKind::Linear,
            angle: v.gradient.angle,
            stops: v
                .gradient
                .stops
                .iter()
                .map(|s| GradientStopView {
                    color: s.color.clone(),
                    at: s.at,
                })
                .collect(),
            css: v.css,
            strip: v.strip,
            can_add: v.can_add,
            can_remove: v.can_remove,
            selected: v.selected.and_then(|i| i32::try_from(i).ok()).unwrap_or(-1),
        },
    )
}

pub fn gradient(css: &str) -> GradientInfo {
    info(gradient_view(css))
}

pub fn edit(css: &str, edit_json: &str) -> GradientInfo {
    let edit: Option<GradientEdit> = serde_json::from_str(edit_json).ok();
    info(edit.and_then(|e| edit_gradient(css, &e)))
}
