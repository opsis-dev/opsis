use std::path::{Path, PathBuf};

use opsis_capture::cdp::CaptureError;
use opsis_capture::film::{Capture, DriverOptions, FilmPage, FrameDriver, FrameFormat};
use opsis_compile::nodes::output::Gl;
use opsis_types::timeline::TimelineSpec;

pub const ORACLE_GL: Gl = Gl::Swangle;

#[derive(Debug, thiserror::Error)]
pub enum PictureError {
    #[error(transparent)]
    Capture(#[from] CaptureError),
    #[error("writing {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
}

#[derive(Debug)]
pub struct Drawing<'a> {
    pub shell: &'a Path,
    pub temp: &'a Path,
    pub url: &'a str,
    pub spec: &'a TimelineSpec,
}

pub async fn draw_frames(
    drawing: &Drawing<'_>,
    frames: &[(u32, PathBuf)],
) -> Result<(), PictureError> {
    let json = serde_json::to_value(drawing.spec).expect("a spec as JSON");
    let options = DriverOptions {
        executable: drawing.shell,
        temp_dir: drawing.temp,
        gl: ORACLE_GL,
        capture: Capture::for_this_computer(),
    };
    let page = FilmPage {
        url: drawing.url,
        spec: &json,
        width: drawing.spec.width,
        height: drawing.spec.height,
        scale: 1.0,
    };
    let mut driver = FrameDriver::open(&options, &page).await?;
    let drawn = write_frames(&mut driver, frames).await;
    driver.close().await;
    drawn
}

async fn write_frames(
    driver: &mut FrameDriver,
    frames: &[(u32, PathBuf)],
) -> Result<(), PictureError> {
    for (frame, path) in frames {
        let png = driver.frame(*frame, FrameFormat::Png).await?;
        std::fs::write(path, png).map_err(|source| PictureError::Write {
            path: path.clone(),
            source,
        })?;
    }
    Ok(())
}
