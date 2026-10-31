use std::path::{Path, PathBuf};

use opsis_engine::paths::ShellPlatform;
use opsis_engine::temp::{ChildKind, child_temp_name};
use tauri::{AppHandle, Manager};

use crate::startup::StartupError;

const APP_FOLDER: &str = "Opsis";
const USER_DATA_ENV: &str = "OPSIS_USER_DATA";
const FILM_BUNDLE_DIR: &str = "film-bundle";

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub user_data: PathBuf,
    pub resources: PathBuf,
    pub packaged: bool,
    pub public_dir: PathBuf,
    pub film_bundle: PathBuf,
    pub temp_root: PathBuf,
    pub views_temp: PathBuf,
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
        let public_dir = if packaged {
            resources.join(FILM_BUNDLE_DIR)
        } else {
            resources.join("public")
        };
        Ok(Self {
            public_dir,
            film_bundle: resources.join(FILM_BUNDLE_DIR),
            temp_root: user_data.join("tmp"),
            views_temp: user_data.join("tmp").join(child_temp_name(
                ChildKind::Shell,
                "views",
                std::process::id(),
            )),
            user_data,
            resources,
            packaged,
            platform,
        })
    }

    pub fn fonts_dir(&self) -> PathBuf {
        self.user_data.join("fonts")
    }

    pub fn database(&self) -> PathBuf {
        self.user_data.join("opsis.db")
    }
}
