use std::path::Path;

use opsis_engine::files::{read_json, write_json};
use opsis_types::values::LastRun;
use serde_json::Value;

use crate::paths::project_paths;

pub fn read_last_run(project_dir: &Path) -> std::io::Result<Option<LastRun>> {
    Ok(read_json::<Value>(&project_paths(project_dir).last_run)?.and_then(|v| LastRun::parse(&v)))
}

pub fn write_last_run(project_dir: &Path, last_run: &LastRun) -> std::io::Result<()> {
    let paths = project_paths(project_dir);
    std::fs::create_dir_all(&paths.meta)?;
    write_json(&paths.last_run, last_run)
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::*;

    #[test]
    fn reads_back_what_it_wrote_and_nothing_from_a_damaged_file() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_last_run(dir.path()).unwrap(), None);
        let run = LastRun {
            updated_at: 12.0,
            nodes: IndexMap::new(),
            failures: None,
        };
        write_last_run(dir.path(), &run).unwrap();
        assert_eq!(read_last_run(dir.path()).unwrap(), Some(run));
        std::fs::write(project_paths(dir.path()).last_run, "{").unwrap();
        assert_eq!(read_last_run(dir.path()).unwrap(), None);
    }
}
