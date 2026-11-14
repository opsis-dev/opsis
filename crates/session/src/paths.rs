use std::path::{Path, PathBuf};

use opsis_engine::paths::ShellPlatform;
use opsis_engine::temp::{ChildKind, child_temp_name};

use crate::startup::StartupError;

const APP_FOLDER: &str = "Opsis";
const USER_DATA_ENV: &str = "OPSIS_USER_DATA";
const PUBLIC_DIR: &str = "public";

#[derive(Debug, Clone)]
pub struct HostFolders {
    pub config: PathBuf,
    pub resources: PathBuf,
    pub home: PathBuf,
}

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub user_data: PathBuf,
    pub home: PathBuf,
    pub resources: PathBuf,
    pub packaged: bool,
    pub public_dir: PathBuf,
    pub temp_root: PathBuf,
    pub views_temp: PathBuf,
    pub platform: ShellPlatform,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

impl AppPaths {
    pub fn new(folders: &HostFolders) -> Result<Self, StartupError> {
        let user_data = match std::env::var_os(USER_DATA_ENV) {
            Some(dir) => PathBuf::from(dir),
            None => folders.config.join(APP_FOLDER),
        };
        let packaged = !cfg!(debug_assertions);
        let resources = if packaged {
            folders.resources.clone()
        } else {
            repo_root()
        };
        let platform = ShellPlatform::current()?;
        let public_dir = resources.join(PUBLIC_DIR);
        Ok(Self {
            public_dir,
            temp_root: user_data.join("tmp"),
            views_temp: user_data.join("tmp").join(child_temp_name(
                ChildKind::Shell,
                "views",
                std::process::id(),
            )),
            user_data,
            home: folders.home.clone(),
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
