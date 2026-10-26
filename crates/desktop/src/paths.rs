use std::ffi::OsString;
use std::path::{Path, PathBuf};

use opsis_engine::paths::{EngineHome, ShellPlatform, resolve_engine_dir};
use opsis_engine::temp::{ChildKind, child_temp_name, temp_env};
use opsis_render::sidecar::SidecarCommand;
use tauri::{AppHandle, Manager};

const APP_FOLDER: &str = "Opsis";
const USER_DATA_ENV: &str = "OPSIS_USER_DATA";
const VIDEO_BUNDLE_DIR: &str = "remotion-bundle";
const PREVIEW_PAGE_DIR: &str = "apps/preview/dist";
const SIDECAR_ENTRY: &str = "apps/sidecar/src/main.ts";
const SIDECAR_PROGRAM: &str = "opsis-render";
const COMPOSITOR_DIR: &str = "compositor";
const EXPORT_SOURCES_DIR: &str = "export-src";

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

    pub fn compositor(&self) -> Option<PathBuf> {
        self.packaged.then(|| self.resources.join(COMPOSITOR_DIR))
    }

    pub fn export_sources(&self) -> PathBuf {
        if self.packaged {
            self.resources.join(EXPORT_SOURCES_DIR)
        } else {
            self.resources.clone()
        }
    }

    fn sidecar_env(&self) -> Vec<(OsString, OsString)> {
        temp_env(&self.sidecar_temp)
            .into_iter()
            .map(|(key, dir)| (OsString::from(key), dir.into_os_string()))
            .collect()
    }

    pub fn sidecar(&self) -> SidecarCommand {
        if self.packaged {
            let folder = std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(Path::to_path_buf))
                .unwrap_or_default();
            return SidecarCommand {
                program: folder.join(format!("{SIDECAR_PROGRAM}{}", std::env::consts::EXE_SUFFIX)),
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
