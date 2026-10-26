use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde::de::DeserializeOwned;

fn missing(error: &std::io::Error) -> bool {
    matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory)
}

pub fn none_if_missing<T>(result: std::io::Result<T>) -> std::io::Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if missing(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn read_text_if_present(file: &Path) -> std::io::Result<Option<String>> {
    none_if_missing(std::fs::read_to_string(file))
}

pub fn read_dir_if_present(dir: &Path) -> std::io::Result<Vec<String>> {
    let Some(entries) = none_if_missing(std::fs::read_dir(dir))? else {
        return Ok(Vec::new());
    };
    entries
        .map(|e| e.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect()
}

enum Read<T> {
    Missing,
    Invalid,
    Valid(T),
}

fn read_valid_json<T: DeserializeOwned>(file: &Path) -> std::io::Result<Read<T>> {
    Ok(match read_text_if_present(file)? {
        None => Read::Missing,
        Some(text) => match serde_json::from_str(&text) {
            Ok(value) => Read::Valid(value),
            Err(_) => Read::Invalid,
        },
    })
}

pub fn read_json<T: DeserializeOwned>(file: &Path) -> std::io::Result<Option<T>> {
    Ok(match read_valid_json(file)? {
        Read::Valid(value) => Some(value),
        Read::Missing | Read::Invalid => None,
    })
}

pub fn read_json_keeping_broken<T: DeserializeOwned>(file: &Path) -> std::io::Result<Option<T>> {
    match read_valid_json(file)? {
        Read::Valid(value) => Ok(Some(value)),
        Read::Missing => Ok(None),
        Read::Invalid => {
            let broken = PathBuf::from(format!("{}.broken", file.display()));
            match std::fs::rename(file, &broken) {
                Ok(()) => log::warn!(
                    "{} couldn't be read and was moved to {}",
                    file.display(),
                    broken.display()
                ),
                Err(_) => log::warn!("{} couldn't be read", file.display()),
            }
            Ok(None)
        }
    }
}

fn partial_file<'a>() -> tempfile::Builder<'a, 'a> {
    let mut builder = tempfile::Builder::new();
    builder.prefix(".partial-");
    #[cfg(unix)]
    builder.permissions(std::os::unix::fs::PermissionsExt::from_mode(0o666));
    builder
}

pub fn atomic_write(file: &Path, data: &[u8]) -> std::io::Result<()> {
    let dir = file.parent().unwrap_or(Path::new("."));
    let mut temp = partial_file().tempfile_in(dir)?;
    temp.write_all(data)?;
    temp.as_file().sync_all()?;
    temp.persist(file).map_err(|e| e.error)?;
    Ok(())
}

pub async fn write_via_temp<T, E, F, Fut>(file: &Path, make: F) -> Result<T, E>
where
    E: From<std::io::Error>,
    F: FnOnce(PathBuf) -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let dir = file.parent().unwrap_or(Path::new("."));
    let ext = file
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let temp = partial_file()
        .suffix(&ext)
        .tempfile_in(dir)?
        .into_temp_path();
    let value = make(temp.to_path_buf()).await?;
    temp.persist(file).map_err(|e| e.error)?;
    Ok(value)
}

pub fn write_json(file: &Path, value: &impl Serialize) -> std::io::Result<()> {
    let text = serde_json::to_string_pretty(value).map_err(std::io::Error::other)?;
    atomic_write(file, format!("{text}\n").as_bytes())
}

#[derive(Default)]
pub struct Prune<'a> {
    pub keep: Option<&'a dyn Fn(&str) -> bool>,
    pub newest: usize,
    pub unused_for: Option<Duration>,
}

pub fn prune_dir(dir: &Path, prune: Prune<'_>) -> std::io::Result<()> {
    let mut files: Vec<(String, PathBuf, Option<SystemTime>)> = read_dir_if_present(dir)?
        .into_iter()
        .map(|name| {
            let file = dir.join(&name);
            let modified = std::fs::metadata(&file).and_then(|m| m.modified()).ok();
            (name, file, modified)
        })
        .collect();
    files.sort_by_key(|a| std::cmp::Reverse(a.2.unwrap_or(UNIX_EPOCH)));
    let cutoff = prune.unused_for.map(|d| SystemTime::now() - d);
    let unused = |modified: Option<SystemTime>| match cutoff {
        None => true,
        Some(cutoff) => modified.is_some_and(|m| m < cutoff),
    };
    for (name, file, modified) in files.into_iter().skip(prune.newest) {
        if !prune.keep.is_some_and(|keep| keep(&name))
            && unused(modified)
            && let Err(error) =
                std::fs::remove_file(&file).or_else(|_| std::fs::remove_dir_all(&file))
        {
            log::warn!("couldn't remove {}: {error}", file.display());
        }
    }
    Ok(())
}

pub fn modified_ms(meta: &std::fs::Metadata) -> f64 {
    meta.modified()
        .ok()
        .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
        .map_or(0.0, |d| d.as_secs_f64() * 1000.0)
}

pub fn stamp_of(meta: &std::fs::Metadata) -> String {
    format!("{}:{}", meta.len(), modified_ms(meta))
}

pub fn file_identity(file: &Path) -> std::io::Result<String> {
    let meta = std::fs::metadata(file)?;
    Ok(format!("{}:{}", meta.len(), modified_ms(&meta).round()))
}

#[cfg(test)]
mod tests {
    use filetime::{FileTime, set_file_mtime};
    use serde::Deserialize;

    use super::*;

