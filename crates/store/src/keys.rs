use std::sync::Arc;

use aes_gcm::aead::{Aead, Generate, Key, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use keyring_core::{CredentialStore, Error as KeyringError};

pub const SERVICE: &str = "Opsis";
const SIGN_IN_KEY: &str = "sign-ins";
const NONCE_LEN: usize = 12;

#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    #[error(
        "This computer has no keyring to protect keys, so they can't be saved. Set up a keyring and try again."
    )]
    NoKeyring,
    #[error("This computer has no secure storage available, so saved sign-ins can't be used.")]
    NoSecureStorage,
    #[error("This key can't be read on this computer. Enter it again.")]
    Unreadable,
    #[error("This saved sign-in can't be read on this computer. Sign in again.")]
    SignInUnreadable,
}

fn storage(error: KeyringError) -> KeyError {
    match error {
        KeyringError::NoStorageAccess(_) | KeyringError::PlatformFailure(_) => {
            KeyError::NoSecureStorage
        }
        _ => KeyError::NoKeyring,
    }
}

#[derive(Clone)]
pub struct Keyring {
    store: Arc<CredentialStore>,
}

impl Keyring {
    pub fn new(store: Arc<CredentialStore>) -> Self {
        Self { store }
    }

    #[cfg(target_os = "macos")]
    pub fn platform() -> Result<Self, KeyError> {
        Ok(Self::new(
            apple_native_keyring_store::keychain::Store::new().map_err(|_| KeyError::NoKeyring)?,
        ))
    }

    #[cfg(target_os = "windows")]
    pub fn platform() -> Result<Self, KeyError> {
        Ok(Self::new(
            windows_native_keyring_store::Store::new().map_err(|_| KeyError::NoKeyring)?,
        ))
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    pub fn platform() -> Result<Self, KeyError> {
        Ok(Self::new(
            zbus_secret_service_keyring_store::Store::new().map_err(|_| KeyError::NoKeyring)?,
        ))
    }

    fn entry(&self, user: &str) -> Result<keyring_core::Entry, KeyError> {
        self.store.build(SERVICE, user, None).map_err(storage)
    }

    pub fn set_key(&self, name: &str, value: &str) -> Result<(), KeyError> {
        self.entry(&format!("key:{name}"))?
            .set_password(value)
            .map_err(storage)
    }

    pub fn key(&self, name: &str) -> Result<Option<String>, KeyError> {
        match self.entry(&format!("key:{name}"))?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(KeyringError::BadEncoding(_)) => Err(KeyError::Unreadable),
            Err(e) => Err(storage(e)),
        }
    }

    fn sign_in_key(&self) -> Result<Key<Aes256Gcm>, KeyError> {
        let entry = self.entry(SIGN_IN_KEY)?;
        match entry.get_password() {
            Ok(text) => {
                let bytes = STANDARD.decode(text).map_err(|_| KeyError::Unreadable)?;
                Key::<Aes256Gcm>::try_from(bytes.as_slice()).map_err(|_| KeyError::Unreadable)
            }
            Err(KeyringError::NoEntry) => {
                let key = Key::<Aes256Gcm>::generate();
                entry.set_password(&STANDARD.encode(key)).map_err(storage)?;
                Ok(key)
            }
            Err(e) => Err(storage(e)),
        }
    }

    pub fn seal(&self, plain: &[u8]) -> Result<Vec<u8>, KeyError> {
        let cipher = Aes256Gcm::new(&self.sign_in_key()?);
        let nonce = Nonce::generate();
        let sealed = cipher
            .encrypt(&nonce, plain)
            .map_err(|_| KeyError::NoSecureStorage)?;
        Ok(nonce.iter().copied().chain(sealed).collect())
    }

    pub fn unseal(&self, sealed: &[u8]) -> Result<Vec<u8>, KeyError> {
        if sealed.len() < NONCE_LEN {
            return Err(KeyError::SignInUnreadable);
        }
        let (nonce, body) = sealed.split_at(NONCE_LEN);
        let nonce = Nonce::try_from(nonce).map_err(|_| KeyError::SignInUnreadable)?;
        Aes256Gcm::new(&self.sign_in_key()?)
            .decrypt(&nonce, body)
            .map_err(|_| KeyError::SignInUnreadable)
    }
}

#[cfg(test)]
pub(crate) fn test_keyring() -> Keyring {
    Keyring::new(keyring_core::mock::Store::new().expect("mock store"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seals_and_unseals_with_one_key() {
        let keys = test_keyring();
        let sealed = keys.seal(b"storage state").unwrap();
        assert!(!sealed.windows(7).any(|w| w == b"storage"));
        assert_eq!(keys.unseal(&sealed).unwrap(), b"storage state");
    }

    #[test]
    fn refuses_what_another_key_sealed() {
        let sealed = test_keyring().seal(b"secret").unwrap();
        assert!(matches!(
            test_keyring().unseal(&sealed),
            Err(KeyError::SignInUnreadable)
        ));
    }

    #[test]
    fn keeps_small_keys_in_the_keyring() {
        let keys = test_keyring();
        assert_eq!(keys.key("voice").unwrap(), None);
        keys.set_key("voice", "sk-test").unwrap();
        assert_eq!(keys.key("voice").unwrap().as_deref(), Some("sk-test"));
    }

    #[test]
    fn refuses_to_seal_without_secure_storage_or_a_keyring() {
        for (error, expected) in [
            (
                KeyringError::NoStorageAccess(std::io::Error::other("locked").into()),
                "no secure storage",
            ),
            (KeyringError::NoDefaultStore, "no keyring"),
        ] {
            let store = keyring_core::mock::Store::new().expect("mock store");
            let keys = Keyring::new(store.clone());
            let entry = keyring_core::api::CredentialStoreApi::build(
                store.as_ref(),
                SERVICE,
                SIGN_IN_KEY,
                None,
            )
            .unwrap();
            let cred: &keyring_core::mock::Cred = entry.as_any().downcast_ref().unwrap();
            cred.set_error(error);
            let refused = keys.seal(b"state").unwrap_err().to_string();
            assert!(refused.contains(expected), "{refused}");
        }
    }
}
