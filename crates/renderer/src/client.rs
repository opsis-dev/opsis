use std::collections::HashSet;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use opsis_scene::node::Stage3d;

use crate::protocol::{ProtocolError, Reply, Request, Setting, receive, send};

const PROGRAM: &str = "opsis-renderer";
const PROGRAM_ENV: &str = "OPSIS_RENDERER";
const DRIVER_FILES_ENV: &str = "VK_DRIVER_FILES";
const ICD_FILE: &str = "software-vulkan.json";
const SEARCHED_FOLDERS: usize = 5;
const TEMP_ENV: [&str; 3] = ["TMPDIR", "TEMP", "TMP"];

static STARTED: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Driver {
    Card,
    Software { library: PathBuf },
}

#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("the drawing program couldn't be found")]
    Missing,
    #[error("the drawing program couldn't start: {0}")]
    Start(std::io::Error),
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
    #[error("{0}")]
    Failed(String),
}

pub fn renderer_program() -> Option<PathBuf> {
    if cfg!(debug_assertions)
        && let Some(path) = std::env::var_os(PROGRAM_ENV)
    {
        return Some(PathBuf::from(path));
    }
    let name = format!("{PROGRAM}{}", std::env::consts::EXE_SUFFIX);
    let exe = std::env::current_exe().ok()?;
    exe.ancestors()
        .skip(1)
        .take(SEARCHED_FOLDERS)
        .map(|dir| dir.join(&name))
        .find(|candidate| candidate.is_file())
}

#[derive(Clone, Copy)]
pub struct StageInputs<'a> {
    pub stage: &'a Stage3d,
    pub size: (u32, u32),
    pub pictures: &'a [Option<(&'a [u8], u32, u32)>],
    pub files: &'a dyn Fn(&str) -> Option<Vec<u8>>,
    pub environment: Option<(&'a [u8], &'a [f32])>,
}

impl std::fmt::Debug for StageInputs<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StageInputs")
            .field("stage", &self.stage)
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct Renderer {
    child: Child,
    input: Option<BufWriter<ChildStdin>>,
    output: BufReader<ChildStdout>,
    models: HashSet<String>,
    environment: bool,
}

fn icd_json(library: &Path) -> String {
    serde_json::json!({
        "file_format_version": "1.0.0",
        "ICD": { "library_path": library, "api_version": "1.0.5" }
    })
    .to_string()
}

impl Renderer {
    pub fn start(program: &Path, driver: &Driver, temp: &Path) -> Result<Self, RendererError> {
        let mut command = Command::new(program);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        let started = STARTED.fetch_add(1, Ordering::Relaxed);
        if !temp.as_os_str().is_empty() {
            let own = temp.join(format!("{PROGRAM}-{}-{started}", std::process::id()));
            std::fs::create_dir_all(&own).map_err(RendererError::Start)?;
            for name in TEMP_ENV {
                command.env(name, &own);
            }
        }
        if let Driver::Software { library } = driver {
            std::fs::create_dir_all(temp).map_err(RendererError::Start)?;
            let icd = temp.join(ICD_FILE);
            let written = temp.join(format!("{ICD_FILE}.{}.{started}", std::process::id()));
            std::fs::write(&written, icd_json(library)).map_err(RendererError::Start)?;
            std::fs::rename(&written, &icd).map_err(RendererError::Start)?;
            command.env(DRIVER_FILES_ENV, icd);
        }
        let mut child = command.spawn().map_err(RendererError::Start)?;
        let (Some(input), Some(output)) = (child.stdin.take(), child.stdout.take()) else {
            return Err(RendererError::Missing);
        };
        Ok(Self {
            child,
            input: Some(BufWriter::new(input)),
            output: BufReader::new(output),
            models: HashSet::new(),
            environment: false,
        })
    }

    fn ask(&mut self, request: &Request, blobs: &[&[u8]]) -> Result<Vec<u8>, RendererError> {
        let input = self.input.as_mut().ok_or(ProtocolError::Closed)?;
        send(input, request, blobs)?;
        let (reply, mut drawn) = receive::<Reply>(&mut self.output)?;
        match reply {
            Reply::Drawn => drawn
                .pop()
                .ok_or(RendererError::Protocol(ProtocolError::Closed)),
            Reply::Failed(message) => Err(RendererError::Failed(message)),
        }
    }

    pub fn run(
        &mut self,
        program: &str,
        inputs: &[&[u8]],
        size: (u32, u32),
        settings: &[Setting],
    ) -> Result<Vec<u8>, RendererError> {
        self.ask(
            &Request::Run {
                program: program.to_owned(),
                size,
                settings: settings.to_vec(),
            },
            inputs,
        )
    }

    pub fn stage(&mut self, inputs: &StageInputs<'_>) -> Result<Vec<u8>, RendererError> {
        let mut blobs: Vec<Vec<u8>> = Vec::new();
        let mut owned: Vec<&[u8]> = inputs
            .pictures
            .iter()
            .filter_map(|p| p.map(|(rgba, _, _)| rgba))
            .collect();
        let mut sent_models = Vec::new();
        for model in &inputs.stage.models {
            let source = &model.source.0;
            if self.models.contains(source) || sent_models.contains(source) {
                continue;
            }
            if let Some(bytes) = (inputs.files)(source) {
                sent_models.push(source.clone());
                blobs.push(bytes);
            }
        }
        let environment = !self.environment && inputs.environment.is_some();
        if environment && let Some((_, irradiance)) = inputs.environment {
            blobs.push(irradiance.iter().flat_map(|f| f.to_le_bytes()).collect());
        }
        let mut model_blobs: Vec<&[u8]> = blobs.iter().map(Vec::as_slice).collect();
        let irradiance = if environment { model_blobs.pop() } else { None };
        owned.append(&mut model_blobs);
        if let (true, Some((reflections, _)), Some(irradiance)) =
            (environment, inputs.environment, irradiance)
        {
            owned.push(reflections);
            owned.push(irradiance);
        }
        let request = Request::Stage {
            stage: Box::new(inputs.stage.clone()),
            size: inputs.size,
            pictures: inputs
                .pictures
                .iter()
                .map(|p| p.map(|(_, w, h)| (w, h)))
                .collect(),
            models: sent_models.clone(),
            environment,
        };
        let drawn = self.ask(&request, &owned)?;
        self.models.extend(sent_models);
        self.environment |= environment;
        Ok(drawn)
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.input.take();
        if let Err(error) = self.child.wait() {
            log::warn!("waiting for the drawing program: {error}");
        }
    }
}
