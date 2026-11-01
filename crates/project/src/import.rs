use std::path::{Path, PathBuf};

use crate::model::{MODEL_EXTENSIONS, ModelError, model_files};
use crate::paths::{is_inside, project_paths, to_posix_rel};

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error(transparent)]
    Model(#[from] ModelError),
    #[error("This model points to files outside its folder or on the internet: {}", .0.join(", "))]
    OutsideFiles(Vec<String>),
    #[error("This model needs these files next to it: {}", .0.join(", "))]
    MissingFiles(Vec<String>),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

fn extension(file: &Path) -> String {
    file.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

fn file_name(file: &Path) -> String {
    file.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn unique_name(folder: &Path, name: &str) -> String {
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let entries = std::fs::read_dir(folder).map_or(0, Iterator::count);
    std::iter::once(name.to_owned())
        .chain((1..).map(|n| format!("{stem}-{n}{ext}")))
        .take(entries + 1)
        .find(|candidate| !folder.join(candidate).exists())
        .expect("a free name")
}

fn import_model(project_dir: &Path, file: &Path) -> Result<String, ImportError> {
    let found = model_files(&std::fs::read(file)?)?;
    if !found.refused.is_empty() {
        return Err(ImportError::OutsideFiles(found.refused));
    }
    let source = file.parent().unwrap_or_else(|| Path::new("."));
    let missing: Vec<String> = found
        .files
        .iter()
        .filter(|f| !source.join(f).exists())
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(ImportError::MissingFiles(missing));
    }
    let models = project_paths(project_dir).models;
    std::fs::create_dir_all(&models)?;
    let stem = file
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let folder = unique_name(&models, &stem);
    let name = file_name(file);
    for part in std::iter::once(&name).chain(&found.files) {
        let to = models.join(&folder).join(part);
        std::fs::create_dir_all(to.parent().unwrap_or(&models))?;
        std::fs::copy(source.join(part), to)?;
    }
    Ok(format!("assets/models/{folder}/{name}"))
}

pub fn import_assets(project_dir: &Path, files: &[PathBuf]) -> Result<Vec<String>, ImportError> {
    let assets = project_paths(project_dir).assets;
    std::fs::create_dir_all(&assets)?;
    files
        .iter()
        .map(|file| {
            if let Some(rel) = is_inside(project_dir, file)
                .then(|| to_posix_rel(project_dir, file))
                .flatten()
            {
                return Ok(rel);
            }
            if MODEL_EXTENSIONS.contains(&extension(file).as_str()) {
                return import_model(project_dir, file);
            }
            let name = unique_name(&assets, &file_name(file));
            std::fs::copy(file, assets.join(&name))?;
            Ok(format!("assets/{name}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn write(file: &Path, text: &str) {
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, text).unwrap();
    }

    #[test]
    fn copies_files_in_with_free_names_and_keeps_files_already_in_the_project() {
        let project = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        write(&outside.path().join("logo.png"), "a");
        write(&project.path().join("assets/logo.png"), "b");
        write(&project.path().join("shots/take.mp4"), "c");
        let imported = import_assets(
            project.path(),
            &[
                outside.path().join("logo.png"),
                outside.path().join("logo.png"),
                project.path().join("shots/take.mp4"),
            ],
        )
        .unwrap();
        assert_eq!(
            imported,
            ["assets/logo-1.png", "assets/logo-2.png", "shots/take.mp4"]
        );
        assert_eq!(
            std::fs::read_to_string(project.path().join("assets/logo-2.png")).unwrap(),
            "a"
        );
    }

    #[test]
    fn copies_a_model_with_the_files_it_names_into_its_own_folder() {
        let project = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let model = json!({ "asset": { "version": "2.0" }, "buffers": [{ "uri": "robot.bin", "byteLength": 1 }], "images": [{ "uri": "maps/skin.png" }] });
        write(&outside.path().join("robot.gltf"), &model.to_string());
        write(&outside.path().join("robot.bin"), "b");
        write(&outside.path().join("maps/skin.png"), "p");
        let imported = import_assets(project.path(), &[outside.path().join("robot.gltf")]).unwrap();
        assert_eq!(imported, ["assets/models/robot/robot.gltf"]);
        assert!(
            project
                .path()
                .join("assets/models/robot/maps/skin.png")
                .exists()
        );
        std::fs::remove_file(outside.path().join("robot.bin")).unwrap();
        let missing =
            import_assets(project.path(), &[outside.path().join("robot.gltf")]).unwrap_err();
        assert_eq!(
            missing.to_string(),
            "This model needs these files next to it: robot.bin"
        );
    }
}
