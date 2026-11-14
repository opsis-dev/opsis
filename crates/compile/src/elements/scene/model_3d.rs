use opsis_macros::{choice, settings};
use opsis_scene::node::{Model3d as StagedModel, Node, Stage3d, div};
use opsis_scene::paint::AssetRef;
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::Num;
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::timeline::CameraPath;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shared::{Lighting, Motion};
use crate::elements::three::motion::{MotionTime, element_pose};
use crate::elements::three::{SUBJECT_RADIUS, fit_distance, lighting_of, pose_matrix, staged};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum ModelMotion {
    #[label("Turntable")]
    Spin,
    #[label("Orbit")]
    Orbit,
    #[label("Swing")]
    Swing,
    #[label("Float")]
    Float,
    #[label("Still")]
    Still,
    #[label("Keyframes")]
    Keyframes,
}

#[settings(export)]
pub struct Model3d {
    #[setting(
        "Model",
        widget = "asset",
        accept = ".glb,.gltf",
        help = "A 3D model file. A model saved as several files keeps them next to it."
    )]
    pub src: String,
    #[setting(
        "Motion",
        default = ModelMotion::Spin
    )]
    pub motion: ModelMotion,
    #[setting(
        "Camera keyframes",
        help = "Turn, raise and move the camera at chosen moments. Add a key at the playhead, then set it. Drag keys on the timeline to move them.",
        widget = "camera",
        show_if = { "motion": ["keyframes"] }
    )]
    pub camera: CameraPath,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 3), step = 0.05)]
    pub speed: f64,
    #[setting(
        "Animation",
        help = "The name of the model's own animation to play. Leave empty for the first one.",
        advanced
    )]
    pub animation: String,
    #[setting(
        "Animation speed",
        default = 1.0,
        range(min = 0.1, max = 4),
        step = 0.05,
        advanced
    )]
    pub animation_speed: f64,
    #[setting("Loop the animation", default = true, advanced)]
    pub r#loop: bool,
    #[setting("Zoom", default = 1.0, range(min = 0.3, max = 3), step = 0.05)]
    pub zoom: f64,
    #[setting(
        "Lighting",
        default = Lighting::Studio
    )]
    pub lighting: Lighting,
    #[setting("Floor shadow", default = true)]
    pub shadow: bool,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "None",
        check = colour_or_empty
    )]
    pub background: String,
    #[setting("Shadow strength", default = 0.55, range(min = 0, max = 1), advanced)]
    pub shadow_opacity: f64,
    #[setting("Grow in", default = true, advanced)]
    pub grow_in: bool,
    #[setting("Motion amount", default = 1.0, range(min = 0, max = 2), advanced)]
    pub motion_amount: f64,
    #[setting(
        "Tilt",
        default = 52,
        range(min = 0, max = 80),
        step = 1,
        unit = "°",
        advanced
    )]
    pub tilt: u32,
    #[setting("End facing the camera", advanced)]
    pub end_facing: bool,
    #[setting("Turn the other way", advanced)]
    pub reverse: bool,
    #[setting(
        "Lens",
        default = 30.0,
        range(min = 10, max = 70),
        unit = "°",
        advanced
    )]
    pub lens: f64,
}

pub fn definition() -> ElementMeta {
    meta::<Model3d>(element(ElementKind::Component, "model-3d",
            "3D model",
            "A 3D model from your project on a turntable, with studio lighting and its own animation.",
            ElementCategory::Scene,
            150).look(NodeLook::of("lucide:cuboid").moves(NodeMotion::Turn)))
}

const MODEL_SIZE: f64 = 2.2;
const SEED: f64 = 1.0;

const fn motion_of(motion: ModelMotion) -> Motion {
    match motion {
        ModelMotion::Spin => Motion::Spin,
        ModelMotion::Orbit => Motion::Orbit,
        ModelMotion::Swing => Motion::Swing,
        ModelMotion::Float => Motion::Float,
        ModelMotion::Still => Motion::Still,
        ModelMotion::Keyframes => Motion::Keyframes,
    }
}

pub fn draw(props: &Model3d, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    if props.src.is_empty() {
        return Ok(div(absolute_fill()).into());
    }
    let frame = cx.frame();
    let fps = cx.fps();
    let pose = element_pose(
        motion_of(props.motion),
        Some(&props.camera),
        &MotionTime {
            frame,
            fps,
            speed: props.speed,
            seed: SEED,
            tilt_deg: f64::from(props.tilt),
            end_facing: props.end_facing,
            amount: props.motion_amount,
            reverse: props.reverse,
            grow_in: props.grow_in,
        },
    );
    let aspect = cx.at.stage.width / cx.at.stage.height;
    Ok(staged(
        Stage3d {
            fov: Num(props.lens),
            distance: Num(fit_distance(SUBJECT_RADIUS, props.lens, aspect)),
            lighting: lighting_of(props.lighting),
            objects: Vec::new(),
            textures: Vec::new(),
            models: vec![StagedModel {
                source: AssetRef(props.src.clone()),
                matrix: pose_matrix(&pose).map(Num),
                size: Num(MODEL_SIZE * props.zoom),
                animation: props.animation.clone(),
                looped: props.r#loop,
                time: Num((frame / fps) * props.animation_speed),
                shadow: props.shadow.then_some(Num(props.shadow_opacity)),
            }],
        },
        &props.background,
    ))
}
