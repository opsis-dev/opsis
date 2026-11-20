use std::path::{Path, PathBuf};

use opsis_persistence::keys::{KeyError, Keyring};
pub use opsis_types::secrets::secret_file;
use opsis_types::secrets::{SECRET_EXT, SECRETS_DIR};

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error(transparent)]
    Key(#[from] KeyError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn secrets_dir(project_dir: &Path) -> PathBuf {
    project_dir.join(SECRETS_DIR)
}

pub fn ensure_secrets_dir(project_dir: &Path) -> std::io::Result<()> {
    let dir = secrets_dir(project_dir);
    std::fs::create_dir_all(&dir)?;
    let ignore = dir.join(".gitignore");
    if !ignore.exists() {
        std::fs::write(ignore, "*\n!.gitignore\n")?;
    }
    Ok(())
}

pub fn write_secret(
    keys: &Keyring,
    project_dir: &Path,
    name: &str,
    value: &str,
) -> Result<(), SecretError> {
    let sealed = keys.seal(value.as_bytes())?;
    ensure_secrets_dir(project_dir)?;
    let file = secret_file(project_dir, name);
    let mut temp = tempfile::NamedTempFile::new_in(secrets_dir(project_dir))?;
    std::io::Write::write_all(&mut temp, &sealed)?;
    temp.persist(file).map_err(|e| e.error)?;
    Ok(())
}

pub fn list_secrets(project_dir: &Path) -> std::io::Result<Vec<String>> {
    let Ok(entries) = std::fs::read_dir(secrets_dir(project_dir)) else {
        return Ok(Vec::new());
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .filter_map(|e| {
            e.file_name()
                .to_string_lossy()
                .strip_suffix(SECRET_EXT)
                .map(str::to_owned)
        })
        .collect();
    names.sort();
    Ok(names)
}

pub fn read_secret(
    keys: &Keyring,
    project_dir: &Path,
    name: &str,
) -> Result<Option<String>, SecretError> {
    let sealed = match std::fs::read(secret_file(project_dir, name)) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let plain = keys.unseal(&sealed)?;
    Ok(Some(
        String::from_utf8(plain).map_err(|_| KeyError::SignInUnreadable)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_keyring() -> Keyring {
        Keyring::new(keyring_core::mock::Store::new().expect("mock store"))
    }

    #[test]
    fn writes_lists_and_reads_sign_ins_in_the_project() {
        let project = tempfile::tempdir().unwrap();
        let keys = test_keyring();
        assert_eq!(
            read_secret(&keys, project.path(), "Main account").unwrap(),
            None
        );
        write_secret(&keys, project.path(), "Main account", r#"{"cookies":[]}"#).unwrap();
        write_secret(&keys, project.path(), "", "{}").unwrap();
        assert_eq!(
            list_secrets(project.path()).unwrap(),
            ["default", "main-account"]
        );
        assert_eq!(
            read_secret(&keys, project.path(), "Main account")
                .unwrap()
                .as_deref(),
            Some(r#"{"cookies":[]}"#)
        );
        assert_eq!(
            std::fs::read_to_string(secrets_dir(project.path()).join(".gitignore")).unwrap(),
            "*\n!.gitignore\n"
        );
    }
}
