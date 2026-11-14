use std::path::{Path, PathBuf};

use opsis_compile::asset_ref::{PreparedKind, PreparedRef, RefTarget, extension_of};
use opsis_engine::files::{atomic_write, file_identity};
use opsis_engine::hash::versioned_hash;

use crate::{PREPARED_VERSION, PrepareError, model, svg, texture, typeface};

const KEY_LENGTH: usize = 32;
const CACHE_FOLDER: &str = ".opsis/cache/prepared";

#[derive(Debug, Clone)]
pub struct PreparedDirs {
    pub project: PathBuf,
    pub public: PathBuf,
}

#[derive(Debug)]
pub struct Prepared {
    pub bytes: Vec<u8>,
}

fn source_file(dirs: &PreparedDirs, prepared: &PreparedRef) -> Option<PathBuf> {
    match &prepared.source {
        RefTarget::Project(rel) => Some(dirs.project.join(rel)),
        RefTarget::App(rel) => Some(dirs.public.join(rel)),
        RefTarget::External(_) | RefTarget::Absolute(_) => None,
    }
}

fn make(prepared: &PreparedRef, source: &Path) -> Result<Vec<u8>, PrepareError> {
    let bytes = std::fs::read(source).map_err(|e| PrepareError::Detail(e.to_string()))?;
    match prepared.kind {
        PreparedKind::Typeface { weight } => {
            Ok(serde_json::to_vec(&typeface::typeface(&bytes, weight)?)
                .map_err(|e| PrepareError::Detail(e.to_string()))?)
        }
        PreparedKind::Model => {
            model::model(&bytes, source.parent().unwrap_or_else(|| Path::new(".")))
        }
        PreparedKind::Svg => svg::svg(&bytes),
        PreparedKind::Texture if extension_of(&source.to_string_lossy()) == "svg" => Ok(bytes),
        PreparedKind::Texture => texture::texture(&bytes),
    }
}

pub fn prepare(dirs: &PreparedDirs, prepared: &PreparedRef) -> Result<Prepared, PrepareError> {
    let source = source_file(dirs, prepared).ok_or(PrepareError::Unreadable)?;
    let identity = file_identity(&source).map_err(|e| PrepareError::Detail(e.to_string()))?;
    let key = versioned_hash(PREPARED_VERSION, &(prepared.path(), identity), KEY_LENGTH);
    let cached = dirs.project.join(CACHE_FOLDER).join(&key);
    if let Ok(bytes) = std::fs::read(&cached) {
        return Ok(Prepared { bytes });
    }
    let bytes = make(prepared, &source)?;
    if let Some(folder) = cached.parent()
        && let Err(error) =
            std::fs::create_dir_all(folder).and_then(|()| atomic_write(&cached, &bytes))
    {
        log::warn!("keeping a prepared file: {error}");
    }
    Ok(Prepared { bytes })
}
