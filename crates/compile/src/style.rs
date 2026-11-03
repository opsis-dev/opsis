use opsis_types::recording::RecordingProps;
use opsis_types::values::{ClipValue, StyleEntry};

use crate::errors::SettingsError;
use crate::nodes::camera::CAMERA_TYPE;
use crate::registry::node_def;

pub fn restyle_with(
    props: RecordingProps,
    entries: &[StyleEntry],
) -> Result<RecordingProps, SettingsError> {
    let (cameras, rest): (Vec<&StyleEntry>, Vec<&StyleEntry>) =
        entries.iter().partition(|e| e.kind == CAMERA_TYPE);
    cameras
        .into_iter()
        .chain(rest)
        .try_fold(props, |p, e| match node_def(&e.kind) {
            Some(def) => def.apply_restyle(p, e),
            None => Ok(p),
        })
}

pub fn retime_with(
    clip: ClipValue,
    entries: &[StyleEntry],
    fps: u32,
) -> Result<ClipValue, SettingsError> {
    entries
        .iter()
        .try_fold(clip, |c, e| match node_def(&e.kind) {
            Some(def) => def.apply_retime(c, e, fps),
            None => Ok(c),
        })
}
