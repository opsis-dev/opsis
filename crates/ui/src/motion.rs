use opsis_compile::motions::{MotionSpec, Part, Pose, motion_spec, sample};
use opsis_qt::{MotionInfo, PoseView};
use opsis_types::looks::NodeMotion;
use serde_json::Value;

fn spec_of(name: &str) -> Option<MotionSpec> {
    serde_json::from_value::<NodeMotion>(Value::String(name.to_owned()))
        .ok()
        .map(motion_spec)
}

pub fn info(name: &str) -> MotionInfo {
    let Some(spec) = spec_of(name) else {
        return MotionInfo {
            ms: 0,
            origin_x: 0.5,
            origin_y: 0.5,
            part: String::new(),
            clip: false,
        };
    };
    let part = match spec.part {
        Part::Glyph => "glyph",
        Part::Shapes => "shapes",
        Part::March => "march",
        Part::Ring => "ring",
    };
    MotionInfo {
        ms: spec.ms,
        origin_x: spec.origin.0,
        origin_y: spec.origin.1,
        part: part.to_owned(),
        clip: spec.clip,
    }
}

pub fn pose(name: &str, elapsed_ms: f64) -> PoseView {
    let p: Pose = spec_of(name).map_or(opsis_compile::motions::REST, |spec| {
        sample(&spec, elapsed_ms)
    });
    PoseView {
        tx: p.tx,
        ty: p.ty,
        sx: p.sx,
        sy: p.sy,
        rotate: p.rotate,
        opacity: p.opacity,
        blur: p.blur,
        dash: p.dash,
        smear: p.smear,
    }
}
