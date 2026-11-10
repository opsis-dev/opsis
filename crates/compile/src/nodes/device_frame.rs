use std::sync::LazyLock;

use opsis_macros::{choice, settings};
use opsis_types::actions::Viewport;
use opsis_types::cards::{CardRow, Icon};
use opsis_types::looks::NodeLook;
use opsis_types::recording::{RecordingProps, ShotProps};
use opsis_types::timeline::{EditHandles, ElementRef, GroupProps};
use opsis_types::values::{ClipValue, CompileValue};
use opsis_types::workflow::PortType;
use serde_json::{Value, json};

use crate::clip::{patch_recordings, with_handles, wrap_clip};
use crate::elements::device::device_mockup_zoom::MockupDevice;
use crate::elements::device::screen_3d::ScreenDevice;
use crate::node::{CLIP_PORT, CardCtx, Category, Ctx, NodeDef, Phase, outputs, port, row};
use crate::nodes::elements::{active_camera, element_props, properties_of};
use crate::nodes::groups::{GroupKind, kind_object_shown, kind_values};
use crate::params::{ParamSchema, Values, schema_of};

pub const DEVICE_FRAME_TYPE: &str = "camera.deviceFrame";
const TILT_TURN_LIMIT: u32 = 30;
const MOVE: &str = "move";
const CAMERA: &str = "camera";

#[choice]
pub enum Angle {
    #[label("Flat")]
    Flat,
    #[label("Tilted")]
    Tilt,
    #[label("3D")]
    ThreeD,
    #[label("Pull back")]
    PullBack,
    #[label("Lineup")]
    Lineup,
}

#[choice]
pub enum FrameDevice {
    #[label("Laptop")]
    Laptop,
    #[label("Phone")]
    Phone,
}

#[settings(eq)]
pub struct DeviceFrame {
    #[setting("Angle", default = Angle::Flat)]
    pub angle: Angle,
    #[setting(
        "Device",
        default = FrameDevice::Laptop,
        show_if = { "angle": ["flat", "tilt"] }
    )]
    pub device: FrameDevice,
    #[setting(
        "Device",
        default = ScreenDevice::Laptop,
        show_if = { "angle": ["three-d"] }
    )]
    pub screen_device: ScreenDevice,
    #[setting(
        "Device",
        default = MockupDevice::Laptop,
        show_if = { "angle": ["pull-back"] }
    )]
    pub pull_device: MockupDevice,
    #[setting(
        "Tilt",
        default = 18,
        range(min = 0, max = 45),
        step = 1,
        unit = "°",
        show_if = { "angle": ["tilt"] }
    )]
    pub tilt: u32,
}

struct Kinds {
    laptop: GroupKind,
    phone: GroupKind,
    screen: GroupKind,
    pull_back: GroupKind,
    lineup: GroupKind,
}

static KINDS: LazyLock<Kinds> = LazyLock::new(|| Kinds {
    laptop: GroupKind::new("laptop", "Laptop", "laptop-frame").hide(&["restTilt"]),
    phone: GroupKind::new("phone", "Phone", "phone-frame").hide(&["variant", "tiltAngle"]),
    screen: GroupKind::new("screen", "3D screen", "screen-3d").hide(&["device", MOVE, CAMERA]),
    pull_back: GroupKind::new("pull-back", "Pull back", "device-mockup-zoom").hide(&["device"]),
    lineup: GroupKind::new("lineup", "Lineup", "multi-device-lineup"),
});

pub const fn drawn_elements() -> [&'static str; 5] {
    [
        "laptop-frame",
        "phone-frame",
        "screen-3d",
        "device-mockup-zoom",
        "multi-device-lineup",
    ]
}

fn with_angle(mut field: Value, angle: &str) -> Value {
    if let Some(object) = field.as_object_mut() {
        let mut rules = object
            .get("x-show-if")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        rules.insert("angle".to_owned(), json!([angle]));
        object.insert("x-show-if".to_owned(), Value::Object(rules));
    }
    field
}

fn schema() -> ParamSchema {
    let k = &*KINDS;
    let mut properties = properties_of(&schema_of::<DeviceFrame>());
    let screen = k.screen.meta();
    for key in [MOVE, CAMERA] {
        if let Some(field) = screen.schema.field(key) {
            properties.insert(key.to_owned(), with_angle(field.clone(), "three-d"));
        }
    }
    let objects = [
        (
            &k.laptop,
            json!({ "angle": ["flat", "tilt"], "device": ["laptop"] }),
        ),
        (
            &k.phone,
            json!({ "angle": ["flat", "tilt"], "device": ["phone"] }),
        ),
        (&k.screen, json!({ "angle": ["three-d"] })),
        (&k.pull_back, json!({ "angle": ["pull-back"] })),
        (&k.lineup, json!({ "angle": ["lineup"] })),
    ];
    for (kind, show_if) in objects {
        properties.insert(kind.id.to_owned(), kind_object_shown(kind, &[], &show_if));
    }
    ParamSchema::new(json!({ "type": "object", "properties": properties }))
}

