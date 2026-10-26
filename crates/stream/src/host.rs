use std::path::PathBuf;
use std::sync::Arc;

use opsis_types::actions::Viewport;
use tokio_util::sync::CancellationToken;

use crate::view::{StreamView, ViewSize};

pub const WINDOW_SIZE: Viewport = Viewport {
    width: 1280.0,
    height: 860.0,
};
pub const VIEW_QUALITY: i64 = 80;

pub trait ViewHost: Send + Sync {
    fn show(&self, title: &str, view: Arc<StreamView>, closed: CancellationToken);
    fn hide(&self, view: &Arc<StreamView>);
}

#[derive(Debug, Clone)]
pub struct ViewEnv {
    pub executable: PathBuf,
    pub temp_dir: PathBuf,
}

impl ViewEnv {
    pub async fn open(
        &self,
        url: &str,
        viewport: Option<Viewport>,
    ) -> opsis_capture::cdp::Result<Arc<StreamView>> {
        let size = viewport.unwrap_or(WINDOW_SIZE);
        StreamView::open(
            &self.executable,
            &self.temp_dir,
            url,
            ViewSize {
                width: size.width,
                height: size.height,
                dpr: 1.0,
            },
            VIEW_QUALITY,
            &[],
        )
        .await
    }
}
