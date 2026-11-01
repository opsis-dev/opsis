use opsis_types::numbers::exact_eq;
use opsis_types::timeline::{
    CAMERA_DISTANCE, CAMERA_HEIGHT, CAMERA_PITCH, CAMERA_YAW, CameraKey, CameraPath, DialRange,
    Ease,
};
use serde::Serialize;
use specta::Type;

use crate::envelope::ease;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraView {
    pub yaw: f64,
    pub pitch: f64,
    pub distance: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum CameraDialKey {
    Yaw,
    Pitch,
    Distance,
    Height,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Type)]
pub struct CameraDial {
    pub key: CameraDialKey,
    pub label: &'static str,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub unit: &'static str,
}

const fn dial(
    key: CameraDialKey,
    label: &'static str,
    range: DialRange,
    step: f64,
    unit: &'static str,
) -> CameraDial {
    CameraDial {
        key,
        label,
        min: range.min,
        max: range.max,
        step,
        unit,
    }
}

pub const CAMERA_DIALS: [CameraDial; 4] = [
    dial(CameraDialKey::Yaw, "Turn", CAMERA_YAW, 1.0, "°"),
    dial(CameraDialKey::Pitch, "Tilt", CAMERA_PITCH, 1.0, "°"),
    dial(
        CameraDialKey::Distance,
        "Distance",
        CAMERA_DISTANCE,
        0.05,
        "×",
    ),
    dial(CameraDialKey::Height, "Height", CAMERA_HEIGHT, 0.05, ""),
];

pub const REST_VIEW: CameraView = CameraView {
    yaw: 0.0,
    pitch: 0.0,
    distance: 1.0,
    height: 0.0,
};

fn view_of(k: &CameraKey) -> CameraView {
    CameraView {
        yaw: k.yaw,
        pitch: k.pitch,
        distance: k.distance,
        height: k.height,
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn mix_views(a: &CameraKey, b: &CameraKey, t: f64) -> CameraView {
    let e = ease(a.ease, t);
    CameraView {
        yaw: lerp(a.yaw, b.yaw, e),
        pitch: lerp(a.pitch, b.pitch, e),
        distance: lerp(a.distance, b.distance, e),
        height: lerp(a.height, b.height, e),
    }
}

pub fn sorted_camera_keys(path: Option<&CameraPath>) -> Vec<CameraKey> {
    let mut keys = path.map_or_default(|p| p.keys.clone());
    keys.sort_by(|a, b| a.frame.total_cmp(&b.frame));
    keys
}

pub fn camera_at(path: Option<&CameraPath>, frame: f64, end_face_on: bool) -> CameraView {
    let mut keys = sorted_camera_keys(path);
    if end_face_on && let Some(last) = keys.last_mut() {
        last.yaw = REST_VIEW.yaw;
        last.pitch = REST_VIEW.pitch;
        last.distance = REST_VIEW.distance;
        last.height = REST_VIEW.height;
    }
    let (Some(first), Some(last)) = (keys.first(), keys.last()) else {
        return REST_VIEW;
    };
    if frame <= first.frame {
        return view_of(first);
    }
    if frame >= last.frame {
        return view_of(last);
    }
    let i = keys.partition_point(|k| k.frame <= frame);
    let (a, b) = (&keys[i - 1], &keys[i]);
    mix_views(a, b, (frame - a.frame) / (b.frame - a.frame))
}

pub fn move_camera_key(path: &CameraPath, index: usize, frame: f64) -> CameraPath {
    let mut keys = sorted_camera_keys(Some(path));
    let Some(key) = keys.get_mut(index) else {
        return path.clone();
    };
    key.frame = frame.round().max(0.0);
    keys.sort_by(|a, b| a.frame.total_cmp(&b.frame));
    CameraPath { keys }
}

pub fn camera_key_at(path: &CameraPath, frame: f64) -> CameraPath {
    let at = frame.round().max(0.0);
    let view = camera_at(Some(path), at, false);
    let mut keys: Vec<CameraKey> = sorted_camera_keys(Some(path))
        .into_iter()
        .filter(|k| !exact_eq(k.frame, at))
        .collect();
    keys.push(CameraKey {
        frame: at,
        yaw: view.yaw,
        pitch: view.pitch,
        distance: view.distance,
        height: view.height,
        ease: Ease::InOut,
    });
    keys.sort_by(|a, b| a.frame.total_cmp(&b.frame));
    CameraPath { keys }
}
