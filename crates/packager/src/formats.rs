use std::path::{Path, PathBuf};

use cargo_packager::config::{
    AppCategory, AppImageConfig, Binary, DebianConfig, DmgConfig, MacOsConfig, NsisConfig,
    PacmanConfig, Resource, WixConfig,
};
use cargo_packager::{Config, PackageFormat};

use crate::app::{
    AppFiles, COPYRIGHT, DESCRIPTION, FFMPEG, HOMEPAGE, IDENTIFIER, MACOS_MINIMUM, PRODUCT,
    PROGRAM, PUBLISHER, SUMMARY, VERSION, desktop_entry,
};
use crate::error::{PackagerError, io_error};
use crate::linking::{Linked, check_self_contained, read_linked};
use crate::linux_deps::{deb_depends, pacman_depends};
use crate::rpm_package::build_rpm;

const DEB_SECTION: &str = "video";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Format {
    Deb,
    Rpm,
    AppImage,
    Pacman,
    Nsis,
    Msi,
    App,
    Dmg,
}

impl Format {
    pub const ALL: [Self; 8] = [
        Self::Deb,
        Self::Rpm,
        Self::AppImage,
        Self::Pacman,
        Self::Nsis,
        Self::Msi,
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
            Self::Msi => "msi",
            Self::App => "app",
            Self::Dmg => "dmg",
        }
    }

    pub const fn builds_on(self) -> &'static str {
        match self {
            Self::Deb | Self::Rpm | Self::AppImage | Self::Pacman => "linux",
            Self::Nsis | Self::Msi => "windows",
            Self::App | Self::Dmg => "macos",
        }
    }

    pub fn parse(name: &str) -> Result<Self, PackagerError> {
        Self::ALL
            .into_iter()
            .find(|format| format.name() == name)
            .ok_or_else(|| PackagerError::UnknownFormat(name.to_owned()))
    }

    const fn packager_format(self) -> Option<PackageFormat> {
        match self {
            Self::Deb => Some(PackageFormat::Deb),
            Self::Rpm => None,
            Self::AppImage => Some(PackageFormat::AppImage),
            Self::Pacman => Some(PackageFormat::Pacman),
            Self::Nsis => Some(PackageFormat::Nsis),
            Self::Msi => Some(PackageFormat::Wix),
            Self::App => Some(PackageFormat::App),
            Self::Dmg => Some(PackageFormat::Dmg),
        }
    }
}

fn ffmpeg_base(app: &AppFiles) -> Result<PathBuf, PackagerError> {
    let suffix = if cfg!(windows) { ".exe" } else { "" };
    let expected = format!("{FFMPEG}-{}{suffix}", app.target_triple);
    let named = app
        .ffmpeg
        .file_name()
        .is_some_and(|name| name.to_string_lossy() == expected);
    if !named {
        return Err(PackagerError::Usage(format!(
            "the FFmpeg program must be named {expected}, not {}",
            app.ffmpeg.display()
        )));
    }
    Ok(app.ffmpeg.with_file_name(FFMPEG))
}

fn text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub fn packager_config(
    app: &AppFiles,
    linked: &Linked,
    formats: &[Format],
    out: &Path,
    desktop_template: &Path,
) -> Result<Config, PackagerError> {
    let wanted: Vec<PackageFormat> = formats
        .iter()
        .filter_map(|format| format.packager_format())
        .collect();
    let linux = wanted.iter().any(|format| {
        matches!(
            format,
            PackageFormat::Deb | PackageFormat::AppImage | PackageFormat::Pacman
        )
    });
    let deb = if linux {
        Some(
            DebianConfig::new()
                .depends(deb_depends(linked)?)
                .desktop_template(desktop_template)
                .section(DEB_SECTION),
        )
    } else {
        None
    };
    let pacman = if linux {
        Some(PacmanConfig::new().depends(pacman_depends(linked)?))
    } else {
        None
    };
    let binaries_dir = app
        .program
        .parent()
        .map(Path::to_owned)
        .ok_or_else(|| PackagerError::Missing(app.program.clone()))?;
    let mut config = Config::default();
    PRODUCT.clone_into(&mut config.product_name);
    VERSION.clone_into(&mut config.version);
    config.binaries = vec![Binary::new(PROGRAM).main(true)];
    config.identifier = Some(IDENTIFIER.to_owned());
    config.formats = Some(wanted);
    out.clone_into(&mut config.out_dir);
    config.binaries_dir = Some(binaries_dir);
    config.target_triple = Some(app.target_triple.clone());
    config.description = Some(SUMMARY.to_owned());
    config.long_description = Some(DESCRIPTION.to_owned());
    config.homepage = Some(HOMEPAGE.to_owned());
    config.authors = Some(vec![PUBLISHER.to_owned()]);
    config.publisher = Some(PUBLISHER.to_owned());
    config.license_file = Some(app.root.join("LICENSE"));
    config.copyright = Some(COPYRIGHT.to_owned());
    config.category = Some(AppCategory::Video);
    config.icons = Some(app.icons().iter().map(|(_, path)| text(path)).collect());
    config.resources = Some(
        app.resource_roots()
            .into_iter()
            .map(|placed| Resource::Mapped {
                src: text(&placed.source),
                target: placed.target,
            })
            .collect(),
    );
    config.external_binaries = Some(vec![ffmpeg_base(app)?]);
    config.deb = deb;
    config.appimage = Some(AppImageConfig::new());
    config.pacman = pacman;
    config.nsis = Some(NsisConfig::new());
    config.wix = Some(WixConfig::new());
    config.dmg = Some(DmgConfig::new());
    config.macos = Some(MacOsConfig::new().minimum_system_version(MACOS_MINIMUM));
    Ok(config)
}

pub fn package(
    app: &AppFiles,
    formats: &[Format],
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
    let linked = read_linked(&app.program)?;
    check_self_contained(&app.program, &linked)?;
    std::fs::create_dir_all(out).map_err(io_error(out))?;
    let mut made = Vec::new();
    if formats
        .iter()
        .any(|format| format.packager_format().is_some())
    {
        let template = out.join(format!("{PROGRAM}.desktop.hbs"));
        std::fs::write(&template, desktop_entry()).map_err(io_error(&template))?;
        let config = packager_config(app, &linked, formats, out, &template)?;
        for output in cargo_packager::package(&config)? {
            made.extend(output.paths);
        }
        std::fs::remove_file(&template).map_err(io_error(&template))?;
    }
    if formats.contains(&Format::Rpm) {
        made.push(build_rpm(app, &linked, out)?);
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
    }

    #[test]
    fn an_unknown_format_is_refused() {
        assert!(
            matches!(Format::parse("snap"), Err(PackagerError::UnknownFormat(name)) if name == "snap")
        );
    }
}
