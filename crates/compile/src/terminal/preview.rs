use opsis_types::actions::Viewport;
use opsis_types::timeline::TimelineSpec;
use opsis_types::values::{RecordingValue, StyleEntry, TakeSource};
use opsis_types::workflow::WorkflowSettings;
use serde_json::{Value, json};

use crate::clip::{ItemExtra, clip_to_spec, recording_clip};
use crate::errors::SettingsError;
use crate::nodes::camera::{CAMERA_TYPE, MOTION_BLUR_TYPE, SHOT_TYPE, clip_from_take_props};
use crate::terminal::{default_look, grid, screen_size};
use crate::time::ms_to_frames;

pub const PREVIEW_FPS: u32 = 30;
pub const PREVIEW_FILE: &str = "preview.mp4";

fn plain(node_id: &str, kind: &str, params: Value) -> StyleEntry {
    StyleEntry {
        node_id: node_id.to_owned(),
        kind: kind.to_owned(),
        params: match params {
            Value::Object(map) => map,
            _ => serde_json::Map::new(),
        },
    }
}

const fn even(size: u32) -> u32 {
    size + size % 2
}

pub fn preview_size(take: &RecordingValue) -> Option<Viewport> {
    let TakeSource::Terminal(source) = &take.source else {
        return None;
    };
    let look = default_look(source.theme.as_deref().cloned());
    let size = screen_size(&grid(&look, source.cols, source.rows));
    Some(Viewport {
        width: even(size.width),
        height: even(size.height),
    })
}

pub fn preview_spec(take: &RecordingValue) -> Result<Option<TimelineSpec>, SettingsError> {
    let Some(size) = preview_size(take) else {
        return Ok(None);
    };
    let take = RecordingValue {
        edits: Vec::new(),
        ..take.clone()
    };
    let id = take.node_id.as_str();
    let entries = [
        plain(id, CAMERA_TYPE, json!({ "zoom": false })),
        plain(
            id,
            SHOT_TYPE,
            json!({ "windowSize": 100, "cornerRadius": 0, "shadow": 0, "liftColours": false }),
        ),
        plain(id, MOTION_BLUR_TYPE, json!({ "enabled": false })),
    ];
    let Some(props) = clip_from_take_props(&take, 1.0, &entries)? else {
        return Ok(None);
    };
    let frames = ms_to_frames(take.duration_ms, PREVIEW_FPS);
    let clip = recording_clip(
        id,
        props,
        frames,
        ItemExtra {
            recording: Some(take.clone()),
            ..ItemExtra::default()
        },
    );
    let settings = WorkflowSettings {
        width: size.width,
        height: size.height,
        fps: PREVIEW_FPS,
        ..WorkflowSettings::default()
    };
    Ok(Some(clip_to_spec(&clip, &settings)))
}
