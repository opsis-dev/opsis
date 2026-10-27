use std::ffi::OsString;
use std::path::{Path, PathBuf};

use opsis_engine::paths::ShellPlatform;
use opsis_engine::temp::{ChildKind, child_temp_name, temp_env};
use opsis_render::sidecar::SidecarCommand;
use tauri::{AppHandle, Manager};

use crate::startup::StartupError;

const APP_FOLDER: &str = "Opsis";
const USER_DATA_ENV: &str = "OPSIS_USER_DATA";
const VIDEO_BUNDLE_DIR: &str = "remotion-bundle";
const PREVIEW_PAGE_DIR: &str = "apps/preview/dist";
const SIDECAR_ENTRY: &str = "apps/sidecar/src/main.ts";
const SIDECAR_PROGRAM: &str = "opsis-render";
const COMPOSITOR_DIR: &str = "compositor";

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub user_data: PathBuf,
    pub resources: PathBuf,
    pub packaged: bool,
    pub public_dir: PathBuf,
    pub preview_page: PathBuf,
    pub video_bundle: PathBuf,
    pub temp_root: PathBuf,
    pub views_temp: PathBuf,
    pub sidecar_temp: PathBuf,
    pub program_dir: PathBuf,
    pub platform: ShellPlatform,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

impl AppPaths {
    pub fn new(app: &AppHandle) -> Result<Self, StartupError> {
        let user_data = match std::env::var_os(USER_DATA_ENV) {
            Some(dir) => PathBuf::from(dir),
            None => app.path().config_dir()?.join(APP_FOLDER),
        };
        let packaged = !cfg!(debug_assertions);
        let resources = if packaged {
            app.path().resource_dir()?
        } else {
            repo_root()
        };
        let platform = ShellPlatform::current()?;
        let program = std::env::current_exe().map_err(|_| StartupError::ProgramFolder)?;
        let program_dir = program
            .parent()
            .ok_or(StartupError::ProgramFolder)?
            .to_path_buf();
        let public_dir = if packaged {
            resources.join(VIDEO_BUNDLE_DIR).join("public")
        } else {
            resources.join("public")
        };
        Ok(Self {
            public_dir,
            preview_page: resources.join(PREVIEW_PAGE_DIR),
            video_bundle: resources.join(VIDEO_BUNDLE_DIR),
            temp_root: user_data.join("tmp"),
            views_temp: user_data.join("tmp").join(child_temp_name(
                ChildKind::Shell,
                "views",
                std::process::id(),
            )),
            sidecar_temp: user_data.join("tmp").join(child_temp_name(
                ChildKind::Sidecar,
                "render",
                std::process::id(),
            )),
            user_data,
            resources,
            packaged,
            program_dir,
            platform,
        })
    }

    pub fn fonts_dir(&self) -> PathBuf {
        self.user_data.join("fonts")
    }

    pub fn database(&self) -> PathBuf {
        self.user_data.join("opsis.db")
    }

    pub fn compositor(&self) -> Option<PathBuf> {
        self.packaged.then(|| self.resources.join(COMPOSITOR_DIR))
    }

    fn sidecar_env(&self) -> Vec<(OsString, OsString)> {
        temp_env(&self.sidecar_temp)
            .into_iter()
            .map(|(key, dir)| (OsString::from(key), dir.into_os_string()))
            .collect()
    }

    pub fn sidecar(&self) -> SidecarCommand {
        if self.packaged {
            return SidecarCommand {
                program: self
                    .program_dir
                    .join(format!("{SIDECAR_PROGRAM}{}", std::env::consts::EXE_SUFFIX)),
                args: Vec::new(),
                env: self.sidecar_env(),
                cwd: Some(self.resources.clone()),
            };
        }
        SidecarCommand {
            program: PathBuf::from("bun"),
            args: vec![
                OsString::from("run"),
                self.resources.join(SIDECAR_ENTRY).into_os_string(),
            ],
            env: self.sidecar_env(),
            cwd: Some(self.resources.clone()),
        }
    }
}
