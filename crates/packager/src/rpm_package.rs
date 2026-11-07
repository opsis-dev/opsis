use std::path::{Path, PathBuf};

use rpm::{BuildConfig, CompressionType, Dependency, FileOptions, PackageBuilder};

use crate::app::{
    AppFiles, DESCRIPTION, FFMPEG, HOMEPAGE, LICENSE, PROGRAM, PUBLISHER, SUMMARY, VERSION,
    desktop_entry,
};
use crate::error::{PackagerError, io_error};
use crate::linking::Linked;
use crate::linux_deps::rpm_requires;

const RELEASE: &str = "1";
const PROGRAM_MODE: u16 = 0o755;
const FILE_MODE: u16 = 0o644;

pub fn install_path(relative: &Path) -> String {
    format!("/usr/lib/{PROGRAM}/{}", relative.to_string_lossy())
}

pub fn build_rpm(app: &AppFiles, linked: &Linked, out: &Path) -> Result<PathBuf, PackagerError> {
    let arch = std::env::consts::ARCH;
    let mut builder = PackageBuilder::new(PROGRAM, VERSION, LICENSE, arch, SUMMARY);
    builder
        .using_config(BuildConfig::v4().compression(CompressionType::Zstd))
        .release(RELEASE)
        .description(DESCRIPTION)
        .url(HOMEPAGE)
        .vendor(PUBLISHER);
    for soname in rpm_requires(linked)? {
        builder.requires(Dependency::any(soname));
    }
    builder.with_file(
        &app.program,
        FileOptions::new(format!("/usr/bin/{PROGRAM}")).permissions(PROGRAM_MODE),
    )?;
    builder.with_file(
        &app.ffmpeg,
        FileOptions::new(format!("/usr/bin/{FFMPEG}")).permissions(PROGRAM_MODE),
    )?;
    for placed in app.resource_files()? {
        builder.with_file(
            &placed.source,
            FileOptions::new(install_path(&placed.target)).permissions(FILE_MODE),
        )?;
    }
    let staging = out.join(".rpm-staging");
    std::fs::create_dir_all(&staging).map_err(io_error(&staging))?;
    let desktop = staging.join(format!("{PROGRAM}.desktop"));
    std::fs::write(&desktop, desktop_entry()).map_err(io_error(&desktop))?;
    builder.with_file(
        &desktop,
        FileOptions::new(format!("/usr/share/applications/{PROGRAM}.desktop"))
            .permissions(FILE_MODE),
    )?;
    for (size, icon) in app.icons() {
        builder.with_file(
            &icon,
            FileOptions::new(format!(
                "/usr/share/icons/hicolor/{size}x{size}/apps/{PROGRAM}.png"
            ))
            .permissions(FILE_MODE),
        )?;
    }
    let package = builder.build()?;
    let path = out.join(format!("{PROGRAM}-{VERSION}-{RELEASE}.{arch}.rpm"));
    package.write_file(&path)?;
    std::fs::remove_dir_all(&staging).map_err(io_error(&staging))?;
    Ok(path)
}
