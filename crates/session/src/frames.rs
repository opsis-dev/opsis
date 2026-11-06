use serde::Serialize;

pub use crate::platform::FrameChannel;

pub fn packed(header: &impl Serialize, jpeg: &[u8]) -> Option<Vec<u8>> {
    let header = serde_json::to_vec(header).ok()?;
    let length = u32::try_from(header.len()).ok()?;
    let mut bytes = Vec::with_capacity(4 + header.len() + jpeg.len());
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(jpeg);
    Some(bytes)
}

pub fn send(channel: &FrameChannel, header: &impl Serialize, jpeg: &[u8]) -> bool {
    packed(header, jpeg).is_some_and(|frame| channel.send(frame))
}
