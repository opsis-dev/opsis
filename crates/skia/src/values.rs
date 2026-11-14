pub use crate::bridge::ffi::{
    Axis, BlendMode, Channel, Color4 as Rgba, FontSpec, Gradient, GradientSpace, HueMethod,
    LightSource, Lighting, Matrix33, Matrix44, PointF, RectF, RoundRect, Sampling, StrokeCap,
    StrokeJoin, TileMode,
};

impl Rgba {
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

impl RectF {
    pub const fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }
}

impl PointF {
    pub const ZERO: Self = Self::new(0.0, 0.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl RoundRect {
    pub const fn new(rect: RectF, radii: [f32; 8]) -> Self {
        Self { rect, radii }
    }

    pub const fn square(rect: RectF) -> Self {
        Self {
            rect,
            radii: [0.0; 8],
        }
    }
}

impl Matrix33 {
    pub const IDENTITY: Self = Self {
        values: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    };

    pub const fn new(values: [f32; 9]) -> Self {
        Self { values }
    }
}

impl Matrix44 {
    pub const IDENTITY: Self = Self {
        values: [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
    };

    pub const fn row_major(values: [f32; 16]) -> Self {
        Self { values }
    }

    pub const fn translation(x: f32, y: f32) -> Self {
        Self {
            values: [
                1.0, 0.0, 0.0, x, 0.0, 1.0, 0.0, y, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }
}

impl FontSpec {
    pub const fn sized(size: f32) -> Self {
        Self {
            size,
            embolden: false,
            skew_x: 0.0,
        }
    }
}

impl Axis {
    pub const fn new(tag: [u8; 4], value: f32) -> Self {
        Self {
            tag: u32::from_be_bytes(tag),
            value,
        }
    }
}

impl Gradient {
    pub const fn new(space: GradientSpace) -> Self {
        Self {
            tile: TileMode::Clamp,
            space,
            hue: HueMethod::Shorter,
            premul: true,
            local: Matrix33::IDENTITY,
        }
    }
}
