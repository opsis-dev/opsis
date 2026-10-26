use std::ffi::OsString;
use std::path::{Path, PathBuf};

use opsis_engine::paths::{EngineHome, ShellPlatform, resolve_engine_dir};
use opsis_render::sidecar::SidecarCommand;
use tauri::{AppHandle, Manager};

const APP_FOLDER: &str = "Opsis";
const USER_DATA_ENV: &str = "OPSIS_USER_DATA";
const VIDEO_BUNDLE_DIR: &str = "remotion-bundle";
const PREVIEW_PAGE_DIR: &str = "apps/preview/dist";
const SIDECAR_ENTRY: &str = "apps/sidecar/src/main.ts";

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub user_data: PathBuf,
    pub resources: PathBuf,
    pub packaged: bool,
    pub public_dir: PathBuf,
    pub preview_page: PathBuf,
    pub video_bundle: PathBuf,
    pub temp_root: PathBuf,
    pub platform: ShellPlatform,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

impl AppPaths {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let user_data = match std::env::var_os(USER_DATA_ENV) {
            Some(dir) => PathBuf::from(dir),
            None => app
                .path()
                .config_dir()
                .map_err(|e| e.to_string())?
                .join(APP_FOLDER),
        };
        let packaged = !cfg!(debug_assertions);
        let resources = if packaged {
            app.path().resource_dir().map_err(|e| e.to_string())?
        } else {
            repo_root()
        };
        let platform = ShellPlatform::current().map_err(|e| e.to_string())?;
        Ok(Self {
            public_dir: resources.join("public"),
            preview_page: resources.join(PREVIEW_PAGE_DIR),
            video_bundle: resources.join(VIDEO_BUNDLE_DIR),
            temp_root: user_data.join("tmp"),
            user_data,
            resources,
            packaged,
            platform,
        })
    }

    pub fn engine_dir(&self) -> PathBuf {
        resolve_engine_dir(
            &EngineHome {
                packaged: self.packaged,
                resources: &self.resources,
                user_data: &self.user_data,
            },
            self.platform,
        )
    }

    pub fn fonts_dir(&self) -> PathBuf {
        self.user_data.join("fonts")
    }

    pub fn database(&self) -> PathBuf {
        self.user_data.join("opsis.db")
    }

    pub fn sidecar(&self) -> SidecarCommand {
        SidecarCommand {
            program: PathBuf::from("bun"),
            args: vec![
                OsString::from("run"),
                self.resources.join(SIDECAR_ENTRY).into_os_string(),
            ],
            env: Vec::new(),
            cwd: Some(self.resources.clone()),
        }
    }
}
