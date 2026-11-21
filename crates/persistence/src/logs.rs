use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{Map, Value};

pub const LOGS_DIR: &str = "logs";
pub const ASSISTANT_LOGS: &str = "assistant";
pub const HIDDEN: &str = "[hidden]";
const MAX_BYTES: u64 = 10 * 1024 * 1024;
const KEPT: usize = 5;
const SECRET_NAMES: [&str; 12] = [
    "authorization",
    "proxy-authorization",
    "api_key",
    "apikey",
    "api-key",
    "x-api-key",
    "key",
    "token",
    "access_token",
    "refresh_token",
    "password",
    "secret",
];
const SECRET_PREFIXES: [&str; 4] = ["Bearer ", "Basic ", "sk-", "sk_"];

pub fn assistant_logs(user_data: &Path) -> PathBuf {
    user_data.join(LOGS_DIR).join(ASSISTANT_LOGS)
}

fn secret_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    SECRET_NAMES.contains(&name.as_str())
        || name.ends_with("_key")
        || name.ends_with("-key")
        || name.ends_with("token")
}

fn secret_text(text: &str) -> bool {
    SECRET_PREFIXES
        .iter()
        .any(|prefix| text.starts_with(prefix))
}

pub fn redacted(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(name, inner)| {
                    let inner = if secret_name(name) && !inner.is_null() {
                        Value::String(HIDDEN.to_owned())
                    } else {
                        redacted(inner)
                    };
                    (name.clone(), inner)
                })
                .collect::<Map<_, _>>(),
        ),
        Value::Array(list) => Value::Array(list.iter().map(redacted).collect()),
        Value::String(text) if secret_text(text) => Value::String(HIDDEN.to_owned()),
        other => other.clone(),
    }
}

#[derive(Debug)]
pub struct RequestLog {
    file: PathBuf,
    max_bytes: u64,
    written: Mutex<()>,
}

impl RequestLog {
    pub fn new(dir: &Path, name: &str) -> Self {
        Self::with_limit(dir, name, MAX_BYTES)
    }

    pub fn with_limit(dir: &Path, name: &str, max_bytes: u64) -> Self {
        Self {
            file: dir.join(format!("{name}.ndjson")),
            max_bytes,
            written: Mutex::new(()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.file
    }

    fn rotated(&self, index: usize) -> PathBuf {
        let stem = self.file.with_extension("");
        PathBuf::from(format!("{}.{index}.ndjson", stem.display()))
    }

    fn rotate(&self) -> std::io::Result<()> {
        for index in (1..KEPT).rev() {
            let from = if index == 1 {
                self.file.clone()
            } else {
                self.rotated(index - 1)
            };
            if from.exists() {
                std::fs::rename(&from, self.rotated(index))?;
            }
        }
        Ok(())
    }

    pub fn write(&self, record: &Value) -> std::io::Result<()> {
        let mut line = serde_json::to_string(&redacted(record))?;
        line.push('\n');
        let _held = self
            .written
            .lock()
            .map_err(|_| std::io::Error::other("the log is held"))?;
        if let Some(dir) = self.file.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let size = std::fs::metadata(&self.file).map_or(0, |m| m.len());
        if size > 0 && size + u64::try_from(line.len()).unwrap_or(u64::MAX) > self.max_bytes {
            self.rotate()?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file)?;
        file.write_all(line.as_bytes())
    }

    pub fn files(&self) -> Vec<PathBuf> {
        std::iter::once(self.file.clone())
            .chain((1..KEPT).map(|index| self.rotated(index)))
            .filter(|path| path.exists())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::json;

    use super::*;

    fn record() -> impl Strategy<Value = (Value, String)> {
        let secret = "[A-Za-z0-9]{12,24}";
        let name = prop::sample::select(vec![
            "authorization",
            "Authorization",
            "apiKey",
            "api_key",
            "x-api-key",
            "token",
            "access_token",
            "password",
            "OPENAI_KEY",
        ]);
        (secret, name, "[a-z ]{0,12}", 0_usize..3).prop_map(|(secret, name, text, place)| {
            let value = match place {
                0 => json!({ "method": "session/prompt", "params": { name: &secret, "text": text } }),
                1 => json!({ "headers": [{ "name": "x", "value": format!("Bearer {secret}") }], "text": text }),
                _ => json!({ "request": { "headers": { name: format!("Basic {secret}") } }, "body": [text] }),
            };
            (value, secret)
        })
    }

    proptest! {
        #[test]
        fn no_key_or_authorization_reaches_the_log((value, secret) in record()) {
            let dir = tempfile::tempdir().unwrap();
            let log = RequestLog::new(dir.path(), "opsis-agent");
            log.write(&value).unwrap();
            let written = std::fs::read_to_string(log.path()).unwrap();
            prop_assert!(!written.contains(&secret), "{written}");
            prop_assert!(written.ends_with('\n'));
        }

        #[test]
        fn ordinary_values_are_written_as_they_are(text in "[a-z ]{0,20}", count in 0_u32..1000) {
            let value = json!({ "method": "session/update", "params": { "text": text, "count": count } });
            prop_assert_eq!(redacted(&value), value);
        }
    }

    #[test]
    fn rotates_by_size_and_keeps_a_fixed_number_of_files() {
        let dir = tempfile::tempdir().unwrap();
        let log = RequestLog::with_limit(dir.path(), "panel", 64);
        for index in 0..40 {
            log.write(&json!({ "line": index, "text": "a line long enough to rotate" }))
                .unwrap();
        }
        let files = log.files();
        assert_eq!(files.len(), KEPT);
        let newest = std::fs::read_to_string(log.path()).unwrap();
        assert!(newest.contains("\"line\":39"));
    }
}
