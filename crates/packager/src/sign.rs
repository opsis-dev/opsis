use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use minisign::{SecretKey, SecretKeyBox};
use opsis_update::feed::feed_key;
use opsis_update::installer::Installer;
use serde::Serialize;

use crate::error::{PackagerError, io_error};
use crate::target::Target;
use crate::tools::write;

pub const KEY_ENV: &str = "OPSIS_UPDATE_KEY";
pub const PASSWORD_ENV: &str = "OPSIS_UPDATE_KEY_PASSWORD";
pub const UPDATE_SUFFIX: &str = ".update.json";

#[derive(Debug, Serialize)]
pub struct UpdateFile {
    pub key: String,
    pub file: String,
    pub signature: String,
}

pub struct UpdateKey(SecretKey);

impl std::fmt::Debug for UpdateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdateKey").finish_non_exhaustive()
    }
}

pub fn decode_key(encoded: &str, password: Option<String>) -> Result<UpdateKey, PackagerError> {
    let bad = |error: &dyn std::fmt::Display| PackagerError::UpdateKey(error.to_string());
    let decoded = STANDARD.decode(encoded.trim()).map_err(|e| bad(&e))?;
    let text = String::from_utf8(decoded).map_err(|e| bad(&e))?;
    let boxed = SecretKeyBox::from_string(&text).map_err(|e| bad(&e))?;
    let opened = match password {
        Some(password) => boxed.into_secret_key(Some(password)),
        None => boxed.into_unencrypted_secret_key(),
    };
    Ok(UpdateKey(opened.map_err(|e| bad(&e))?))
}

pub fn key_from_env() -> Result<Option<UpdateKey>, PackagerError> {
    let Some(encoded) = std::env::var(KEY_ENV).ok().filter(|key| !key.is_empty()) else {
        return Ok(None);
    };
    let password = std::env::var(PASSWORD_ENV)
        .ok()
        .filter(|password| !password.is_empty());
    decode_key(&encoded, password).map(Some)
}

pub fn signature(key: &UpdateKey, file: &Path) -> Result<String, PackagerError> {
    let reader = std::fs::File::open(file).map_err(io_error(file))?;
    let name = file
        .file_name()
        .map_or_default(|name| name.to_string_lossy().into_owned());
    let trusted = format!("file:{name}");
    minisign::sign(None, &key.0, reader, Some(&trusted), None)
        .map(minisign::SignatureBox::into_string)
        .map_err(|error| PackagerError::UpdateKey(error.to_string()))
}

pub fn sign_update(
    key: &UpdateKey,
    file: &Path,
    installer: Installer,
    target: &Target,
) -> Result<Vec<PathBuf>, PackagerError> {
    let signature = signature(key, file)?;
    let name = file
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| PackagerError::Missing(file.to_owned()))?;
    let sig = file.with_file_name(format!("{name}.sig"));
    write(&sig, &signature)?;
    let record = UpdateFile {
        key: feed_key(target.os, target.arch, installer),
        file: name.clone(),
        signature,
    };
    let sidecar = file.with_file_name(format!("{name}{UPDATE_SUFFIX}"));
    let json = serde_json::to_string_pretty(&record)
        .map_err(|error| PackagerError::UpdateKey(error.to_string()))?;
    write(&sidecar, json)?;
    Ok(vec![sig, sidecar])
}

#[cfg(test)]
mod tests {
    use minisign::KeyPair;

    use super::*;

    fn encoded(pair: &KeyPair) -> String {
        STANDARD.encode(pair.sk.to_box(None).unwrap().into_string())
    }

    #[test]
    #[ignore = "needs OPSIS_UPDATE_KEY and OPSIS_UPDATE_KEY_PASSWORD from the ignored .env or the repository's secrets"]
    fn the_update_key_signs_what_the_committed_public_key_verifies() {
        let key = key_from_env().unwrap().expect("OPSIS_UPDATE_KEY is set");
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("Opsis.AppImage");
        std::fs::write(&file, b"the program").unwrap();
        let signature =
            minisign_verify::Signature::decode(&signature(&key, &file).unwrap()).unwrap();
        opsis_update::fetch::public_key()
            .unwrap()
            .verify(b"the program", &signature, false)
            .unwrap();
    }

    #[test]
    fn a_password_protected_key_opens_only_with_its_password() {
        let pair = KeyPair::generate_encrypted_keypair(Some("secret".to_owned())).unwrap();
        assert!(decode_key(&encoded(&pair), Some("secret".to_owned())).is_ok());
        assert!(decode_key(&encoded(&pair), Some("wrong".to_owned())).is_err());
        assert!(decode_key(&encoded(&pair), None).is_err());
    }

    #[test]
    fn a_file_signed_with_the_key_verifies_with_its_public_key() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("Opsis.AppImage");
        std::fs::write(&file, b"the program").unwrap();
        let pair = KeyPair::generate_unencrypted_keypair().unwrap();
        let key = decode_key(&encoded(&pair), None).unwrap();
        let target = Target::parse("x86_64-unknown-linux-gnu").unwrap();
        sign_update(&key, &file, Installer::AppImage, &target).unwrap();
        let text = std::fs::read_to_string(dir.path().join("Opsis.AppImage.sig")).unwrap();
        let public =
            minisign_verify::PublicKey::decode(&pair.pk.to_box().unwrap().into_string()).unwrap();
        let signature = minisign_verify::Signature::decode(&text).unwrap();
        public.verify(b"the program", &signature, false).unwrap();
        let record: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.path().join("Opsis.AppImage.update.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(record["key"], "linux-x86_64-appimage");
        assert_eq!(record["file"], "Opsis.AppImage");
    }

    #[test]
    fn a_key_that_isnt_one_is_refused() {
        assert!(decode_key("bm90IGEga2V5", None).is_err());
    }
}
