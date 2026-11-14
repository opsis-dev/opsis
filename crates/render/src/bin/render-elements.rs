use std::path::{Path, PathBuf};
use std::sync::Arc;

use opsis_compile::elements::{ElementMeta, elements};
use opsis_compile::nodes::output::StillFormat;
use opsis_engine::ffmpeg::bundled_ffmpeg;
use opsis_footage::decoder::FootageTools;
use opsis_footage::footage::Footage;
use opsis_prepare::serve::PreparedDirs;
use opsis_render::element_spec::{ElementError, element_spec, frame_at};
use opsis_render::frames::FilmError;
use opsis_render::native::{NATIVE_BASE, NativeFiles, NativeStill, render_native_still};

const STILL_WIDTH: f64 = 480.0;
const JPEG_QUALITY: f64 = 82.0;
const DEFAULT_POSITION: f64 = 0.5;
const PUBLIC_DIR: &str = "public";

#[derive(Debug, thiserror::Error)]
enum StillError {
    #[error(transparent)]
    Element(#[from] ElementError),
    #[error(transparent)]
    Render(#[from] FilmError),
}

fn positions(arg: Option<String>) -> Vec<f64> {
    arg.map_or_else(
        || vec![DEFAULT_POSITION],
        |list| {
            list.split(',')
                .filter_map(|p| p.trim().parse().ok())
                .collect()
        },
    )
}

fn still_file(out: &Path, key: &str, at: f64, several: bool) -> PathBuf {
    if several {
        out.join(format!("{key}@{at}.jpg"))
    } else {
        out.join(format!("{key}.jpg"))
    }
}

async fn stills_for(
    meta: &ElementMeta,
    positions: &[f64],
    files: &Arc<NativeFiles>,
    out: &Path,
) -> Result<(), StillError> {
    let spec = element_spec(meta, NATIVE_BASE)?;
    for &at in positions {
        render_native_still(NativeStill {
            frame: frame_at(spec.duration_in_frames, at),
            scale: (STILL_WIDTH / f64::from(spec.width)).min(1.0),
            spec: spec.clone(),
            files: Arc::clone(files),
            format: StillFormat::Jpeg,
            jpeg_quality: Some(JPEG_QUALITY),
            output: still_file(out, &meta.key, at, positions.len() > 1),
        })
        .await?;
    }
    Ok(())
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
        .expect("the repository folder")
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut args = std::env::args().skip(1);
    let only = args.next().unwrap_or_default();
    let positions = positions(args.next());
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .expect("a home folder");
    let out = home.join(".cache/opsis-dev/element-stills");
    let temp = out.join("tmp");
    let project = temp.join("project");
    std::fs::create_dir_all(&project).expect("the output folder");
    let files = Arc::new(NativeFiles {
        dirs: PreparedDirs {
            project,
            public: repo_root().join(PUBLIC_DIR),
        },
        footage: Arc::new(Footage::new(
            FootageTools {
                ffmpeg: bundled_ffmpeg().unwrap_or_default(),
                temp: temp.clone(),
            },
            Vec::new(),
        )),
        drawing: opsis_film::Drawing {
            temp: temp.join("drawing"),
            ..opsis_film::Drawing::default()
        },
    });
    let chosen: Vec<&ElementMeta> = elements()
        .iter()
        .filter(|m| m.key.contains(&only))
        .collect();
    let mut failures = Vec::new();
    for meta in &chosen {
        match stills_for(meta, &positions, &files, &out).await {
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
    if !failures.is_empty() {
        eprintln!("{}", failures.join("\n"));
        std::process::exit(1);
    }
}
