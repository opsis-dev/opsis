use std::path::{Path, PathBuf};
use std::process::Command;

use opsis_compile::nodes::output::RENDER_TYPE;
use opsis_golden::{dev_cache, ffmpeg};
use opsis_oracle::films::{EXPORT_FOLDER, ExampleRun, PROJECT_FILE, run_example};
use opsis_types::numbers::float;
use opsis_types::workflow::Workflow;
use serde_json::Value;

const ORACLE_ENV: &str = "OPSIS_ORACLE";
const FILTER_ENV: &str = "OPSIS_NATIVE_FILM";
const SAMPLE_EVERY: u32 = 30;
const TOLERANCE: f32 = 0.05;
const WRONG_PIXEL: f32 = 0.5;
const WRONG_SHARE: f64 = 0.002;

fn oracle_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os(ORACLE_ENV) {
        return PathBuf::from(dir);
    }
    let root = dev_cache("oracle");
    std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("{}: {e}. Run bun run oracle first.", root.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.join("films.json").is_file())
        .min()
        .unwrap_or_else(|| panic!("no oracle with films in {}", root.display()))
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap_or_else(|e| panic!("{}: {e}", to.display()));
    for entry in std::fs::read_dir(from).unwrap_or_else(|e| panic!("{}: {e}", from.display())) {
        let entry = entry.expect("a folder entry");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target)
                .unwrap_or_else(|e| panic!("{}: {e}", target.display()));
            let modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or_else(|e| panic!("{}: {e}", entry.path().display()));
            std::fs::File::options()
                .write(true)
                .open(&target)
                .and_then(|f| f.set_modified(modified))
                .unwrap_or_else(|e| panic!("{}: {e}", target.display()));
        }
    }
}

fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn forget_renders(project: &Path, workflow: &Workflow) {
    let last = read_json(&project.join(".opsis/last-run.json"));
    for node in workflow.nodes.iter().filter(|n| n.kind == RENDER_TYPE) {
        if let Some(hash) = last["nodes"][&node.id]["hash"].as_str() {
            let cached = project.join(".opsis/cache").join(hash);
            if cached.is_dir() {
                std::fs::remove_dir_all(&cached)
                    .unwrap_or_else(|e| panic!("{}: {e}", cached.display()));
            }
        }
    }
    for folder in [EXPORT_FOLDER, ".opsis/pictures"] {
        let dir = project.join(folder);
        if dir.is_dir() {
            std::fs::remove_dir_all(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        }
    }
}

fn picture_size(file: &Path) -> (u32, u32) {
    let probe = opsis_probe::probe(file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    let video = probe.video.expect("a picture");
    (video.width, video.height)
}

fn sampled_frames(file: &Path, (width, height): (u32, u32)) -> Vec<Vec<u8>> {
    let output = Command::new(ffmpeg())
        .args(["-v", "error", "-i"])
        .arg(file)
        .args([
            "-vf",
            &format!("select='not(mod(n\\,{SAMPLE_EVERY}))'"),
            "-fps_mode",
            "passthrough",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-",
        ])
        .output()
        .unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    assert!(
        output.status.success(),
        "{}: {}",
        file.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let size = usize::try_from(width * height * 3).expect("a frame size");
    output
        .stdout
        .chunks_exact(size)
        .map(<[u8]>::to_vec)
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct Score {
    mean: f32,
    wrong: f64,
}

fn flip(want: &[u8], got: &[u8], (width, height): (u32, u32)) -> Score {
    let a = nv_flip::FlipImageRgb8::with_data(width, height, want);
    let b = nv_flip::FlipImageRgb8::with_data(width, height, got);
    let error = nv_flip::flip(a, b, nv_flip::DEFAULT_PIXELS_PER_DEGREE);
    let mean = nv_flip::FlipPool::from_image(&error).mean();
    let values = error.to_vec();
    let wrong =
        float(values.iter().filter(|v| **v > WRONG_PIXEL).count()) / float(values.len().max(1));
    Score { mean, wrong }
}

fn compare(oracle: &Path, ours: &Path) -> Vec<String> {
    let size = picture_size(oracle);
    if picture_size(ours) != size {
        return vec![format!(
            "{}: size {:?} against {size:?}",
            ours.display(),
            picture_size(ours)
        )];
    }
    let want = sampled_frames(oracle, size);
    let got = sampled_frames(ours, size);
    let mut lines = Vec::new();
    if want.len() != got.len() {
        lines.push(format!(
            "{}: {} sampled frames against {}",
            ours.display(),
            got.len(),
            want.len()
        ));
    }
    for (index, (a, b)) in want.iter().zip(&got).enumerate() {
        let score = flip(a, b, size);
        if score.mean > TOLERANCE || score.wrong > WRONG_SHARE {
            lines.push(format!(
                "{} frame {}: {:.4} {:.5}",
                ours.display(),
                u32::try_from(index).unwrap_or(0) * SAMPLE_EVERY,
                score.mean,
                score.wrong
            ));
        }
    }
    lines
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the oracle's films (bun run oracle films), the engine and the staged FFmpeg"]
async fn every_example_film_matches_the_browser() {
    let oracle = oracle_dir();
    let filter = std::env::var(FILTER_ENV).unwrap_or_default();
    let films = read_json(&oracle.join("films.json"));
    let mut failures = Vec::new();
    let mut compared = 0;
    for film in films["films"].as_array().expect("a film list") {
        let name = film["example"].as_str().expect("an example name");
        if !filter.is_empty() && !filter.split(',').any(|f| f == name) {
            continue;
        }
        let project = dev_cache("native-films").join(name);
        if project.is_dir() {
            std::fs::remove_dir_all(&project)
                .unwrap_or_else(|e| panic!("{}: {e}", project.display()));
        }
        copy_tree(&oracle.join("projects").join(name), &project);
        let workflow: Workflow = serde_json::from_value(read_json(&project.join(PROJECT_FILE)))
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        forget_renders(&project, &workflow);
        let temp = dev_cache("native-films-temp").join(name);
        std::fs::create_dir_all(&temp).unwrap_or_else(|e| panic!("{}: {e}", temp.display()));
        let ffmpeg = ffmpeg();
        let run = ExampleRun {
            project: &project,
            temp: &temp,
            ffmpeg: &ffmpeg,
        };
        if let Err(error) = run_example(&run, workflow).await {
            failures.push(format!("{name}: {error}"));
            continue;
        }
        for file in film["files"].as_array().expect("film files") {
            let file = file.as_str().expect("a film file");
            compared += 1;
            failures.extend(compare(
                &oracle.join("films").join(name).join(file),
                &project.join(EXPORT_FOLDER).join(file),
            ));
        }
    }
    assert!(compared > 0 || !failures.is_empty(), "no film was compared");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
