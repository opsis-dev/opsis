use std::path::{Path, PathBuf};

use opsis_update::installer::Installer;

use crate::app::AppFiles;
use crate::appimage::{AppImageTools, build_appimage};
use crate::deb::build_deb;
use crate::error::PackagerError;
use crate::linking::{check_self_contained, read_linked};
use crate::marker::marked_program;
use crate::pacman::build_pacman;
use crate::rpm_package::build_rpm;
use crate::sign::{UpdateKey, key_from_env, sign_update};
use crate::target::Target;
use crate::tools::{fresh_dir, remove_dir};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Format {
    Deb,
    Rpm,
    AppImage,
    Pacman,
    Nsis,
    App,
    Dmg,
}

impl Format {
    pub const ALL: [Self; 7] = [
        Self::Deb,
        Self::Rpm,
        Self::AppImage,
        Self::Pacman,
        Self::Nsis,
        Self::App,
        Self::Dmg,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Deb => "deb",
            Self::Rpm => "rpm",
            Self::AppImage => "appimage",
            Self::Pacman => "pacman",
            Self::Nsis => "nsis",
            Self::App => "app",
            Self::Dmg => "dmg",
        }
    }

    pub const fn builds_on(self) -> &'static str {
        match self {
            Self::Deb | Self::Rpm | Self::AppImage | Self::Pacman => "linux",
            Self::Nsis => "windows",
            Self::App | Self::Dmg => "macos",
        }
    }

    pub const fn installer(self) -> Installer {
        match self {
            Self::Deb => Installer::Deb,
            Self::Rpm => Installer::Rpm,
            Self::AppImage => Installer::AppImage,
            Self::Pacman => Installer::Pacman,
            Self::Nsis => Installer::Nsis,
            Self::App | Self::Dmg => Installer::App,
        }
    }

    pub fn parse(name: &str) -> Result<Self, PackagerError> {
        Self::ALL
            .into_iter()
            .find(|format| format.name() == name)
            .ok_or_else(|| PackagerError::UnknownFormat(name.to_owned()))
    }
}

#[derive(Debug, Clone, Default)]
pub struct Tools {
    pub appimage: Option<AppImageTools>,
}

fn signed(
    key: Option<&UpdateKey>,
    file: PathBuf,
    installer: Installer,
    target: &Target,
) -> Result<Vec<PathBuf>, PackagerError> {
    let mut made = Vec::new();
    if let Some(key) = key {
        made.extend(sign_update(key, &file, installer, target)?);
    }
    made.insert(0, file);
    Ok(made)
}

fn appimage_tools(tools: &Tools) -> Result<&AppImageTools, PackagerError> {
    tools.appimage.as_ref().ok_or_else(|| {
        PackagerError::Usage("the AppImage needs --appimagetool and --appimage-runtime".to_owned())
    })
}

fn build_one(
    format: Format,
    app: &AppFiles,
    target: &Target,
    tools: &Tools,
    key: Option<&UpdateKey>,
    out: &Path,
) -> Result<Vec<PathBuf>, PackagerError> {
    let linked = read_linked(&app.program)?;
    let marked_dir = fresh_dir(&out.join(format!(".marked-{}", format.name())))?;
    let program = marked_program(&app.program, format.installer(), &marked_dir)?;
    let made = match format {
        Format::Deb => vec![build_deb(app, &linked, &program, target, out)?],
        Format::Rpm => vec![build_rpm(app, &linked, &program, target, out)?],
        Format::Pacman => vec![build_pacman(app, &linked, &program, target, out)?],
        Format::AppImage => signed(
            key,
            build_appimage(app, &program, target, appimage_tools(tools)?, out)?,
            Installer::AppImage,
            target,
        )?,
        Format::Nsis => signed(
            key,
            crate::windows::build_nsis(app, &program, target, out)?,
            Installer::Nsis,
            target,
        )?,
        Format::App | Format::Dmg => {
            let bundle = crate::macos::build_app(app, &program, out)?;
            let archive = crate::macos::update_archive(&bundle, target, out)?;
            let mut made = vec![bundle.clone()];
            made.extend(signed(key, archive, Installer::App, target)?);
            if format == Format::Dmg {
                made.push(crate::macos::build_dmg(&bundle, target, out)?);
            }
            made
        }
    };
    remove_dir(&marked_dir)?;
    Ok(made)
}

fn plan(formats: &[Format]) -> Vec<Format> {
    let mut planned: Vec<Format> = formats.to_vec();
    planned.sort_unstable();
    planned.dedup();
    if planned.contains(&Format::Dmg) {
        planned.retain(|format| *format != Format::App);
    }
    planned
}

pub fn package(
    app: &AppFiles,
    formats: &[Format],
    tools: &Tools,
    out: &Path,
) -> Result<Vec<PathBuf>, PackagerError> {
    let here = std::env::consts::OS;
    if let Some(format) = formats.iter().find(|format| format.builds_on() != here) {
        return Err(PackagerError::WrongPlatform {
            format: format.name(),
            builds_on: format.builds_on(),
            here,
        });
    }
    app.check()?;
    let target = Target::parse(&app.target_triple)?;
    let linked = read_linked(&app.program)?;
    check_self_contained(&app.program, &linked)?;
    std::fs::create_dir_all(out).map_err(crate::error::io_error(out))?;
    let key = key_from_env()?;
    let mut made = Vec::new();
    for format in plan(formats) {
        made.extend(build_one(format, app, &target, tools, key.as_ref(), out)?);
    }
    Ok(made)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn a_format_reads_back_from_its_name(format in prop::sample::select(Format::ALL.to_vec())) {
            prop_assert_eq!(Format::parse(format.name()).unwrap(), format);
        }

        #[test]
        fn a_plan_builds_each_format_once_and_the_app_once(formats in prop::collection::vec(prop::sample::select(Format::ALL.to_vec()), 0..12)) {
            let planned = plan(&formats);
            let mut unique = planned.clone();
            unique.dedup();
            prop_assert_eq!(&unique, &planned);
            prop_assert!(!(planned.contains(&Format::App) && planned.contains(&Format::Dmg)));
            for format in &formats {
                prop_assert!(planned.contains(format) || (*format == Format::App && planned.contains(&Format::Dmg)));
            }
        }
    }

    #[test]
    fn an_unknown_format_is_refused() {
        assert!(
            matches!(Format::parse("snap"), Err(PackagerError::UnknownFormat(name)) if name == "snap")
        );
    }
}
