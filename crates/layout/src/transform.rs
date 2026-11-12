use opsis_scene::paint::Transform;
use opsis_scene::prop;
use opsis_scene::style::Style as SceneStyle;
use opsis_scene::values::{Angle, Length, Num, Position2};

use crate::computed::{Context, TextStyle, resolve};
use crate::output::Rect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix(pub [f64; 16]);

impl Default for Matrix {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Matrix {
    pub const IDENTITY: Self = Self([
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]);

    const fn at(&self, row: usize, column: usize) -> f64 {
        self.0[column * 4 + row]
    }

    #[must_use]
    pub fn then(&self, other: &Self) -> Self {
        let mut out = [0.0; 16];
        for row in 0..4 {
            for column in 0..4 {
                out[column * 4 + row] = (0..4).map(|k| self.at(row, k) * other.at(k, column)).sum();
            }
        }
        Self(out)
    }

    pub const fn translation(x: f64, y: f64, z: f64) -> Self {
        Self([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0,
        ])
    }

    pub const fn scaling(x: f64, y: f64, z: f64) -> Self {
        Self([
            x, 0.0, 0.0, 0.0, 0.0, y, 0.0, 0.0, 0.0, 0.0, z, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[expect(
        clippy::suboptimal_flops,
        reason = "the browser's matrix arithmetic rounds every step, and the boxes have to match it"
    )]
    pub fn rotation(axis_x: f64, axis_y: f64, axis_z: f64, degrees: f64) -> Self {
        let length = (axis_x * axis_x + axis_y * axis_y + axis_z * axis_z).sqrt();
        if length <= f64::EPSILON {
            return Self::IDENTITY;
        }
        let (ux, uy, uz) = (axis_x / length, axis_y / length, axis_z / length);
        let (sine, cosine) = degrees.to_radians().sin_cos();
        let rest = 1.0 - cosine;
        Self([
            rest * ux * ux + cosine,
            rest * ux * uy + sine * uz,
            rest * ux * uz - sine * uy,
            0.0,
            rest * ux * uy - sine * uz,
            rest * uy * uy + cosine,
            rest * uy * uz + sine * ux,
            0.0,
            rest * ux * uz + sine * uy,
            rest * uy * uz - sine * ux,
            rest * uz * uz + cosine,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }

    pub fn skew(x_degrees: f64, y_degrees: f64) -> Self {
        let (tx, ty) = (x_degrees.to_radians().tan(), y_degrees.to_radians().tan());
        Self([
            1.0, ty, 0.0, 0.0, tx, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    pub fn perspective(distance: f64) -> Self {
        let mut out = Self::IDENTITY;
        if distance > 0.0 {
            out.0[11] = -1.0 / distance;
        }
        out
    }

    #[expect(
        clippy::suboptimal_flops,
        reason = "the browser's matrix arithmetic rounds every step, and the boxes have to match it"
    )]
    pub fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        let px = self.at(0, 0) * x + self.at(0, 1) * y + self.at(0, 3);
        let py = self.at(1, 0) * x + self.at(1, 1) * y + self.at(1, 3);
        let pw = self.at(3, 0) * x + self.at(3, 1) * y + self.at(3, 3);
        if pw.abs() <= f64::EPSILON {
            return (px, py);
        }
        (px / pw, py / pw)
    }

    pub fn faces_back(&self, rect: Rect) -> bool {
        let origin = self.apply(rect.x, rect.y);
        let across = self.apply(rect.x + 1.0, rect.y);
        let down = self.apply(rect.x, rect.y + 1.0);
        let (ax, ay) = (across.0 - origin.0, across.1 - origin.1);
        let (dx, dy) = (down.0 - origin.0, down.1 - origin.1);
        ax.mul_add(dy, -(ay * dx)) < 0.0
    }

    pub fn bounds(&self, rect: Rect) -> Rect {
        let corners = [
            self.apply(rect.x, rect.y),
            self.apply(rect.x + rect.width, rect.y),
            self.apply(rect.x, rect.y + rect.height),
            self.apply(rect.x + rect.width, rect.y + rect.height),
        ];
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for (x, y) in corners {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
        Rect {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

struct Resolver<'a> {
    text: &'a TextStyle,
    context: &'a Context,
}

impl Resolver<'_> {
    fn px(&self, length: &Length, basis: f64) -> f64 {
        resolve(length, &self.text.units(), self.context).px_or(basis, 0.0)
    }
}

fn degrees(angle: &Angle) -> f64 {
    angle.degrees()
}

fn function(transform: &Transform, resolver: &Resolver<'_>, rect: &Rect) -> Matrix {
    match transform {
        Transform::Translate(x, y, z) => Matrix::translation(
            resolver.px(x, rect.width),
            resolver.px(y, rect.height),
            resolver.px(z, 0.0),
        ),
        Transform::Scale(Num(x), Num(y), Num(z)) => Matrix::scaling(*x, *y, *z),
        Transform::Rotate(angle) => Matrix::rotation(0.0, 0.0, 1.0, degrees(angle)),
        Transform::Rotate3d(Num(x), Num(y), Num(z), angle) => {
            Matrix::rotation(*x, *y, *z, degrees(angle))
        }
        Transform::RotateX(angle) => Matrix::rotation(1.0, 0.0, 0.0, degrees(angle)),
        Transform::RotateY(angle) => Matrix::rotation(0.0, 1.0, 0.0, degrees(angle)),
        Transform::Skew(x, y) => Matrix::skew(degrees(x), degrees(y)),
        Transform::Matrix(m) => Matrix([
            m[0].0, m[1].0, 0.0, 0.0, m[2].0, m[3].0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, m[4].0, m[5].0,
            0.0, 1.0,
        ]),
        Transform::Matrix3d(m) => Matrix(m.map(|n| n.0)),
        Transform::Perspective(distance) => {
            distance.map_or(Matrix::IDENTITY, |d| Matrix::perspective(d.0))
        }
    }
}

fn origin(style: &SceneStyle, resolver: &Resolver<'_>, rect: &Rect) -> (f64, f64, f64) {
    prop!(style, TransformOrigin).map_or((rect.width / 2.0, rect.height / 2.0, 0.0), |(x, y, z)| {
        (
            resolver.px(x, rect.width),
            resolver.px(y, rect.height),
            resolver.px(z, 0.0),
        )
    })
}

pub fn has_transform(style: &SceneStyle) -> bool {
    prop!(style, Transform).is_some_and(|t| !t.is_empty())
        || prop!(style, Translate).is_some()
        || prop!(style, Rotate).is_some()
        || prop!(style, Scale).is_some()
}

pub fn local(style: &SceneStyle, text: &TextStyle, context: &Context, rect: &Rect) -> Matrix {
    if !has_transform(style) {
        return Matrix::IDENTITY;
    }
    let resolver = Resolver { text, context };
    let (ox, oy, oz) = origin(style, &resolver, rect);
    let mut matrix = Matrix::translation(rect.x + ox, rect.y + oy, oz);
    if let Some((x, y, z)) = prop!(style, Translate) {
        matrix = matrix.then(&Matrix::translation(
            resolver.px(x, rect.width),
            resolver.px(y, rect.height),
            resolver.px(z, 0.0),
        ));
    }
    if let Some((Num(x), Num(y), Num(z), angle)) = prop!(style, Rotate) {
        matrix = matrix.then(&Matrix::rotation(*x, *y, *z, degrees(angle)));
    }
    if let Some((Num(x), Num(y), Num(z))) = prop!(style, Scale) {
        matrix = matrix.then(&Matrix::scaling(*x, *y, *z));
    }
    for item in prop!(style, Transform).into_iter().flatten() {
        matrix = matrix.then(&function(item, &resolver, rect));
    }
    matrix.then(&Matrix::translation(-(rect.x + ox), -(rect.y + oy), -oz))
}

pub fn perspective_of(
    style: &SceneStyle,
    text: &TextStyle,
    context: &Context,
    rect: &Rect,
) -> Option<Matrix> {
    let distance = prop!(style, Perspective)?.0;
    let resolver = Resolver { text, context };
    let center = Position2::center();
    let at = prop!(style, PerspectiveOrigin).unwrap_or(&center);
    let (ox, oy) = (
        rect.x + resolver.px(&at.x, rect.width),
        rect.y + resolver.px(&at.y, rect.height),
    );
    Some(
        Matrix::translation(ox, oy, 0.0)
            .then(&Matrix::perspective(distance))
            .then(&Matrix::translation(-ox, -oy, 0.0)),
    )
}
