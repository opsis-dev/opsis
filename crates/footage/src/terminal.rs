use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, OnceLock};

use opsis_media::grade::GradeChain;
use opsis_terminal::cast::read_path;
use opsis_terminal::colours::{Swatch, read_swatch, swatch};
use opsis_terminal::prepare::{Mask, PrepareInput, prepare};
use opsis_types::terminal::TerminalTheme;
use opsis_types::terminal_frames::{Frames, write};

use crate::FootageError;
use crate::decoder::FootageTools;

#[derive(Debug, Clone)]
pub struct TerminalFootage {
    pub cast: PathBuf,
    pub theme: TerminalTheme,
    pub masks: Vec<Mask>,
    pub fps: u32,
    pub grade: Option<GradeChain>,
}

#[derive(Debug)]
pub struct TerminalState {
    source: TerminalFootage,
    made: OnceLock<Result<Arc<Vec<u8>>, String>>,
}

pub fn swatch_args(grade: &GradeChain, swatch: &Swatch) -> Vec<String> {
    let mut args: Vec<String> = [
        "-nostdin",
        "-hide_banner",
        "-nostats",
        "-loglevel",
        "error",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgb24",
        "-s",
    ]
    .map(str::to_owned)
    .to_vec();
    args.push(format!("{}x{}", swatch.width, swatch.height));
    args.extend(["-i".to_owned(), "pipe:0".to_owned()]);
    for extra in &grade.extra_inputs {
        args.extend(["-i".to_owned(), extra.clone()]);
    }
    args.extend(
        [
            "-filter_complex",
            &format!("{};[out]format=rgb24[graded]", grade.chain),
            "-map",
            "[graded]",
            "-frames:v",
            "1",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "pipe:1",
        ]
        .map(str::to_owned),
    );
    args
}

fn graded_swatch(
    tools: &FootageTools,
    grade: &GradeChain,
    swatch: &Swatch,
) -> Result<Vec<u8>, String> {
    let mut child = Command::new(&tools.ffmpeg)
        .args(swatch_args(grade, swatch))
        .env("TMPDIR", &tools.temp)
        .env("TEMP", &tools.temp)
        .env("TMP", &tools.temp)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut stdin = child.stdin.take().ok_or("The colour grade has no input")?;
    let rgb = swatch.rgb.clone();
    let writer = std::thread::spawn(move || stdin.write_all(&rgb));
    let mut graded = Vec::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout.read_to_end(&mut graded).map_err(|e| e.to_string())?;
    }
    let mut log = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        stderr.read_to_string(&mut log).map_err(|e| e.to_string())?;
    }
    let status = child.wait().map_err(|e| e.to_string())?;
    if writer.join().is_err() {
        return Err("The colour grade stopped reading the colours".to_owned());
    }
    if !status.success() || graded.len() < swatch.rgb.len() {
        return Err(format!("The colour grade failed: {}", log.trim()));
    }
    Ok(graded)
}

fn graded(frames: &mut Frames, grade: &GradeChain, tools: &FootageTools) -> Result<(), String> {
    let colours = frames.colours();
    let swatch = swatch(&colours);
    let graded = graded_swatch(tools, grade, &swatch)?;
    frames.recolour(&read_swatch(&colours, &graded));
    Ok(())
}

impl TerminalState {
    pub const fn new(source: TerminalFootage) -> Self {
        Self {
            source,
            made: OnceLock::new(),
        }
    }

    fn make(source: &TerminalFootage, tools: &FootageTools) -> Result<Arc<Vec<u8>>, String> {
        let cast = read_path(&source.cast).map_err(|e| e.to_string())?;
        let mut frames = prepare(&PrepareInput {
            cast: &cast,
            fps: source.fps,
            theme: &source.theme,
            masks: &source.masks,
        })
        .map_err(|e| e.to_string())?;
        if let Some(grade) = &source.grade {
            graded(&mut frames, grade, tools)?;
        }
        write(&frames).map(Arc::new).map_err(|e| e.to_string())
    }

    pub fn frames(&self, tools: &FootageTools) -> Result<Arc<Vec<u8>>, FootageError> {
        self.made
            .get_or_init(|| Self::make(&self.source, tools))
            .clone()
            .map_err(FootageError::Terminal)
    }
}
