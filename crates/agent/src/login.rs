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

pub async fn store(pool: &SqlitePool, keys: &Keyring, answers: Answers) -> Result<(), LoginError> {
    let Answers { chosen, key } = answers;
    if chosen.service.needs_key() {
        let key = key.ok_or(LoginError::NoKey(chosen.service.id()))?;
        let keys = keys.clone();
        let name = chosen.service.key_name();
        tokio::task::spawn_blocking(move || keys.set_key(&name, key.trim()))
            .await
            .map_err(|_| KeyError::NoKeyring)??;
    }
    settings::write(pool, &chosen).await?;
    Ok(())
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
        store(&pool, &keys, answers).await.unwrap();
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
            store(&pool, &mock_keys(), answers).await,
            Err(LoginError::NoKey(_))
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
}
