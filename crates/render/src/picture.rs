use std::path::{Path, PathBuf};
use std::time::SystemTime;

use opsis_compile::nodes::media::{Codec, EncodePreset};
use opsis_compile::nodes::output::{Gl, PixelFormat, RenderProres};
use opsis_engine::files::{Prune, prune_dir, write_via_temp};
use opsis_engine::hash::versioned_hash;
use opsis_engine::versions::{PICTURE_VERSION, PREPARED_VERSION};
use opsis_project::paths::project_paths;
use opsis_runner::keys::ENGINE_VERSION;
use opsis_types::timeline::TimelineSpec;
use serde::{Deserialize, Serialize};
use serde_json::json;

const KEEP_PICTURES: usize = 12;
const HASH_LENGTH: usize = 24;
const SOUND_AND_ADDRESS: [&str; 3] = ["audio", "mix", "assets"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PictureOptions {
    pub codec: Codec,
    pub crf: f64,
    pub scale: f64,
    pub gl: Gl,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x264_preset: Option<EncodePreset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pixel_format: Option<PixelFormat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pro_res_profile: Option<RenderProres>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number_of_gif_loops: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub every_nth_frame: Option<u32>,
}

pub fn picture_key(spec: &TimelineSpec, opts: &PictureOptions, app_version: &str) -> String {
    let mut picture = serde_json::to_value(spec).expect("spec serializes");
    if let Some(fields) = picture.as_object_mut() {
        for key in SOUND_AND_ADDRESS {
            fields.remove(key);
        }
    }
    versioned_hash(
        PICTURE_VERSION,
        &json!({ "engine": ENGINE_VERSION, "prepared": PREPARED_VERSION, "app": app_version, "picture": picture, "opts": opts }),
        HASH_LENGTH,
    )
}

pub struct Picture {
    pub file: PathBuf,
    pub reused: bool,
}

fn touch(file: &Path) {
    let touched = std::fs::File::options()
        .write(true)
        .open(file)
        .and_then(|f| f.set_modified(SystemTime::now()));
    if let Err(error) = touched {
        log::warn!("couldn't mark {} as used: {error}", file.display());
    }
}

pub async fn cached_picture<E, F, Fut>(
    project_dir: &Path,
    name: &str,
    render: F,
) -> Result<Picture, E>
where
    E: From<std::io::Error>,
    F: FnOnce(PathBuf) -> Fut,
    Fut: Future<Output = Result<(), E>>,
{
    let dir = project_paths(project_dir).pictures;
    tokio::fs::create_dir_all(&dir).await?;
    let file = dir.join(name);
    if file.exists() {
        touch(&file);
        return Ok(Picture { file, reused: true });
    }
    write_via_temp(&file, render).await?;
    prune_dir(
        &dir,
        Prune {
            newest: KEEP_PICTURES,
            ..Prune::default()
        },
    )?;
    Ok(Picture {
        file,
        reused: false,
    })
}
