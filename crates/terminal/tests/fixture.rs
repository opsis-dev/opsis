use std::path::PathBuf;

use opsis_compile::terminal::default_theme;
use opsis_terminal::cast::{read_path, write_v3};
use opsis_terminal::prepare::{PrepareInput, prepare};
use opsis_terminal::take::{TakeOptions, record_take};
use opsis_types::terminal::{
    EnvVar, ShellChoice, TerminalAction, TerminalSessionValue, TerminalStep, WaitFor,
};
use opsis_types::terminal_frames::{Frames, Redaction, write};
use serde_json::{Value, json};

const CAST: &str = "terminal/take.cast";
const FRAMES: &str = "terminal/take.frames";
const SUMMARY: &str = "terminal/take.summary.json";
const FPS: u32 = 30;

fn sandbox() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/sandbox")
}

fn step(id: &str, action: TerminalAction) -> TerminalStep {
    TerminalStep {
        id: id.to_owned(),
        node_id: id.to_owned(),
        label: id.to_owned(),
        action,
        wait_after_ms: 300,
    }
}

fn run(command: &str) -> TerminalAction {
    TerminalAction::Run {
        command: command.to_owned(),
        wait: WaitFor::Prompt,
        limit_ms: 20_000,
        zoom_output: true,
    }
}

#[test]
#[ignore = "needs a shell; records fixtures/terminal/take.cast again"]
fn records_the_fixture_take() {
    let value = TerminalSessionValue {
        shell: ShellChoice::Clean,
        folder: String::new(),
        cols: 80,
        rows: 20,
        env: Vec::new(),
        typing_ms: 30,
        pointer_ms: 300,
        steps: vec![
            step("hide", TerminalAction::Hide),
            step("cd", run("cd sandbox && clear")),
            step("show", TerminalAction::Show),
            step("start", TerminalAction::StartRecording),
            step("spinner", run("./spinner.sh")),
            step(
                "chapter",
                TerminalAction::Chapter {
                    title: "Status".to_owned(),
                },
            ),
            step("table", run("./table.sh")),
            step("sync", run("./sync.sh")),
            step("wide", run("echo '日本語 ✔ ✗ 🚀 → λ ★ ❯'")),
        ],
    };
    let home = tempfile::tempdir().unwrap();
    let copy = home.path().join("sandbox");
    std::fs::create_dir_all(&copy).unwrap();
    for entry in std::fs::read_dir(sandbox()).unwrap() {
        let entry = entry.unwrap();
        std::fs::copy(entry.path(), copy.join(entry.file_name())).unwrap();
    }
    let value = TerminalSessionValue {
        env: vec![EnvVar {
            name: "HOME".to_owned(),
            value: home.path().to_string_lossy().into_owned(),
        }],
        ..value
    };
    let project = home.path().to_path_buf();
    let temp = tempfile::tempdir().unwrap();
    let mut log = |line: String| eprintln!("{line}");
    let take = record_take(&mut TakeOptions {
        session: &value,
        project_dir: &project,
        temp: temp.path(),
        name: "Fixture",
        pause_before_ms: 0,
        keep_after_ms: 500,
        cancelled: &|| false,
        log: &mut log,
        live: None,
    })
    .unwrap();
    if opsis_golden::writing() {
        let mut header = take.header.clone();
        header.timestamp = None;
        std::fs::write(opsis_golden::path(CAST), write_v3(&header, &take.events)).unwrap();
    }
}

fn summary(frames: &Frames) -> Value {
    let text_of = |index: usize| -> Vec<String> {
        frames.frames[index]
            .row_indices
            .iter()
            .map(|&i| {
                let row = &frames.row_table[usize::try_from(i).unwrap()];
                let mut line = String::new();
                let mut at = 0;
                for run in row {
                    let col = usize::from(run.col);
                    line.push_str(&" ".repeat(col.saturating_sub(at)));
                    line.push_str(&run.text);
                    at = col + usize::from(run.cells);
                }
                line.trim_end().to_owned()
            })
            .collect()
    };
    let last = frames.frames.len() - 1;
    let mut distinct: Vec<&[u32]> = Vec::new();
    for frame in &frames.frames {
        if distinct.last() != Some(&frame.row_indices.as_slice()) {
            distinct.push(&frame.row_indices);
        }
    }
    let colours: std::collections::BTreeSet<String> = frames
        .row_table
        .iter()
        .flatten()
        .flat_map(|r| [r.fg, r.bg])
        .map(|c| format!("#{c:06x}"))
        .collect();
    json!({
        "size": [frames.cols, frames.rows],
        "fps": frames.fps,
        "frames": frames.frames.len(),
        "rows": frames.row_table.len(),
        "changes": distinct.len(),
        "redacted": frames.row_table.iter().flatten().filter(|r| r.redaction != Redaction::None).count(),
        "colours": colours,
        "first": text_of(0),
        "middle": text_of(last / 2),
        "last": text_of(last),
    })
}

#[test]
fn prepares_the_fixture_take_as_recorded() {
    let cast = read_path(opsis_golden::path(CAST)).unwrap();
    let theme = default_theme();
    let frames = prepare(&PrepareInput {
        cast: &cast,
        fps: FPS,
        theme: &theme,
        masks: &[],
    })
    .unwrap();
    let bytes = write(&frames).unwrap();
    opsis_golden::check(SUMMARY, &summary(&frames));
    let stored = opsis_golden::path(FRAMES);
    if opsis_golden::writing() {
        std::fs::write(&stored, &bytes).unwrap();
    }
    assert!(
        std::fs::read(&stored).unwrap() == bytes,
        "the frames file differs from {FRAMES}"
    );
}
