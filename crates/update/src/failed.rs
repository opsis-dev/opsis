use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const FAILED_FILE: &str = "update-failed.json";
const BYTE_ORDER_MARK: char = '\u{feff}';

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailedUpdate {
    pub version: String,
    pub reason: String,
}

impl FailedUpdate {
    pub fn message(&self) -> String {
        format!(
            "Version {} didn't install. {} Check for updates to try again.",
            self.version, self.reason
        )
    }
}

pub fn failed_file(user_data: &Path) -> PathBuf {
    user_data.join(FAILED_FILE)
}

pub fn record(file: &Path, failed: &FailedUpdate) -> std::io::Result<()> {
    std::fs::write(file, serde_json::to_vec(failed)?)
}

pub fn take(file: &Path) -> Option<FailedUpdate> {
    let text = std::fs::read_to_string(file).ok()?;
    if let Err(error) = std::fs::remove_file(file) {
        log::warn!("removing the note of a failed update: {error}");
    }
    serde_json::from_str(text.trim_start_matches(BYTE_ORDER_MARK))
        .inspect_err(|error| log::warn!("reading the note of a failed update: {error}"))
        .ok()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn a_noted_failure_is_read_once_as_written(
            version in "[0-9]{1,2}\\.[0-9]{1,2}\\.[0-9]{1,2}",
            reason in ".{0,60}",
            marked in any::<bool>(),
        ) {
            let dir = tempfile::tempdir().unwrap();
            let file = failed_file(dir.path());
            let failed = FailedUpdate { version, reason };
            if marked {
                let text = format!("{BYTE_ORDER_MARK}{}", serde_json::to_string(&failed).unwrap());
                std::fs::write(&file, text).unwrap();
            } else {
                record(&file, &failed).unwrap();
            }
            prop_assert_eq!(take(&file), Some(failed));
            prop_assert!(!file.exists());
            prop_assert_eq!(take(&file), None);
        }
    }
}
