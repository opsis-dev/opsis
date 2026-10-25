use std::path::Path;

use opsis_engine::files::{none_if_missing, read_json, stamp_of, write_json};
use opsis_types::values::NodeResult;

const RESULT_FILE: &str = "result.json";

pub fn read_cached(
    cache_dir: &Path,
    hash: &str,
    project_dir: &Path,
) -> std::io::Result<Option<NodeResult>> {
    let Some(result) = read_json::<NodeResult>(&cache_dir.join(hash).join(RESULT_FILE))? else {
        return Ok(None);
    };
    if result.hash != hash {
        return Ok(None);
    }
    for artifact in &result.artifacts {
        if artifact.stamp.is_none()
            || artifact_stamp(&project_dir.join(&artifact.path))? != artifact.stamp
        {
            return Ok(None);
        }
    }
    Ok(Some(result))
}

fn artifact_stamp(path: &Path) -> std::io::Result<Option<String>> {
    Ok(none_if_missing(std::fs::metadata(path))?.map(|meta| stamp_of(&meta)))
}

pub fn stamp_artifacts(mut result: NodeResult, project_dir: &Path) -> std::io::Result<NodeResult> {
    for artifact in &mut result.artifacts {
        if let Some(stamp) = artifact_stamp(&project_dir.join(&artifact.path))? {
            artifact.stamp = Some(stamp);
        }
    }
    Ok(result)
}

pub fn write_cached(cache_dir: &Path, result: &NodeResult) -> std::io::Result<()> {
    let dir = cache_dir.join(&result.hash);
    std::fs::create_dir_all(&dir)?;
    write_json(&dir.join(RESULT_FILE), result)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn result(hash: &str, artifacts: serde_json::Value) -> NodeResult {
        serde_json::from_value(json!({ "hash": hash, "outputs": {}, "artifacts": artifacts, "finishedAt": 0, "durationMs": 0 })).unwrap()
    }

    #[test]
    fn reads_a_saved_result_only_from_the_folder_of_its_own_key() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("cache");
        write_cached(&cache, &result("aaa", json!([]))).unwrap();
        assert_eq!(
            read_cached(&cache, "aaa", dir.path())
                .unwrap()
                .map(|r| r.hash)
                .as_deref(),
            Some("aaa")
        );
        std::fs::create_dir_all(cache.join("bbb")).unwrap();
        std::fs::copy(cache.join("aaa/result.json"), cache.join("bbb/result.json")).unwrap();
        assert!(read_cached(&cache, "bbb", dir.path()).unwrap().is_none());
    }

    #[test]
    fn reads_a_saved_result_only_while_its_files_are_the_ones_it_saved() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("cache");
        std::fs::create_dir_all(dir.path().join("out")).unwrap();
        std::fs::write(dir.path().join("out/film.png"), "red").unwrap();
        let stamped = stamp_artifacts(
            result("red", json!([{ "kind": "image", "path": "out/film.png" }])),
            dir.path(),
        )
        .unwrap();
        write_cached(&cache, &stamped).unwrap();
        assert!(read_cached(&cache, "red", dir.path()).unwrap().is_some());
        std::fs::write(
            dir.path().join("out/film.png"),
            "blue, written by another export",
        )
        .unwrap();
        assert!(read_cached(&cache, "red", dir.path()).unwrap().is_none());
    }

    #[test]
    fn stamps_an_exported_folder_so_its_saved_result_is_used_again() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("cache");
        std::fs::create_dir_all(dir.path().join("out/video-code")).unwrap();
        let stamped = stamp_artifacts(
            result(
                "code",
                json!([{ "kind": "file", "path": "out/video-code" }]),
            ),
            dir.path(),
        )
        .unwrap();
        assert!(stamped.artifacts[0].stamp.is_some());
        write_cached(&cache, &stamped).unwrap();
        assert!(read_cached(&cache, "code", dir.path()).unwrap().is_some());
    }
}
