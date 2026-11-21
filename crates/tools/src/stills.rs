use std::path::{Path, PathBuf};
use std::sync::Arc;

use opsis_compile::elements::{ElementMeta, elements};
use opsis_compile::nodes::output::StillFormat;
use opsis_engine::ffmpeg::bundled_ffmpeg;
use opsis_film::Drawing;
use opsis_footage::decoder::FootageTools;
use opsis_footage::footage::Footage;
use opsis_prepare::serve::PreparedDirs;
use opsis_render::element_spec::{element_spec, frame_at};
use opsis_render::native::{NATIVE_BASE, NativeFiles, NativeStill, render_native_still};

use crate::error::ToolError;
use crate::output;

const STILLS_CACHE: &str = "element-stills";
const STILL_WIDTH: f64 = 480.0;
const JPEG_QUALITY: f64 = 82.0;
const PUBLIC_DIR: &str = "public";

fn still_file(out: &Path, key: &str, at: f64, several: bool) -> PathBuf {
    if several {
        out.join(format!("{key}@{at}.jpg"))
    } else {
        out.join(format!("{key}.jpg"))
    }
}

fn native_files(temp: &Path) -> Result<Arc<NativeFiles>, ToolError> {
    let project = temp.join("project");
    output::folder(&project)?;
    Ok(Arc::new(NativeFiles {
        dirs: PreparedDirs {
            project,
            public: output::repository().join(PUBLIC_DIR),
        },
        footage: Arc::new(Footage::new(
            FootageTools {
                ffmpeg: bundled_ffmpeg().unwrap_or_default(),
                temp: temp.to_path_buf(),
            },
            Vec::new(),
        )),
        drawing: Drawing {
            temp: temp.join("drawing"),
            ..Drawing::default()
        },
    }))
}

async fn stills_for(
    meta: &ElementMeta,
    at: &[f64],
    files: &Arc<NativeFiles>,
    out: &Path,
) -> Result<(), String> {
    let spec = element_spec(meta, NATIVE_BASE).map_err(|e| e.to_string())?;
    for &share in at {
        render_native_still(NativeStill {
            frame: frame_at(spec.duration_in_frames, share),
            scale: (STILL_WIDTH / f64::from(spec.width)).min(1.0),
            spec: spec.clone(),
            files: Arc::clone(files),
            format: StillFormat::Jpeg,
            jpeg_quality: Some(JPEG_QUALITY),
            output: still_file(out, &meta.key, share, at.len() > 1),
        })
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn render(filter: &str, at: &[f64]) -> Result<(), ToolError> {
    let out = opsis_golden::dev_cache(STILLS_CACHE);
    let temp = out.join("tmp");
    let files = native_files(&temp)?;
    let chosen: Vec<&ElementMeta> = elements()
        .iter()
        .filter(|meta| meta.key.contains(filter))
        .collect();
    let mut failures = Vec::new();
    for meta in &chosen {
        match stills_for(meta, at, &files, &out).await {
            Ok(()) => println!("ok   {}", meta.key),
            Err(error) => {
                println!("FAIL {}", meta.key);
                failures.push(format!("{}: {error}", meta.key));
            }
        }
    }
    println!(
        "\n{} of {} rendered into {}",
        chosen.len() - failures.len(),
        chosen.len(),
        out.display()
    );
    if let Err(error) = std::fs::remove_dir_all(&temp) {
        eprintln!("couldn't remove {}: {error}", temp.display());
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(ToolError::Stills {
            failed: failures.len(),
            total: chosen.len(),
            list: failures.join("\n"),
        })
    }
}
