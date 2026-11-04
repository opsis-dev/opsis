use opsis_types::numbers::whole;
use opsis_types::values::{ClipValue, CompileValue, MediaKind, MediaValue, TakeKind, TakeSource};
use opsis_types::workflow::PortType;

use crate::clip::clip_of;
use crate::node::{FamilyLabel, Inputs, PortFamily, PortSpec, numbered_index};

fn image_as_media(value: &CompileValue) -> Option<CompileValue> {
    let CompileValue::Image(image) = value else {
        return None;
    };
    Some(CompileValue::Media(MediaValue {
        src: image.src.clone(),
        kind: MediaKind::Image,
        duration_ms: None,
        width: image.width,
        height: image.height,
        has_audio: None,
    }))
}

fn recording_as_media(value: &CompileValue) -> Option<CompileValue> {
    let CompileValue::Recording(recording) = value else {
        return None;
    };
    let TakeSource::Browser(browser) = &recording.source else {
        return None;
    };
    Some(CompileValue::Media(MediaValue {
        src: browser.video.clone(),
        kind: MediaKind::Video,
        duration_ms: Some(whole(recording.duration_ms)),
        width: Some(browser.viewport.width),
        height: Some(browser.viewport.height),
        has_audio: Some(false),
    }))
}

type Conversion = fn(&CompileValue) -> Option<CompileValue>;

fn conversion(source: PortType, target: PortType) -> Option<Conversion> {
    match (target, source) {
        (PortType::Media, PortType::Image) => Some(image_as_media),
        (PortType::Media, PortType::Recording) => Some(recording_as_media),
        _ => None,
    }
}

pub fn can_connect(source: PortType, target: PortType) -> bool {
    source == target || conversion(source, target).is_some()
}

pub fn value_converts(value: &CompileValue, target: PortType) -> bool {
    value.port_type() == target
        || conversion(value.port_type(), target).is_some_and(|convert| convert(value).is_some())
}

pub fn value_conversions(value: &CompileValue) -> Vec<PortType> {
    PortType::ALL
        .iter()
        .copied()
        .filter(|t| value_converts(value, *t))
        .collect()
}

pub fn refused_conversion(value: &CompileValue, target: PortType) -> Option<String> {
    if value_converts(value, target) {
        return None;
    }
    Some(match value {
        CompileValue::Recording(take) if take.kind() == TakeKind::Terminal => {
            "A terminal take has no video file. Make a clip of it with Clip from take".to_owned()
        }
        _ => format!(
            "This {} can't connect to a {} input",
            value.port_type().word(),
            target.word()
        ),
    })
}

pub fn convert_input(
    value: Option<&CompileValue>,
    spec: Option<&PortSpec>,
) -> Option<CompileValue> {
    let value = value?;
    let Some(spec) = spec else {
        return Some(value.clone());
    };
    if value.port_type() == spec.kind {
        return Some(value.clone());
    }
    conversion(value.port_type(), spec.kind).and_then(|convert| convert(value))
}

pub const IN_PREFIX: &str = "in";

pub fn in_index(handle: &str) -> Option<usize> {
    numbered_index(IN_PREFIX, handle)
}

pub fn in_port(index: usize) -> String {
    format!("{IN_PREFIX}{}", index + 1)
}

pub fn clip_label(i: usize, _last: Option<usize>) -> String {
    format!("Clip {}", i + 1)
}

pub fn layer_label(i: usize, last: Option<usize>) -> String {
    let end = match i {
        0 => " (back)",
        _ if last == Some(i) => " (front)",
        _ => "",
    };
    format!("Layer {}{end}", i + 1)
}

pub fn scene_label(i: usize, _last: Option<usize>) -> String {
    format!("Scene {}", i + 1)
}

pub fn clip_family(label: FamilyLabel) -> PortFamily {
    PortFamily::new(IN_PREFIX, PortType::Clip, 0, label)
}

pub fn connected_clips(inputs: &Inputs) -> Vec<(String, &ClipValue)> {
    let mut found: Vec<(usize, &str, &ClipValue)> = inputs
        .iter()
        .filter_map(|(k, v)| Some((in_index(k)?, k.as_str(), clip_of(Some(v))?)))
        .collect();
    found.sort_by_key(|(i, _, _)| *i);
    found
        .into_iter()
        .map(|(_, port, clip)| (port.to_owned(), clip))
        .collect()
}
