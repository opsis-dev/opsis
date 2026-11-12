use opsis_scene::timeline::Stage;

const DESIGN_WIDTH: f64 = 1920.0;
const LANDSCAPE_ASPECT: f64 = 1.3;
const SQUARISH_ASPECT: f64 = 0.7;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Insets {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

const SQUARISH_INSETS: Insets = Insets {
    top: 0.03,
    right: 0.03,
    bottom: 0.05,
    left: 0.03,
};
const TALL_INSETS: Insets = Insets {
    top: 0.1,
    right: 0.05,
    bottom: 0.16,
    left: 0.05,
};

pub fn is_landscape(width: f64, height: f64) -> bool {
    width / height >= LANDSCAPE_ASPECT
}

pub fn shape_insets(width: f64, height: f64) -> Insets {
    if is_landscape(width, height) {
        return Insets::default();
    }
    let f = if width / height >= SQUARISH_ASPECT {
        SQUARISH_INSETS
    } else {
        TALL_INSETS
    };
    Insets {
        top: f.top * height,
        right: f.right * width,
        bottom: f.bottom * height,
        left: f.left * width,
    }
}

pub fn design_scale_for(width: f64) -> f64 {
    width / DESIGN_WIDTH
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverlayFrame {
    pub k: f64,
    pub insets: Insets,
}

pub fn overlay_frame(stage: Stage) -> OverlayFrame {
    OverlayFrame {
        k: design_scale_for(stage.width.max(stage.height)),
        insets: shape_insets(stage.width, stage.height),
    }
}

pub fn overlay_padding(
    style: opsis_scene::style::Style,
    pad: f64,
    insets: Insets,
) -> opsis_scene::style::Style {
    style.padding(opsis_scene::values::Edges {
        top: (pad + insets.top).into(),
        right: (pad + insets.right).into(),
        bottom: (pad + insets.bottom).into(),
        left: (pad + insets.left).into(),
    })
}
