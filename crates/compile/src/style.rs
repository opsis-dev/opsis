use opsis_types::recording::RecordingProps;
use opsis_types::values::{ClipValue, StyleEntry};

use crate::registry::node_def;

pub fn restyle_with(
    props: RecordingProps,
    entries: &[StyleEntry],
) -> Result<RecordingProps, String> {
    entries
        .iter()
        .try_fold(props, |p, e| match node_def(&e.kind) {
            Some(def) => def.apply_restyle(p, e),
            None => Ok(p),
        })
}

pub fn retime_with(clip: ClipValue, entries: &[StyleEntry], fps: f64) -> Result<ClipValue, String> {
    entries
        .iter()
        .try_fold(clip, |c, e| match node_def(&e.kind) {
            Some(def) => def.apply_retime(c, e, fps),
            None => Ok(c),
        })
}
