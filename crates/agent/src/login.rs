use std::io::{BufRead, IsTerminal, Write};

use opsis_persistence::keys::{KeyError, Keyring};
use sqlx::SqlitePool;

use crate::settings::{self, ModelSettings, Service, SettingsError};

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error(transparent)]
    Key(#[from] KeyError),
    #[error("{0} needs a key.")]
    NoKey(&'static str),
    #[error("The login ended before it was finished.")]
    Ended,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answers {
    pub chosen: ModelSettings,
    pub key: Option<String>,
}

pub trait Asker {
    fn line(&mut self, prompt: &str) -> Option<String>;
    fn secret(&mut self, prompt: &str) -> Option<String>;
}

#[derive(Debug)]
pub struct Terminal;

fn read_line() -> Option<String> {
    std::io::stdin().lock().lines().next().and_then(Result::ok)
}

impl Asker for Terminal {
    fn line(&mut self, prompt: &str) -> Option<String> {
        eprint!("{prompt}");
        std::io::stderr().flush().ok();
        read_line()
    }

    fn secret(&mut self, prompt: &str) -> Option<String> {
        if std::io::stdin().is_terminal() {
            return rpassword::prompt_password(prompt).ok();
        }
        self.line(prompt)
    }
}

fn services() -> String {
    Service::ALL.map(Service::id).join(", ")
}

pub fn ask(asker: &mut impl Asker) -> Result<Answers, LoginError> {
    let service = asker
        .line(&format!("Model service ({}): ", services()))
        .ok_or(LoginError::Ended)?;
    let service = service.trim().to_owned();
    let model = asker.line("Model: ").ok_or(LoginError::Ended)?;
    let takes_address = Service::from_id(&service).is_some_and(Service::takes_address);
    let address = if takes_address {
        Some(asker.line("Server address: ").ok_or(LoginError::Ended)?)
    } else {
        None
    };
    let chosen = ModelSettings::new(&service, &model, address.as_deref())?;
    let key = if chosen.service.needs_key() {
        let key = asker.secret("Key: ").ok_or(LoginError::Ended)?;
        Some(key).filter(|key| !key.trim().is_empty())
    } else {
        None
    };
    Ok(Answers { chosen, key })
}

pub async fn store(
    pool: &SqlitePool,
    keys: Result<&Keyring, &KeyError>,
    answers: Answers,
) -> Result<(), LoginError> {
    let Answers { chosen, key } = answers;
    if chosen.service.needs_key() {
        let key = key.ok_or(LoginError::NoKey(chosen.service.id()))?;
        let keys = keys.map_err(Clone::clone)?.clone();
        let name = chosen.service.key_name();
        tokio::task::spawn_blocking(move || keys.set_key(&name, key.trim()))
            .await
            .map_err(|_| KeyError::NoKeyring)??;
    }
    settings::write(pool, &chosen).await?;
    Ok(())
}

pub async fn logout(pool: &SqlitePool, keys: &Keyring) -> Result<(), LoginError> {
    let keys = keys.clone();
    tokio::task::spawn_blocking(move || {
        Service::ALL
            .into_iter()
            .filter(|service| service.needs_key())
            .try_for_each(|service| keys.clear_key(&service.key_name()))
    })
    .await
    .map_err(|_| KeyError::NoKeyring)??;
    settings::forget(pool).await?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyState {
    Stored,
    Missing,
    Unreachable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub chosen: Option<ModelSettings>,
    pub key: KeyState,
}

pub async fn status(
    pool: &SqlitePool,
    keys: Result<Keyring, KeyError>,
) -> Result<Status, LoginError> {
    let chosen = settings::read(pool).await?;
    let key = match (&chosen, keys) {
        (Some(chosen), Ok(keys)) if chosen.service.needs_key() => {
            let name = chosen.service.key_name();
            match tokio::task::spawn_blocking(move || keys.key(&name)).await {
                Ok(Ok(Some(_))) => KeyState::Stored,
                Ok(Ok(None)) => KeyState::Missing,
                _ => KeyState::Unreachable,
            }
        }
        (Some(chosen), Err(_)) if chosen.service.needs_key() => KeyState::Unreachable,
        _ => KeyState::Missing,
    };
    Ok(Status { chosen, key })
}

pub fn status_text(status: &Status) -> String {
    let Some(chosen) = &status.chosen else {
        return "opsis-agent has no model yet. Run opsis-agent login to choose one.".to_owned();
    };
    let at = chosen
        .address
        .as_deref()
        .map_or_else(String::new, |address| format!(" at {address}"));
    let uses = format!(
        "opsis-agent uses {} on {}{at}.",
        chosen.model,
        chosen.service.id()
    );
    if !chosen.service.needs_key() {
        return uses;
    }
    let key = match status.key {
        KeyState::Stored => "Its key is in the system keyring.",
        KeyState::Missing => "Its key is missing; run opsis-agent login again.",
        KeyState::Unreachable => "The system keyring can't be reached, so its key can't be read.",
    };
    format!("{uses} {key}")
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    #[derive(Debug)]
    struct Typed(VecDeque<&'static str>);

    impl Asker for Typed {
        fn line(&mut self, _prompt: &str) -> Option<String> {
            self.0.pop_front().map(str::to_owned)
        }

        fn secret(&mut self, prompt: &str) -> Option<String> {
            self.line(prompt)
        }
    }

    fn typed(answers: &[&'static str]) -> Typed {
        Typed(answers.iter().copied().collect())
    }

    fn mock_keys() -> Keyring {
        Keyring::new(keyring_core::mock::Store::new().expect("mock store"))
    }

    #[tokio::test]
    async fn a_login_keeps_the_key_in_the_keyring_only() {
        let dir = tempfile::tempdir().unwrap();
        let pool = crate::db::open(dir.path()).await.unwrap();
        let keys = mock_keys();
        let answers = ask(&mut typed(&[
            "openrouter",
            "qwen/qwen3-coder",
            "sk-or-secret",
        ]))
        .unwrap();
        let chosen = answers.chosen.clone();
        store(&pool, Ok(&keys), answers).await.unwrap();
        assert_eq!(settings::read(&pool).await.unwrap(), Some(chosen.clone()));
        assert_eq!(
            keys.key(&chosen.service.key_name()).unwrap().as_deref(),
            Some("sk-or-secret")
        );
        pool.close().await;
        let file = crate::db::AGENT_DB_FILE;
        for name in [file.to_owned(), format!("{file}-wal")] {
            let bytes = std::fs::read(dir.path().join(name)).unwrap_or_default();
            assert!(!bytes.windows(12).any(|w| w == b"sk-or-secret"));
        }
    }

    #[tokio::test]
    async fn a_logout_forgets_the_model_and_removes_the_key() {
        let dir = tempfile::tempdir().unwrap();
        let pool = crate::db::open(dir.path()).await.unwrap();
        let keys = mock_keys();
        let answers = ask(&mut typed(&[
            "anthropic",
            "claude-sonnet-5-5",
            "sk-ant-secret",
        ]))
        .unwrap();
        let name = answers.chosen.service.key_name();
        store(&pool, Ok(&keys), answers).await.unwrap();
        let before = status(&pool, Ok(keys.clone())).await.unwrap();
        assert_eq!(before.key, KeyState::Stored);
        assert!(!status_text(&before).contains("sk-ant-secret"));
        logout(&pool, &keys).await.unwrap();
        assert_eq!(keys.key(&name).unwrap(), None);
        let after = status(&pool, Ok(keys)).await.unwrap();
        assert_eq!(after.chosen, None);
    }

    #[test]
    fn a_server_on_this_computer_asks_for_its_address_and_no_key() {
        let answers = ask(&mut typed(&["local", "qwen3", "http://127.0.0.1:8080/v1"])).unwrap();
        assert_eq!(
            answers.chosen.address.as_deref(),
            Some("http://127.0.0.1:8080/v1")
        );
        assert_eq!(answers.key, None);
    }

    #[tokio::test]
    async fn a_service_that_needs_a_key_is_refused_without_one() {
        let dir = tempfile::tempdir().unwrap();
        let pool = crate::db::open(dir.path()).await.unwrap();
        let answers = ask(&mut typed(&["anthropic", "claude-sonnet-5-5", " "])).unwrap();
        assert!(matches!(
            store(&pool, Ok(&mock_keys()), answers).await,
            Err(LoginError::NoKey(_))
        ));
        assert_eq!(settings::read(&pool).await.unwrap(), None);
    }

    #[tokio::test]
    async fn a_service_with_no_key_logs_in_without_a_keyring() {
        let dir = tempfile::tempdir().unwrap();
        let pool = crate::db::open(dir.path()).await.unwrap();
        let answers = ask(&mut typed(&["local", "qwen3", "http://127.0.0.1:8080/v1"])).unwrap();
        let chosen = answers.chosen.clone();
        store(&pool, Err(&KeyError::NoKeyring), answers)
            .await
            .unwrap();
        assert_eq!(settings::read(&pool).await.unwrap(), Some(chosen));
    }

    #[tokio::test]
    async fn a_service_that_needs_a_key_needs_the_keyring() {
        let dir = tempfile::tempdir().unwrap();
        let pool = crate::db::open(dir.path()).await.unwrap();
        let answers = ask(&mut typed(&[
            "anthropic",
            "claude-sonnet-5-5",
            "sk-ant-secret",
        ]))
        .unwrap();
        assert!(matches!(
            store(&pool, Err(&KeyError::NoKeyring), answers).await,
            Err(LoginError::Key(KeyError::NoKeyring))
        ));
        assert_eq!(settings::read(&pool).await.unwrap(), None);
    }

    #[test]
    fn a_login_cut_short_says_so() {
        assert!(matches!(
            ask(&mut typed(&["openai"])),
            Err(LoginError::Ended)
        ));
    }

    #[test]
    fn says_what_it_uses_and_never_the_key() {
        let chosen = |service: &str, address: Option<&str>| {
            Some(ModelSettings::new(service, "m1", address).unwrap())
        };
        assert_eq!(
            status_text(&Status {
                chosen: None,
                key: KeyState::Missing
            }),
            "opsis-agent has no model yet. Run opsis-agent login to choose one."
        );
        assert_eq!(
            status_text(&Status {
                chosen: chosen("local", None),
                key: KeyState::Missing
            }),
            "opsis-agent uses m1 on local at http://127.0.0.1:11434/v1."
        );
        assert_eq!(
            status_text(&Status {
                chosen: chosen("anthropic", None),
                key: KeyState::Stored
            }),
            "opsis-agent uses m1 on anthropic. Its key is in the system keyring."
        );
        assert_eq!(
            status_text(&Status {
                chosen: chosen("openai", None),
                key: KeyState::Missing
            }),
            "opsis-agent uses m1 on openai. Its key is missing; run opsis-agent login again."
        );
    }
}
