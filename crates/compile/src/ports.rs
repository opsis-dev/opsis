use opsis_types::values::{ClipValue, CompileValue, MediaKind, MediaValue};
use opsis_types::workflow::PortType;

use crate::clip::clip_of;
use crate::node::{Inputs, PortSpec, opt};

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
    Some(CompileValue::Media(MediaValue {
        src: recording.src.clone(),
        kind: MediaKind::Video,
        duration_ms: Some(recording.duration_ms),
        width: Some(recording.viewport.width),
        height: Some(recording.viewport.height),
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

pub const IN_PORTS: [&str; 6] = ["in1", "in2", "in3", "in4", "in5", "in6"];
pub const CUT_PORTS: [&str; 5] = ["in2", "in3", "in4", "in5", "in6"];

pub fn in_port_index(handle: &str) -> Option<usize> {
    IN_PORTS.iter().position(|p| *p == handle)
}

pub fn next_in_port(handle: &str) -> Option<&'static str> {
    in_port_index(handle).and_then(|i| IN_PORTS.get(i + 1).copied())
}

pub fn clip_label(i: usize) -> String {
    format!("Clip {}", i + 1)
}

pub fn layer_label(i: usize) -> String {
    let end = match i {
        0 => " (back)",
        5 => " (front)",
        _ => "",
    };
    format!("Layer {}{end}", i + 1)
}

pub fn scene_label(i: usize) -> String {
    format!("Scene {}", i + 1)
}

pub fn clip_ports(label: fn(usize) -> String) -> Vec<(&'static str, PortSpec)> {
    IN_PORTS
        .iter()
        .enumerate()
        .map(|(i, p)| (*p, opt(PortType::Clip, label(i))))
        .collect()
}

pub fn connected_clips(inputs: &Inputs) -> Vec<(&'static str, &ClipValue)> {
    IN_PORTS
        .iter()
        .filter_map(|p| clip_of(inputs.get(*p)).map(|c| (*p, c)))
        .collect()
}
