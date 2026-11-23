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
        "{program} loads {}; a release links Qt statically, built without ICU (bun run setup)",
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
    #[error("unknown format {0}; the formats are deb, rpm, appimage, pacman, nsis, app and dmg")]
    UnknownFormat(String),
    #[error("{format} is built on {builds_on}, not {here}")]
    WrongPlatform {
        format: &'static str,
        builds_on: &'static str,
        here: &'static str,
    },
    #[error("{0}")]
    Usage(String),
    #[error("{tool} isn't installed or isn't on the PATH: {source}")]
    ToolMissing {
        tool: String,
        source: std::io::Error,
    },
    #[error("{tool} failed ({status}){}", if output.is_empty() { String::new() } else { format!(":\n{output}") })]
    ToolFailed {
        tool: String,
        status: String,
        output: String,
    },
    #[error("{program} holds the installer marker {found} times; a release program holds it once")]
    Marker { program: PathBuf, found: usize },
    #[error("{0} isn't a target triple this packager knows")]
    UnknownTarget(String),
    #[error("ldd couldn't find {0} on this system, so the AppImage can't be put together")]
    SystemLibraryMissing(String),
    #[error("the update key can't be used: {0}")]
    UpdateKey(String),
}

pub fn io_error(path: impl Into<PathBuf>) -> impl FnOnce(std::io::Error) -> PackagerError {
    let path = path.into();
    move |source| PackagerError::Io { path, source }
}
