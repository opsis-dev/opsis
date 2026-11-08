use std::path::{Path, PathBuf};

use crate::error::PackagerError;

pub const PRODUCT: &str = "Opsis";
pub const PROGRAM: &str = "opsis";
pub const FFMPEG: &str = "opsis-ffmpeg";
pub const IDENTIFIER: &str = "app.opsis.desktop";
pub const PUBLISHER: &str = "Bismit Panda";
pub const COPYRIGHT: &str = "Copyright 2026 Bismit Panda";
pub const LICENSE: &str = "GPL-3.0-or-later";
pub const HOMEPAGE: &str = "https://github.com/bismitpanda/opsis";
pub const SUMMARY: &str = "Record a real website and turn it into a polished product demo.";
pub const DESCRIPTION: &str = "Opsis records a real website, then turns the take into a product demo with camera moves, captions, sound and graphics, and exports it as a video.";
pub const MACOS_MINIMUM: &str = "11.0";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const RESOURCE_FOLDERS: [&str; 3] = ["film-bundle", "terminal-bundle", "resources"];
const RESOURCE_FILES: [&str; 3] = [
    "LICENSE",
    "THIRD_PARTY_NOTICES.md",
    "QT_NOTICES.generated.md",
];
const ICON_SIZES: [u32; 8] = [16, 24, 32, 48, 64, 128, 256, 512];

#[derive(Debug, Clone)]
pub struct AppFiles {
    pub program: PathBuf,
    pub ffmpeg: PathBuf,
    pub target_triple: String,
    pub root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    pub source: PathBuf,
    pub target: PathBuf,
}

impl AppFiles {
    pub fn check(&self) -> Result<(), PackagerError> {
        let needed = [self.program.clone(), self.ffmpeg.clone()]
            .into_iter()
            .chain(
                self.resource_roots()
                    .into_iter()
                    .map(|placed| placed.source),
            )
            .chain(self.icons().into_iter().map(|(_, path)| path));
        for path in needed {
            if !path.exists() {
                return Err(PackagerError::Missing(path));
            }
        }
        Ok(())
    }

    pub fn resource_roots(&self) -> Vec<Placed> {
        RESOURCE_FOLDERS
            .iter()
            .chain(&RESOURCE_FILES)
            .map(|name| Placed {
                source: self.root.join(name),
                target: PathBuf::from(name),
            })
            .collect()
    }

    pub fn resource_files(&self) -> Result<Vec<Placed>, PackagerError> {
        let mut files = Vec::new();
        for root in self.resource_roots() {
            collect_files(&root.source, &root.target, &mut files)?;
        }
        files.sort_by(|a, b| a.target.cmp(&b.target));
        Ok(files)
    }

    pub fn icons(&self) -> Vec<(u32, PathBuf)> {
        ICON_SIZES
            .iter()
            .map(|size| {
                (
                    *size,
                    self.root
                        .join("brand")
                        .join("icons")
                        .join(format!("{size}.png")),
                )
            })
            .collect()
    }
}

fn collect_files(
    source: &Path,
    target: &Path,
    files: &mut Vec<Placed>,
) -> Result<(), PackagerError> {
    if source.is_file() {
        files.push(Placed {
            source: source.to_owned(),
            target: target.to_owned(),
        });
        return Ok(());
    }
    let entries = std::fs::read_dir(source).map_err(crate::error::io_error(source))?;
    for entry in entries {
        let entry = entry.map_err(crate::error::io_error(source))?;
        collect_files(&entry.path(), &target.join(entry.file_name()), files)?;
    }
    Ok(())
}

pub fn desktop_entry() -> String {
    [
        "[Desktop Entry]",
        "Categories=AudioVideo;Video;",
        &format!("Comment={SUMMARY}"),
        &format!("Exec={PROGRAM}"),
        &format!("Icon={PROGRAM}"),
        &format!("Name={PRODUCT}"),
        "Terminal=false",
        "Type=Application",
        "",
    ]
    .join("\n")
}
