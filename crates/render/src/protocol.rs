use opsis_compile::nodes::media::Codec;
use opsis_compile::nodes::output::{Gl, StillFormat};
use opsis_types::JsonObject;
use opsis_types::timeline::TimelineSpec;
use opsis_types::ts_export::render_plain;
use serde::{Deserialize, Serialize};
use specta::{Type, Types};

pub const PROTOCOL_FILE: &str = "apps/sidecar/src/generated/protocol.ts";
pub const COMPOSITION_ID: &str = "DemoTimeline";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RenderTarget {
    pub serve_url: String,
    pub browser_executable: String,
    #[serde(deserialize_with = "Option::deserialize")]
    #[specta(type = Option<String>)]
    pub binaries_directory: Option<String>,
    pub license_key: String,
    pub composition: String,
    #[specta(type = JsonObject)]
    pub spec: TimelineSpec,
    pub gl: Gl,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MediaJob {
    #[serde(flatten)]
    pub target: RenderTarget,
    pub output: String,
    pub codec: Codec,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crf: Option<f64>,
    pub scale: f64,
    pub muted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StillJob {
    #[serde(flatten)]
    pub target: RenderTarget,
    pub frame: u32,
    pub output: String,
    pub format: StillFormat,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jpeg_quality: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct CancelJob {
    pub target: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(tag = "method", content = "params")]
pub enum SidecarCall {
    #[serde(rename = "render.media")]
    RenderMedia(MediaJob),
    #[serde(rename = "render.still")]
    RenderStill(StillJob),
    #[serde(rename = "cancel")]
    Cancel(CancelJob),
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SidecarRequest {
    pub id: u32,
    #[serde(flatten)]
    pub call: SidecarCall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum ProgressMethod {
    Progress,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
pub struct JobProgress {
    pub id: u32,
    pub progress: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SidecarFailure {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(untagged)]
pub enum SidecarMessage {
    Progress {
        method: ProgressMethod,
        params: JobProgress,
    },
    Failed {
        id: u32,
        error: SidecarFailure,
    },
    Done {
        id: u32,
        result: (),
    },
}

pub fn protocol_types() -> String {
    render_plain(
        &Types::default()
            .register::<SidecarRequest>()
            .register::<SidecarMessage>(),
    )
}
