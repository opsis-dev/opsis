use std::collections::BTreeMap;
use std::path::Path;

use opsis_compile::asset_ref::project_rel_path;
use opsis_compile::cards::named_files;
use opsis_engine::files::stamp_of;
use serde_json::Value;

use crate::paths::is_inside;

pub fn file_stamp(file: &Path) -> Option<String> {
    let meta = std::fs::metadata(file)
        .ok()
        .filter(std::fs::Metadata::is_file)?;
    Some(stamp_of(&meta))
}

pub fn ref_stamps(project_dir: &Path, refs: &[String]) -> BTreeMap<String, String> {
    refs.iter()
        .filter_map(|value| {
            let file = project_dir.join(project_rel_path(value.trim())?);
            let stamp = if is_inside(project_dir, &file) {
                file_stamp(&file)
            } else {
                None
            };
            stamp.map(|s| (value.clone(), s))
        })
        .collect()
}

pub fn file_stamps(project_dir: &Path, params: &Value) -> BTreeMap<String, String> {
    ref_stamps(project_dir, &named_files(params))
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use filetime::{FileTime, set_file_mtime};
    use serde_json::json;

    use super::*;

    #[test]
    fn changes_when_a_project_file_is_replaced_in_place() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("assets")).unwrap();
        let file = dir.path().join("assets/clip.mp4");
        std::fs::write(&file, "first").unwrap();
        let params = json!({ "path": "assets/clip.mp4", "label": "Intro" });
        let before = file_stamps(dir.path(), &params);
        std::fs::write(&file, "second take").unwrap();
        set_file_mtime(
            &file,
            FileTime::from_system_time(SystemTime::now() + Duration::from_secs(5)),
        )
        .unwrap();
        let after = file_stamps(dir.path(), &params);
        assert_eq!(before.keys().collect::<Vec<_>>(), ["assets/clip.mp4"]);
        assert_ne!(after, before);
    }

    #[test]
    fn reads_files_in_rows_and_project_refs_and_skips_missing_files_and_files_outside() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("logo.png"), "png").unwrap();
        let stamps = file_stamps(
            dir.path(),
            &json!({
                "rows": [{ "image": "project:logo.png" }, { "image": "gone.png" }],
                "outside": "../secret.txt",
                "app": "app:placeholders/logo.svg",
            }),
        );
        assert_eq!(stamps.keys().collect::<Vec<_>>(), ["project:logo.png"]);
    }
}
