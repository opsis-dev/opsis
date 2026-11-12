use opsis_layout::computed::{Context, TextStyle, resolve};
use opsis_layout::output::Rect;
use opsis_layout::transform::Matrix;
use opsis_scene::values::{Corners, Length};
use opsis_skia::{Matrix44, PointF, RectF, RoundRect};
use opsis_types::numbers::narrow;

pub const fn rect_f(rect: Rect) -> RectF {
    RectF::new(
        narrow(rect.x),
        narrow(rect.y),
        narrow(rect.width),
        narrow(rect.height),
    )
}

pub const fn point(x: f64, y: f64) -> PointF {
    PointF::new(narrow(x), narrow(y))
}

pub fn matrix44(matrix: &Matrix) -> Matrix44 {
    let m = &matrix.0;
    let mut values = [0.0_f32; 16];
    for row in 0..4 {
        for column in 0..4 {
            values[row * 4 + column] = narrow(m[column * 4 + row]);
        }
    }
    Matrix44::row_major(values)
}

pub fn px(length: &Length, text: &TextStyle, context: &Context, basis: f64) -> f64 {
    resolve(length, &text.units(), context).px_or(basis, 0.0)
}

pub type Radii = [(f64, f64); 4];

pub fn radii(
    corners: Option<&Corners<(Length, Length)>>,
    rect: Rect,
    text: &TextStyle,
    context: &Context,
) -> Radii {
    let Some(corners) = corners else {
        return [(0.0, 0.0); 4];
    };
    let one = |(x, y): &(Length, Length)| {
        (
            px(x, text, context, rect.width).max(0.0),
            px(y, text, context, rect.height).max(0.0),
        )
    };
    let mut out = [
        one(&corners.top_left),
        one(&corners.top_right),
        one(&corners.bottom_right),
        one(&corners.bottom_left),
    ];
    let fits = |sum: f64, side: f64| if sum > side { side / sum } else { 1.0 };
    let scale = fits(out[0].0 + out[1].0, rect.width)
        .min(fits(out[3].0 + out[2].0, rect.width))
        .min(fits(out[0].1 + out[3].1, rect.height))
        .min(fits(out[1].1 + out[2].1, rect.height));
    if scale < 1.0 {
        for corner in &mut out {
            corner.0 *= scale;
            corner.1 *= scale;
        }
    }
    out
}

pub fn shrink(radii: &Radii, sides: [f64; 4]) -> Radii {
    let [top, right, bottom, left] = sides;
    [
        ((radii[0].0 - left).max(0.0), (radii[0].1 - top).max(0.0)),
        ((radii[1].0 - right).max(0.0), (radii[1].1 - top).max(0.0)),
        (
            (radii[2].0 - right).max(0.0),
            (radii[2].1 - bottom).max(0.0),
        ),
        ((radii[3].0 - left).max(0.0), (radii[3].1 - bottom).max(0.0)),
    ]
}

pub fn spread(radii: &Radii, by: f64) -> Radii {
    radii.map(|(x, y)| {
        let grow = |r: f64| {
            if r <= 0.0 { 0.0 } else { (r + by).max(0.0) }
        };
        (grow(x), grow(y))
    })
}

pub fn round_rect(rect: Rect, radii: &Radii) -> RoundRect {
    let mut values = [0.0_f32; 8];
    for (i, (x, y)) in radii.iter().enumerate() {
        values[i * 2] = narrow(*x);
        values[i * 2 + 1] = narrow(*y);
    }
    RoundRect::new(rect_f(rect), values)
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the colour matrices and gradient geometry follow the browser's arithmetic, one product and sum at a time, and the pictures have to match it"
)]
pub fn outset(rect: Rect, by: f64) -> Rect {
    Rect {
        x: rect.x - by,
        y: rect.y - by,
        width: (rect.width + 2.0 * by).max(0.0),
        height: (rect.height + 2.0 * by).max(0.0),
    }
}

pub fn moved(rect: Rect, dx: f64, dy: f64) -> Rect {
    Rect {
        x: rect.x + dx,
        y: rect.y + dy,
        ..rect
    }
}

pub fn is_rounded(radii: &Radii) -> bool {
    radii.iter().any(|(x, y)| *x > 0.0 && *y > 0.0)
}
