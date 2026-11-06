use opsis_compile::color::{ColorFormat, Hsva, format_color, parse_color};
use opsis_qt::ColorView;

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
