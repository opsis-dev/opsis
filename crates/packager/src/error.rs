use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum PackagerError {
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{0} is missing")]
    Missing(PathBuf),
    #[error("reading {path}: {source}")]
    Parse {
        path: PathBuf,
        source: goblin::error::Error,
    },
    #[error("{0} isn't a program this packager reads")]
    NotAProgram(PathBuf),
    #[error(
        "{program} loads {}; a release links Qt statically, built without ICU (bun run qt:static)",
        libraries.join(", ")
    )]
    LinksQt {
        program: PathBuf,
        libraries: Vec<String>,
    },
    #[error(
        "{program} looks for libraries in {}, which an installed copy won't have",
        runpaths.join(", ")
    )]
    AbsoluteRunpath {
        program: PathBuf,
        runpaths: Vec<String>,
    },
    #[error("no package is known for {}; add it to crates/packager/src/linux_deps.rs", .0.join(", "))]
    UnknownLibraries(Vec<String>),
    #[error(
        "unknown format {0}; the formats are deb, rpm, appimage, pacman, nsis, msi, app and dmg"
    )]
    UnknownFormat(String),
    #[error("{format} is built on {builds_on}, not {here}")]
    WrongPlatform {
        format: &'static str,
        builds_on: &'static str,
        here: &'static str,
    },
    #[error("{0}")]
    Usage(String),
    #[error("packaging: {0}")]
    Packager(#[from] cargo_packager::Error),
    #[error("building the rpm: {0}")]
    Rpm(#[from] rpm::Error),
}

pub fn io_error(path: impl Into<PathBuf>) -> impl FnOnce(std::io::Error) -> PackagerError {
    let path = path.into();
    move |source| PackagerError::Io { path, source }
}
