use opsis_macros::settings;
use opsis_scene::node::{Bevel3d, Node, Object3d, Outline2d, Shape3d, Stage3d, Texture3d, div};
use opsis_scene::paint::AssetRef;
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::Num;
use opsis_types::looks::NodeLook;
use opsis_types::timeline::CameraPath;

use crate::elements::draw::{Cx, DrawFailure};
use crate::elements::entry::{element, meta};
use crate::elements::shared::{Lighting, Motion};
use crate::elements::three::motion::{MotionTime, element_pose};
use crate::elements::three::{
    Matrix, SUBJECT_RADIUS, fit_distance, fitted_size, lighting_of, multiply, object, pose_matrix,
    rounded_rect, staged, standard, textured, translation, unlit,
};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct Card3d {
    #[setting(
        "Image",
        widget = "asset",
        accept = "image/*",
        zero_label = "Sample screen"
    )]
    pub src: String,
    #[setting(
        "Rounded corners",
        default = 0.04,
        range(min = 0, max = 0.2),
        step = 0.01
    )]
    pub corner_radius: f64,
    #[setting(
        "Thickness",
        default = 0.04,
        range(min = 0.01, max = 0.3),
        step = 0.01,
        advanced
    )]
    pub depth: f64,
    #[setting(
        "Tilt",
        default = 40,
        range(min = 0, max = 80),
        step = 1,
        unit = "°",
        help = "How far the card is turned before it settles."
    )]
    pub tilt: u32,
    #[setting(
        "Motion",
        default = Motion::TiltIn
    )]
    pub motion: Motion,
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
        "Lighting",
        default = Lighting::Soft
    )]
    pub lighting: Lighting,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "None",
        check = colour_or_empty
    )]
    pub background: String,
    #[setting("Zoom", default = 1.0, range(min = 0.3, max = 3))]
    pub zoom: f64,
    #[setting("Edge colour", default = "#ffffff", widget = "color", check = colour, advanced)]
    pub edge_color: String,
    #[setting("Matte", default = 0.6, range(min = 0, max = 1), advanced)]
    pub roughness: f64,
    #[setting("Shadow strength", default = 0.55, range(min = 0, max = 1), advanced)]
    pub shadow: f64,
    #[setting("Grow in", default = true, advanced)]
    pub grow_in: bool,
    #[setting("End facing the camera", advanced)]
    pub end_facing: bool,
    #[setting("Motion amount", default = 1.0, range(min = 0, max = 2), advanced)]
    pub motion_amount: f64,
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
    meta::<Card3d>(
        element(
            ElementKind::Component,
            "card-3d",
            "3D card",
            "An image on a card with rounded corners and a soft shadow that tilts into place.",
            ElementCategory::Scene,
            120,
        )
        .look(NodeLook::of("lucide-lab:copy-image")),
    )
}

const SAMPLE_SCREEN: &str = "app:placeholders/screen-wide.svg";
const CARD_SIZE: f64 = 2.2;
const FACE_GAP: f64 = 0.002;
const BODY_ROUGHNESS: f64 = 0.45;
const SHADOW_SPREAD: f64 = 1.25;
const SHADOW_DROP: f64 = 0.18;
const SHADOW_BACK: f64 = 0.6;
const SHADOW_SIDE: u32 = 256;
const SHADOW_ALPHA: f64 = 0.55;
const SEED: f64 = 1.0;
const BEVEL_SHARE: f64 = 0.2;
const BEVEL_SEGMENTS: u32 = 4;
const BODY_CURVES: u32 = 16;
const FACE_CURVES: u32 = 12;
pub const PICTURE: u32 = 0;
const GLOW: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardLook<'a> {
    pub edge_color: &'a str,
    pub metalness: f64,
    pub face_roughness: f64,
}

#[expect(
    clippy::suboptimal_flops,
    reason = "each step rounds on its own, and the recorded goldens hold what that draws"
)]
pub fn card_objects(
    (width, height): (f64, f64),
    longest: f64,
    radius: f64,
    depth: f64,
    look: &CardLook<'_>,
    posed: &Matrix,
) -> [Object3d; 2] {
    let corner = radius * longest;
    let outline = Outline2d {
        shape: rounded_rect(width, height, corner),
        holes: Vec::new(),
    };
    let front = depth / 2.0 + depth * BEVEL_SHARE + FACE_GAP;
    let edge = if look.edge_color.is_empty() {
        "#ffffff"
    } else {
        look.edge_color
    };
    let body = object(
        Shape3d::Extruded {
            outlines: vec![outline.clone()],
            curve_segments: BODY_CURVES,
            depth: Num(depth),
            bevel: Some(Bevel3d {
                thickness: Num(depth * BEVEL_SHARE),
                size: Num(corner.min(depth) * BEVEL_SHARE),
                offset: Num(0.0),
                segments: BEVEL_SEGMENTS,
            }),
        },
        &multiply(posed, &translation(0.0, 0.0, -depth / 2.0)),
        standard(edge, BODY_ROUGHNESS, look.metalness),
    );
    let face = object(
        Shape3d::Flat {
            outline,
            curve_segments: FACE_CURVES,
            uv_span: [Num(width), Num(height)],
            uv_offset: [Num(0.5), Num(0.5)],
        },
        &multiply(posed, &translation(0.0, 0.0, front)),
        textured(standard("#ffffff", look.face_roughness, 0.0), PICTURE),
    );
    [body, face]
}

pub fn picture_size(cx: &Cx<'_>, src: &str) -> Option<(f64, f64)> {
    let (w, h) = cx.measure.picture_size(src)?;
    Some((if w > 0.0 { w } else { 1.0 }, if h > 0.0 { h } else { 1.0 }))
}

pub fn draw(props: &Card3d, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let src = if props.src.is_empty() {
        SAMPLE_SCREEN
    } else {
        props.src.as_str()
    };
    let Some((w, h)) = picture_size(cx, src) else {
        return Ok(div(absolute_fill()).into());
    };
    let pose = element_pose(
        props.motion,
        Some(&props.camera),
        &MotionTime {
            frame: cx.frame(),
            fps: cx.fps(),
            speed: props.speed,
            seed: SEED,
            tilt_deg: f64::from(props.tilt),
            end_facing: props.end_facing,
            amount: props.motion_amount,
            reverse: props.reverse,
            grow_in: props.grow_in,
        },
    );
    let longest = CARD_SIZE * props.zoom;
    let (width, height) = fitted_size(w, h, longest);
    let [body, face] = card_objects(
        (width, height),
        longest,
        props.corner_radius,
        props.depth,
        &CardLook {
            edge_color: &props.edge_color,
            metalness: 0.0,
            face_roughness: props.roughness,
        },
        &pose_matrix(&pose),
    );
    let shadow = object(
        Shape3d::Plane {
            width: Num(width * SHADOW_SPREAD),
            height: Num(height * SHADOW_SPREAD),
        },
        &translation(0.0, -SHADOW_DROP, -SHADOW_BACK),
        unlit(props.shadow, GLOW),
    );
    let aspect = cx.at.stage.width / cx.at.stage.height;
    Ok(staged(
        Stage3d {
            fov: Num(props.lens),
            distance: Num(fit_distance(SUBJECT_RADIUS, props.lens, aspect)),
            lighting: lighting_of(props.lighting),
            objects: vec![shadow, body, face],
            textures: vec![
                Texture3d::Picture(AssetRef(src.to_owned())),
                Texture3d::Glow {
                    side: SHADOW_SIDE,
                    alpha: Num(SHADOW_ALPHA),
                },
            ],
            models: Vec::new(),
        },
        &props.background,
    ))
}
