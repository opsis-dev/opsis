use std::path::{Path, PathBuf};

use axum::Router;
use opsis_compile::asset_ref::FILM_FOLDER;
use opsis_engine::http_files::{FileServer, folder_routes};
use opsis_engine::paths::{ShellPlatform, engine_paths, find_headless_shell};
use opsis_prepare::serve::{PreparedDirs, prepared_routes};

pub const ORACLE_FOLDER: &str = ".opsis/oracle";

pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .expect("a home folder")
}

pub fn user_data() -> PathBuf {
    if let Some(dir) = std::env::var_os("OPSIS_USER_DATA") {
        return PathBuf::from(dir);
    }
    let config = if cfg!(target_os = "macos") {
        home().join("Library/Application Support")
    } else if cfg!(windows) {
        std::env::var_os("APPDATA").map_or_else(|| home().join("AppData/Roaming"), PathBuf::from)
    } else {
        std::env::var_os("XDG_CONFIG_HOME").map_or_else(|| home().join(".config"), PathBuf::from)
    };
    config.join("Opsis")
}

pub fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn engine_dir() -> PathBuf {
    user_data().join("engine")
}

pub fn headless_shell() -> PathBuf {
    let engine = engine_paths(
        &engine_dir(),
        ShellPlatform::current().expect("a supported platform"),
    );
    find_headless_shell(&engine)
        .expect("No engine in the app data. Set it up from Settings → Engine first.")
        .to_path_buf()
}

pub fn bundle(name: &str, script: &str) -> PathBuf {
    let folder = repo().join(name);
    assert!(
        folder.join("index.html").exists(),
        "There's no {name}. Run `bun run {script}` first."
    );
    folder
}

pub fn film_bundle() -> PathBuf {
    bundle("film-bundle", "film:bundle")
}

pub fn oracle_bundle() -> PathBuf {
    bundle("oracle-bundle", "oracle:bundle")
}

pub fn film_page() -> Router {
    folder_routes(FILM_FOLDER, &film_bundle())
}

pub fn oracle_page() -> Router {
    folder_routes(ORACLE_FOLDER, &oracle_bundle())
}

pub async fn serve(project: &Path, pages: Router) -> FileServer {
    let dirs = PreparedDirs {
        project: project.to_path_buf(),
        public: repo().join("public"),
    };
    FileServer::start_with(project, prepared_routes(dirs).merge(pages))
        .await
        .expect("a file server for the pages and prepared files")
}
