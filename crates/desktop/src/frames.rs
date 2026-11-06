use std::fmt;
use std::sync::Arc;

use specta::Type;
use specta::datatype::DataType;
use tauri::ipc::{Channel, InvokeResponseBody, IpcResponse};

pub struct Frame(Vec<u8>);

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

struct ChannelSink(FrameChannel);

impl fmt::Debug for ChannelSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ChannelSink").field(&self.0.id()).finish()
    }
}

impl opsis_session::platform::FrameSender for ChannelSink {
    fn id(&self) -> u32 {
        self.0.id()
    }

    fn send(&self, frame: Vec<u8>) -> bool {
        self.0.send(Frame(frame)).is_ok()
    }
}

pub fn sink(channel: FrameChannel) -> opsis_session::frames::FrameChannel {
    Arc::new(ChannelSink(channel))
}