fn json_of(value: impl serde::Serialize) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

fn picked(p: &DeviceFrame, values: &Values) -> (&'static GroupKind, Values) {
    let k = &*KINDS;
    let mut fixed = Values::new();
    let kind = match (p.angle, p.device) {
        (Angle::Flat | Angle::Tilt, FrameDevice::Laptop) => {
            let lid = if p.angle == Angle::Tilt { p.tilt } else { 0 };
            fixed.insert("restTilt".to_owned(), Value::from(lid));
            &k.laptop
        }
        (Angle::Flat | Angle::Tilt, FrameDevice::Phone) => {
            let tilted = p.angle == Angle::Tilt;
            fixed.insert(
                "variant".to_owned(),
                Value::from(if tilted { "tilt" } else { "flat" }),
            );
            let turn = i64::from(p.tilt.min(TILT_TURN_LIMIT));
            fixed.insert("tiltAngle".to_owned(), Value::from(-turn));
            &k.phone
        }
        (Angle::ThreeD, _) => {
            fixed.insert("device".to_owned(), json_of(p.screen_device));
            for key in [MOVE, CAMERA] {
                if let Some(value) = values.get(key) {
                    fixed.insert(key.to_owned(), value.clone());
                }
            }
            &k.screen
        }
        (Angle::PullBack, _) => {
            fixed.insert("device".to_owned(), json_of(p.pull_device));
            &k.pull_back
        }
        (Angle::Lineup, _) => &k.lineup,
    };
    let mut merged = kind_values(kind, values, &[]);
    merged.extend(fixed);
    (kind, merged)
}

fn shown_on(p: &DeviceFrame) -> (Icon, String) {
    let (icon, device) = match p.angle {
        Angle::Flat | Angle::Tilt => match p.device {
            FrameDevice::Laptop => (Icon::Laptop, p.device.label()),
            FrameDevice::Phone => (Icon::Smartphone, p.device.label()),
        },
        Angle::ThreeD => (Icon::Laptop, p.screen_device.label()),
        Angle::PullBack => (Icon::Laptop, p.pull_device.label()),
        Angle::Lineup => (Icon::Laptop, "Phone, tablet and laptop"),
    };
    (icon, format!("{device} · {}", p.angle.label()))
}

fn card(ctx: &CardCtx<'_, DeviceFrame>) -> Vec<Option<CardRow>> {
    let (icon, text) = shown_on(&ctx.params);
    vec![Some(row(icon, text))]
}

fn framed(ctx: &Ctx<'_, DeviceFrame>, clip: &ClipValue) -> ClipValue {
    let (kind, values) = picked(&ctx.params, ctx.values);
    let meta = kind.meta();
    let props = element_props(meta, &values);
    let screened = patch_recordings(clip, &|r| RecordingProps {
        shot: ShotProps {
            in_device: Some(true),
            ..r.shot
        },
        ..r
    });
    let group = GroupProps {
        wrapper: Some(ElementRef {
            component: meta.key.clone(),
            props,
        }),
        design: meta.design.as_ref().map(|d| Viewport {
            width: d.width,
            height: d.height,
        }),
        ..GroupProps::default()
    };
    let wrapped = wrap_clip(&screened, ctx.node_id, group, Some(clip.duration_in_frames));
    with_handles(
        wrapped,
        EditHandles::default(),
        None,
        active_camera(&values),
    )
}

pub fn device_frame() -> NodeDef {
    let mut def = NodeDef::new(
        DEVICE_FRAME_TYPE,
        "Device frame",
        Category::Camera,
        Phase::Compile,
        schema(),
    );
    def = def
        .describe("Puts the clip on a device's screen: a laptop or phone, a 3D screen, a pull back from a device, or a lineup of devices.")
        .look(NodeLook::of("lucide:laptop").morphs("lucide:smartphone"))
        .inputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
        .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
        .card(card)
        .compiles(|ctx: &Ctx<'_, DeviceFrame>| {
            match ctx.input(CLIP_PORT).and_then(CompileValue::as_clip) {
                Some(clip) => outputs([(CLIP_PORT, CompileValue::Clip(framed(ctx, clip)))]),
                None => crate::node::Outputs::new(),
            }
        });
    def
}