    fn dir_with(ages: &[(&str, u64)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let now = SystemTime::now();
        for (name, age) in ages {
            let file = dir.path().join(name);
            std::fs::write(&file, name).unwrap();
            set_file_mtime(
                &file,
                FileTime::from_system_time(now - Duration::from_secs(*age)),
            )
            .unwrap();
        }
        dir
    }

    fn left(dir: &Path, names: &[&str]) -> Vec<String> {
        names
            .iter()
            .filter(|n| dir.join(n).exists())
            .map(|n| (*n).to_owned())
            .collect()
    }

    #[test]
    fn keeps_the_newest_files() {
        let dir = dir_with(&[("a", 30), ("b", 20), ("c", 10)]);
        prune_dir(
            dir.path(),
            Prune {
                newest: 2,
                ..Prune::default()
            },
        )
        .unwrap();
        assert_eq!(left(dir.path(), &["a", "b", "c"]), ["b", "c"]);
    }

    #[test]
    fn keeps_what_the_caller_names_and_anything_used_recently() {
        let dir = dir_with(&[("kept", 900), ("old", 900), ("fresh", 1)]);
        let keep = |name: &str| name == "kept";
        prune_dir(
            dir.path(),
            Prune {
                keep: Some(&keep),
                unused_for: Some(Duration::from_secs(600)),
                ..Prune::default()
            },
        )
        .unwrap();
        assert_eq!(
            left(dir.path(), &["kept", "old", "fresh"]),
            ["kept", "fresh"]
        );
    }

    #[test]
    fn prunes_nothing_in_a_missing_folder() {
        assert!(
            prune_dir(
                &std::env::temp_dir().join("opsis-dir-none"),
                Prune::default()
            )
            .is_ok()
        );
    }

    #[test]
    fn file_identity_changes_with_the_file_and_fails_for_a_missing_one() {
        let dir = dir_with(&[("a", 10)]);
        let file = dir.path().join("a");
        let before = file_identity(&file).unwrap();
        std::fs::write(&file, "longer text").unwrap();
        assert_ne!(file_identity(&file).unwrap(), before);
        assert!(file_identity(&dir.path().join("b")).is_err());
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct N {
        n: f64,
    }

    #[test]
    fn reads_what_the_type_accepts_and_nothing_for_a_missing_or_wrong_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("data.json");
        assert_eq!(read_json::<N>(&file).unwrap(), None);
        write_json(&file, &N { n: 1.0 }).unwrap();
        assert_eq!(read_json::<N>(&file).unwrap(), Some(N { n: 1.0 }));
        std::fs::write(&file, "{ broken").unwrap();
        assert_eq!(read_json::<N>(&file).unwrap(), None);
        std::fs::write(&file, r#"{"n":"one"}"#).unwrap();
        assert_eq!(read_json::<N>(&file).unwrap(), None);
    }

    #[test]
    fn moves_a_file_it_cant_read_aside_when_asked_to_keep_it() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("data.json");
        std::fs::write(&file, "{ broken").unwrap();
        assert_eq!(read_json_keeping_broken::<N>(&file).unwrap(), None);
        assert!(!file.exists());
        assert!(dir.path().join("data.json.broken").exists());
    }

    #[test]
    fn missing_files_and_folders_become_none_and_other_errors_pass_through() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("data.json");
        assert!(none_if_missing(std::fs::read_dir(&file)).unwrap().is_none());
        std::fs::write(&file, "x").unwrap();
        assert!(
            none_if_missing(std::fs::read_dir(file.join("inside")))
                .unwrap()
                .is_none()
        );
        assert!(none_if_missing::<()>(Err(std::io::Error::other("denied"))).is_err());
    }

    #[tokio::test]
    async fn writes_through_a_temporary_file_with_the_same_extension_and_removes_it_on_failure() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("stem.wav");
        let made = write_via_temp(&file, |temp| async move {
            assert_eq!(temp.extension().unwrap(), "wav");
            std::fs::write(&temp, "sound")?;
            Ok::<_, std::io::Error>(7)
        })
        .await
        .unwrap();
        assert_eq!(
            (made, std::fs::read_to_string(&file).unwrap().as_str()),
            (7, "sound")
        );
        let failed = write_via_temp(&dir.path().join("broken.wav"), |temp| async move {
            std::fs::write(&temp, "half")?;
            Err::<(), _>(std::io::Error::other("stopped"))
        })
        .await;
        assert!(failed.is_err());
        assert_eq!(read_dir_if_present(dir.path()).unwrap(), ["stem.wav"]);
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn writes_files_with_the_modes_of_any_new_file() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let mode = |name: &str| {
            std::fs::metadata(dir.path().join(name))
                .unwrap()
                .permissions()
                .mode()
                & 0o777
        };
        std::fs::write(dir.path().join("plain.txt"), "a").unwrap();
        atomic_write(&dir.path().join("atomic.txt"), b"a").unwrap();
        write_via_temp(&dir.path().join("made.mp4"), |temp| async move {
            std::fs::write(temp, "a")
        })
        .await
        .unwrap();
        assert_eq!(mode("atomic.txt"), mode("plain.txt"));
        assert_eq!(mode("made.mp4"), mode("plain.txt"));
    }

    #[test]
    fn atomic_write_leaves_no_temporary_file_behind_when_it_fails() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("data.json");
        std::fs::create_dir_all(file.join("inside")).unwrap();
        assert!(atomic_write(&file, b"x").is_err());
        assert_eq!(read_dir_if_present(dir.path()).unwrap(), ["data.json"]);
    }
}
