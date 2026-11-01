use std::path::{Path, PathBuf};

use opsis_compile::asset_ref::{RefTarget, project_ref, ref_target};
use opsis_types::project::{
    ANALYSIS_DIR, ASSETS_DIR, CACHE_DIR, LAST_RUN_FILE, META_DIR, MIX_DIR, MODELS_DIR, OUT_DIR,
    PICTURES_DIR, ProjectLocations, TRACES_DIR,
};
use opsis_types::settings::{PROJECTS_FOLDER_NAME, ProjectsSection};

pub fn project_locations(settings: &ProjectsSection, home: &Path) -> ProjectLocations {
    let default_dir = match settings.default_dir.as_str() {
        "" => home.join(PROJECTS_FOLDER_NAME),
        dir => PathBuf::from(dir),
    };
    let last = Path::new(&settings.last_dir);
    let suggested_dir = if !settings.last_dir.is_empty() && last.is_dir() {
        last.to_path_buf()
    } else {
        default_dir.clone()
    };
    ProjectLocations {
        default_dir: default_dir.to_string_lossy().into_owned(),
        suggested_dir: suggested_dir.to_string_lossy().into_owned(),
    }
}

pub struct ProjectPaths {
    pub meta: PathBuf,
    pub cache: PathBuf,
    pub analysis: PathBuf,
    pub mix: PathBuf,
    pub pictures: PathBuf,
    pub traces: PathBuf,
    pub out: PathBuf,
    pub last_run: PathBuf,
    pub assets: PathBuf,
    pub models: PathBuf,
}

pub fn project_paths(dir: &Path) -> ProjectPaths {
    ProjectPaths {
        meta: dir.join(META_DIR),
        cache: dir.join(CACHE_DIR),
        analysis: dir.join(ANALYSIS_DIR),
        mix: dir.join(MIX_DIR),
        pictures: dir.join(PICTURES_DIR),
        traces: dir.join(TRACES_DIR),
        out: dir.join(OUT_DIR),
        last_run: dir.join(LAST_RUN_FILE),
        assets: dir.join(ASSETS_DIR),
        models: dir.join(MODELS_DIR),
    }
}

fn lexical(path: &Path) -> PathBuf {
    path_clean::clean(std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf()))
}

pub fn is_inside(root: &Path, file: &Path) -> bool {
    lexical(file).starts_with(lexical(root))
}

pub fn to_posix_rel(root: &Path, file: &Path) -> Option<String> {
    let rel = lexical(file)
        .strip_prefix(lexical(root))
        .ok()?
        .to_path_buf();
    let parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    Some(parts.join("/"))
}

pub fn to_project_ref(root: &Path, file: &Path) -> Option<String> {
    to_posix_rel(root, file).map(|rel| project_ref(&rel))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("Use a file on this computer, not a web address.")]
pub struct WebAddress;

pub fn ref_to_path(
    project_dir: &Path,
    public_dir: &Path,
    reference: &str,
) -> Result<PathBuf, WebAddress> {
    match ref_target(reference) {
        RefTarget::Project(rel) => Ok(project_dir.join(rel)),
        RefTarget::App(rel) => Ok(public_dir.join(rel)),
        RefTarget::Absolute(path) => Ok(PathBuf::from(path)),
        RefTarget::External(_) => Err(WebAddress),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_folder_and_anything_under_it_including_names_that_start_with_dots() {
        let root = Path::new("/projects/demo");
        assert!(is_inside(root, root));
        assert!(is_inside(root, &root.join("assets").join("clip.mp4")));
        assert!(is_inside(root, &root.join("..notes.png")));
    }

    #[test]
    fn refuses_anything_outside() {
        let root = Path::new("/projects/demo");
        assert!(!is_inside(root, &root.join("..")));
        assert!(!is_inside(
            root,
            &root.join("..").join("other").join("a.png")
        ));
        assert!(!is_inside(root, Path::new("/elsewhere/a.png")));
        assert!(!is_inside(root, Path::new("/projects/demo-two/a.png")));
    }

    #[test]
    fn writes_paths_inside_the_project_as_refs() {
        let root = Path::new("/projects/demo");
        assert_eq!(
            to_project_ref(root, &root.join("assets").join("a b.png")).as_deref(),
            Some("project:assets/a b.png")
        );
        assert_eq!(to_project_ref(root, Path::new("/elsewhere/a.png")), None);
    }

    #[test]
    fn suggests_the_last_folder_while_it_exists_and_otherwise_the_default() {
        let home = tempfile::tempdir().unwrap();
        let last = home.path().join("films");
        let mut settings = ProjectsSection {
            default_dir: String::new(),
            last_dir: last.to_string_lossy().into_owned(),
        };
        let default = home
            .path()
            .join(PROJECTS_FOLDER_NAME)
            .to_string_lossy()
            .into_owned();
        assert_eq!(
            project_locations(&settings, home.path()).suggested_dir,
            default
        );
        std::fs::create_dir(&last).unwrap();
        assert_eq!(
            project_locations(&settings, home.path()),
            ProjectLocations {
                default_dir: default,
                suggested_dir: settings.last_dir.clone()
            }
        );
        settings.default_dir = "/srv/opsis".into();
        assert_eq!(
            project_locations(&settings, home.path()).default_dir,
            "/srv/opsis"
        );
    }
}
