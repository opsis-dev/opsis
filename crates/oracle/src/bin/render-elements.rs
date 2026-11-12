use std::path::{Path, PathBuf};

use opsis_capture::film::{Capture, DriverOptions, FilmPage};
use opsis_compile::asset_ref::FILM_FOLDER;
use opsis_compile::elements::{ElementMeta, elements};
use opsis_compile::nodes::output::{Gl, StillFormat};
use opsis_oracle::elements::{ElementError, element_spec, frame_at};
use opsis_oracle::host::{film_page, headless_shell, home, serve};
use opsis_render::frames::{FilmError, StillRender, render_still};

const STILL_WIDTH: f64 = 480.0;
const JPEG_QUALITY: f64 = 82.0;
const DEFAULT_POSITION: f64 = 0.5;

#[derive(Debug, thiserror::Error)]
enum StillError {
    #[error(transparent)]
    Element(#[from] ElementError),
    #[error(transparent)]
    Render(#[from] FilmError),
}

struct Host {
    shell: PathBuf,
    temp: PathBuf,
    project_base: String,
    out: PathBuf,
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

async fn stills_for(meta: &ElementMeta, positions: &[f64], host: &Host) -> Result<(), StillError> {
    let spec = element_spec(meta, &host.project_base)?;
    let json = serde_json::to_value(&spec).expect("a spec as JSON");
    let url = format!("{}{FILM_FOLDER}/index.html", host.project_base);
    for &at in positions {
        render_still(&StillRender {
            driver: DriverOptions {
                executable: &host.shell,
                temp_dir: &host.temp,
                gl: Gl::Angle,
                capture: Capture::for_this_computer(),
            },
            page: FilmPage {
                url: &url,
                spec: &json,
                width: spec.width,
                height: spec.height,
                scale: (STILL_WIDTH / f64::from(spec.width)).min(1.0),
            },
            frame: frame_at(spec.duration_in_frames, at),
            format: StillFormat::Jpeg,
            jpeg_quality: Some(JPEG_QUALITY),
            output: &still_file(&host.out, &meta.key, at, positions.len() > 1),
        })
        .await?;
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut args = std::env::args().skip(1);
    let only = args.next().unwrap_or_default();
    let positions = positions(args.next());
    let out = home().join(".cache/opsis-dev/element-stills");
    let temp = out.join("tmp");
    let project = temp.join("project");
    std::fs::create_dir_all(&project).expect("the output folder");
    let server = serve(&project, film_page()).await;
    let host = Host {
        shell: headless_shell(),
        temp: temp.clone(),
        project_base: server.url.clone(),
        out: out.clone(),
    };
    let chosen: Vec<&ElementMeta> = elements()
        .iter()
        .filter(|m| m.key.contains(&only))
        .collect();
    let mut failures = Vec::new();
    for meta in &chosen {
        match stills_for(meta, &positions, &host).await {
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
