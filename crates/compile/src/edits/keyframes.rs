use opsis_types::edits::KeyframeEdit;
use opsis_types::timeline::{CameraKey, CameraPath, Ease, EnvelopeKey, Keyframe};
use opsis_types::workflow::{Workflow, WorkflowNode};
use serde_json::{Map, Value};

use crate::edits::EditError;
use crate::edits::workflow::node_mut;
use crate::envelope::{envelope_at, envelope_track, sort_keys};
use crate::text::round_to;
use crate::timeline::camera_path::{CAMERA_DIALS, CameraDialKey, camera_key_at};

const ENVELOPE_PARAM: &str = "envelope";
const CAMERA_PARAM: &str = "camera";
const VALUE_DIGITS: i32 = 3;
const NEW_KEY_GAP_SECONDS: f64 = 1.0;

fn envelope_of(node: &WorkflowNode) -> Result<Map<String, Value>, EditError> {
    match node.params.get(ENVELOPE_PARAM) {
        None | Some(Value::Null) => Ok(Map::new()),
        Some(Value::Object(map)) => Ok(map.clone()),
        Some(_) => Err(EditError::UnreadableKeys(node.id.clone())),
    }
}

fn track_keys(
    envelope: &Map<String, Value>,
    track: EnvelopeKey,
    node: &str,
) -> Result<Vec<Keyframe>, EditError> {
    match envelope.get(track.name()) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(keys) => serde_json::from_value::<Vec<Keyframe>>(keys.clone())
            .map_err(|_| EditError::UnreadableKeys(node.to_owned())),
    }
}

fn whole_frame(frame: f64) -> f64 {
    frame.round().max(0.0)
}

fn track_value(track: EnvelopeKey, value: f64) -> f64 {
    let info = envelope_track(track);
    round_to(value.clamp(info.min, info.max), VALUE_DIGITS)
}

fn edit_track(
    wf: &mut Workflow,
    node_id: &str,
    track: EnvelopeKey,
    change: impl FnOnce(&mut Vec<Keyframe>, f64),
) -> Result<(), EditError> {
    let fps = f64::from(wf.settings.fps);
    let node = node_mut(wf, node_id)?;
    let mut envelope = envelope_of(node)?;
    let mut keys = track_keys(&envelope, track, node_id)?;
    change(&mut keys, fps);
    let keys = sort_keys(&keys);
    if keys.is_empty() {
        envelope.remove(track.name());
    } else {
        envelope.insert(
            track.name().to_owned(),
            serde_json::to_value(keys).unwrap_or(Value::Null),
        );
    }
    node.params
        .insert(ENVELOPE_PARAM.to_owned(), Value::Object(envelope));
    Ok(())
}

fn camera_of(node: &WorkflowNode) -> Result<CameraPath, EditError> {
    match node.params.get(CAMERA_PARAM) {
        None | Some(Value::Null) => Ok(CameraPath::default()),
        Some(path) => serde_json::from_value(path.clone())
            .map_err(|_| EditError::UnreadableKeys(node.id.clone())),
    }
}

fn edit_camera(
    wf: &mut Workflow,
    node_id: &str,
    change: impl FnOnce(CameraPath) -> CameraPath,
) -> Result<(), EditError> {
    let node = node_mut(wf, node_id)?;
    let mut path = change(camera_of(node)?);
    path.keys.sort_by(|a, b| a.frame.total_cmp(&b.frame));
    node.params.insert(
        CAMERA_PARAM.to_owned(),
        serde_json::to_value(path).unwrap_or(Value::Null),
    );
    Ok(())
}

fn dial_value(key: CameraDialKey, value: f64) -> f64 {
    CAMERA_DIALS
        .iter()
        .find(|d| d.key == key)
        .map_or(value, |d| value.clamp(d.min, d.max))
}

fn set_camera_key(key: &mut CameraKey, patch: CameraKeyPatch) {
    if let Some(frame) = patch.frame {
        key.frame = whole_frame(frame);
    }
    if let Some(yaw) = patch.yaw {
        key.yaw = dial_value(CameraDialKey::Yaw, yaw);
    }
    if let Some(pitch) = patch.pitch {
        key.pitch = dial_value(CameraDialKey::Pitch, pitch);
    }
    if let Some(distance) = patch.distance {
        key.distance = dial_value(CameraDialKey::Distance, distance);
    }
    if let Some(height) = patch.height {
        key.height = dial_value(CameraDialKey::Height, height);
    }
    if let Some(ease) = patch.ease {
        key.ease = ease;
    }
}

#[derive(Clone, Copy)]
struct CameraKeyPatch {
    frame: Option<f64>,
    yaw: Option<f64>,
    pitch: Option<f64>,
    distance: Option<f64>,
    height: Option<f64>,
    ease: Option<Ease>,
}

fn at_index(index: u32) -> usize {
    usize::try_from(index).unwrap_or(usize::MAX)
}

pub fn apply_keyframe_edit(wf: &mut Workflow, edit: KeyframeEdit) -> Result<(), EditError> {
    match edit {
        KeyframeEdit::AddKey {
            node_id,
            track,
            frame,
        } => {
            edit_track(wf, &node_id, track, |keys, _| {
                let frame = whole_frame(frame);
                let value = envelope_at(keys, frame, envelope_track(track).rest);
                keys.push(Keyframe {
                    frame,
                    value: track_value(track, value),
                    ease: Ease::Linear,
                });
            })?;
        }
        KeyframeEdit::AppendKey { node_id, track } => {
            edit_track(wf, &node_id, track, |keys, fps| {
                let next = keys.last().map_or_else(
                    || Keyframe {
                        frame: 0.0,
                        value: envelope_track(track).rest,
                        ease: Ease::Linear,
                    },
                    |last| Keyframe {
                        frame: last.frame + NEW_KEY_GAP_SECONDS * fps,
                        value: last.value,
                        ease: Ease::Linear,
                    },
                );
                keys.push(next);
            })?;
        }
        KeyframeEdit::SetKey {
            node_id,
            track,
            index,
            frame,
            value,
            ease,
        } => {
            edit_track(wf, &node_id, track, |keys, _| {
                if let Some(key) = keys.get_mut(at_index(index)) {
                    if let Some(frame) = frame {
                        key.frame = whole_frame(frame);
                    }
                    if let Some(value) = value {
                        key.value = track_value(track, value);
                    }
                    if let Some(ease) = ease {
                        key.ease = ease;
                    }
                }
            })?;
        }
        KeyframeEdit::RemoveKey {
            node_id,
            track,
            index,
        } => {
            edit_track(wf, &node_id, track, |keys, _| {
                if at_index(index) < keys.len() {
                    keys.remove(at_index(index));
                }
            })?;
        }
        KeyframeEdit::AddCameraKey { node_id, frame } => {
            edit_camera(wf, &node_id, |path| camera_key_at(&path, frame))?;
        }
        KeyframeEdit::SetCameraKey {
            node_id,
            index,
            frame,
            yaw,
            pitch,
            distance,
            height,
            ease,
        } => {
            edit_camera(wf, &node_id, |mut path| {
                if let Some(key) = path.keys.get_mut(at_index(index)) {
                    set_camera_key(
                        key,
                        CameraKeyPatch {
                            frame,
                            yaw,
                            pitch,
                            distance,
                            height,
                            ease,
                        },
                    );
                }
                path
            })?;
        }
        KeyframeEdit::RemoveCameraKey { node_id, index } => {
            edit_camera(wf, &node_id, |mut path| {
                if at_index(index) < path.keys.len() {
                    path.keys.remove(at_index(index));
                }
                path
            })?;
        }
    }
    Ok(())
}
