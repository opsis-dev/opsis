use serde::Serialize;
use specta::Type;
use specta::datatype::DataType;
use tauri::ipc::{Channel, InvokeResponseBody, IpcResponse};

pub struct Frame(Vec<u8>);

impl Frame {
    pub fn packed(header: &impl Serialize, jpeg: &[u8]) -> Option<Self> {
        let header = serde_json::to_vec(header).ok()?;
        let length = u32::try_from(header.len()).ok()?;
        let mut bytes = Vec::with_capacity(4 + header.len() + jpeg.len());
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(&header);
        bytes.extend_from_slice(jpeg);
        Some(Self(bytes))
    }
}

impl IpcResponse for Frame {
    fn body(self) -> tauri::Result<InvokeResponseBody> {
        Ok(InvokeResponseBody::Raw(self.0))
    }
}

impl Type for Frame {
    fn definition(_: &mut specta::Types) -> DataType {
        DataType::Reference(specta_typescript::define("ArrayBuffer"))
    }
}

pub type FrameChannel = Channel<Frame>;

pub fn send(channel: &FrameChannel, header: &impl Serialize, jpeg: &[u8]) -> bool {
    match Frame::packed(header, jpeg) {
        Some(frame) => channel.send(frame).is_ok(),
        None => false,
    }
}
