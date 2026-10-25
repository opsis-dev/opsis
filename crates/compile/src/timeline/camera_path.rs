use opsis_types::timeline::{CameraKey, CameraPath, Ease};

use crate::envelope::ease;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraView {
    pub yaw: f64,
    pub pitch: f64,
    pub distance: f64,
    pub height: f64,
}

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
    let mut keys = path.map(|p| p.keys.clone()).unwrap_or_default();
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
        .filter(|k| k.frame != at)
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
