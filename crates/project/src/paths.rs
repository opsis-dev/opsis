use std::path::{Path, PathBuf};

use opsis_compile::asset_ref::project_ref;
use opsis_types::project::{
    ANALYSIS_DIR, ASSETS_DIR, CACHE_DIR, LAST_RUN_FILE, META_DIR, MIX_DIR, MODELS_DIR, OUT_DIR,
    PICTURES_DIR, TRACES_DIR,
};

pub const DEFAULT_FOLDER_NAME: &str = "demo";

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

pub fn project_folder_name(name: &str) -> String {
    match slug::slugify(name) {
        slug if slug.is_empty() => DEFAULT_FOLDER_NAME.to_owned(),
        slug => slug,
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
    fn names_the_folder_after_the_project() {
        assert_eq!(project_folder_name("My Demo: v1.2"), "my-demo-v1-2");
        assert_eq!(project_folder_name("  ...  "), DEFAULT_FOLDER_NAME);
    }
}
