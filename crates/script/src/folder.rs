use std::io;
use std::path::{Path, PathBuf};

use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};

use crate::paths::{PathRefusal, inside_project, writable};

#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error(transparent)]
    Refused(#[from] PathRefusal),
    #[error("{path}: {source}")]
    Io { path: String, source: io::Error },
}

impl FileError {
    fn io(path: &str, source: io::Error) -> Self {
        Self::Io {
            path: path.to_owned(),
            source,
        }
    }

    pub fn code(&self) -> i64 {
        match self {
            Self::Refused(_) => 13,
            Self::Io { source, .. } => source.raw_os_error().map_or(5, i64::from),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mode {
    pub read: bool,
    pub write: bool,
    pub append: bool,
    pub truncate: bool,
    pub create: bool,
}

impl Mode {
    pub fn parse(text: &str) -> Option<Self> {
        let mut chars = text.chars();
        let first = chars.next()?;
        let rest: String = chars.collect();
        let plus = rest.starts_with('+');
        let tail = rest.strip_prefix('+').unwrap_or(&rest);
        if !tail.chars().all(|c| c == 'b') {
            return None;
        }
        Some(match first {
            'r' => Self {
                read: true,
                write: plus,
                append: false,
                truncate: false,
                create: false,
            },
            'w' => Self {
                read: plus,
                write: true,
                append: false,
                truncate: true,
                create: true,
            },
            'a' => Self {
                read: plus,
                write: true,
                append: true,
                truncate: false,
                create: true,
            },
            _ => return None,
        })
    }

    pub const READ: Self = Self {
        read: true,
        write: false,
        append: false,
        truncate: false,
        create: false,
    };
}

#[derive(Debug)]
pub struct Folder {
    dir: Dir,
    root: PathBuf,
}

impl Folder {
    pub fn open(root: &Path) -> io::Result<Self> {
        Ok(Self {
            dir: Dir::open_ambient_dir(root, ambient_authority())?,
            root: root.to_path_buf(),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn relative(&self, path: &str) -> Result<PathBuf, FileError> {
        Ok(inside_project(path, &self.root)?)
    }

    fn resolved(&self, path: &str, relative: &Path) -> Result<PathBuf, FileError> {
        match self.dir.canonicalize(relative) {
            Ok(found) => Ok(found),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let parent = relative
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or_else(|| Path::new("."));
                let name = relative
                    .file_name()
                    .ok_or_else(|| FileError::io(path, error))?;
                let parent = self
                    .dir
                    .canonicalize(parent)
                    .map_err(|e| FileError::io(path, e))?;
                Ok(parent.join(name))
            }
            Err(error) => Err(FileError::io(path, error)),
        }
    }

    fn for_writing(&self, path: &str) -> Result<PathBuf, FileError> {
        let relative = self.relative(path)?;
        let resolved = self.resolved(path, &relative)?;
        if writable(&resolved) {
            Ok(relative)
        } else {
            Err(PathRefusal::ReadOnly(path.to_owned()).into())
        }
    }

    pub fn open_file(&self, path: &str, mode: Mode) -> Result<std::fs::File, FileError> {
        let relative = if mode.write {
            self.for_writing(path)?
        } else {
            self.relative(path)?
        };
        let mut options = OpenOptions::new();
        options
            .read(mode.read)
            .write(mode.write && !mode.append)
            .append(mode.append)
            .truncate(mode.truncate)
            .create(mode.create);
        self.dir
            .open_with(&relative, &options)
            .map(cap_std::fs::File::into_std)
            .map_err(|e| FileError::io(path, e))
    }

    pub fn read_text(&self, path: &str) -> Result<String, FileError> {
        let relative = self.relative(path)?;
        self.dir
            .read_to_string(&relative)
            .map_err(|e| FileError::io(path, e))
    }

    pub fn exists(&self, path: &str) -> bool {
        self.relative(path)
            .is_ok_and(|relative| self.dir.metadata(&relative).is_ok())
    }

    pub fn remove(&self, path: &str) -> Result<(), FileError> {
        let relative = self.for_writing(path)?;
        let is_dir = self
            .dir
            .symlink_metadata(&relative)
            .map_err(|e| FileError::io(path, e))?
            .is_dir();
        if is_dir {
            self.dir.remove_dir(&relative)
        } else {
            self.dir.remove_file(&relative)
        }
        .map_err(|e| FileError::io(path, e))
    }

    pub fn rename(&self, from: &str, to: &str) -> Result<(), FileError> {
        let source = self.for_writing(from)?;
        let target = self.for_writing(to)?;
        self.dir
            .rename(&source, &self.dir, &target)
            .map_err(|e| FileError::io(from, e))
    }
}
