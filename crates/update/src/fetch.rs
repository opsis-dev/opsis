use std::io::Write;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use futures_util::StreamExt;
use minisign_verify::{PublicKey, Signature};
use tokio_util::sync::CancellationToken;

use crate::error::UpdateError;
use crate::feed::{Entry, Feed, feed_url, is_newer, release_page, this_computer};
use crate::installer::Installer;

const PUBLIC_KEY: &str = include_str!("../../../signing/opsis.key.pub");
const USER_AGENT: &str = concat!("Opsis/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Available {
    pub version: String,
    pub notes: String,
    pub page: String,
    pub file: Option<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Downloaded {
    pub installer: Installer,
    pub version: String,
    pub path: PathBuf,
}

pub fn client() -> Result<reqwest::Client, UpdateError> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|_| UpdateError::NoCertificates)
}

pub fn public_key() -> Result<PublicKey, UpdateError> {
    let decoded = STANDARD
        .decode(PUBLIC_KEY.trim())
        .map_err(|_| UpdateError::Signature)?;
    let text = String::from_utf8(decoded).map_err(|_| UpdateError::Signature)?;
    PublicKey::decode(&text).map_err(|_| UpdateError::Signature)
}

pub fn offered(
    feed: Feed,
    installer: Installer,
    running: &str,
) -> Result<Option<Available>, UpdateError> {
    if !is_newer(&feed.version, running).map_err(UpdateError::Version)? {
        return Ok(None);
    }
    let file = if installer.updates_itself() {
        let entry = feed
            .platforms
            .get(&this_computer(installer))
            .cloned()
            .ok_or_else(|| UpdateError::NoFile(feed.version.clone()))?;
        Some(entry)
    } else {
        None
    };
    Ok(Some(Available {
        page: release_page(&feed.version),
        version: feed.version,
        notes: feed.notes,
        file,
    }))
}

pub async fn check(
    client: &reqwest::Client,
    installer: Installer,
    running: &str,
) -> Result<Option<Available>, UpdateError> {
    let response = client
        .get(feed_url())
        .send()
        .await
        .map_err(UpdateError::Network)?;
    if !response.status().is_success() {
        return Err(UpdateError::Status(response.status().as_u16()));
    }
    let body = response.bytes().await.map_err(UpdateError::Network)?;
    let feed: Feed = serde_json::from_slice(&body).map_err(UpdateError::Unreadable)?;
    offered(feed, installer, running)
}

fn file_name(url: &str) -> String {
    url.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or("update")
        .to_owned()
}

pub async fn download(
    client: &reqwest::Client,
    entry: &Entry,
    key: &PublicKey,
    into: &Path,
    cancel: &CancellationToken,
    on_bytes: &mut (dyn FnMut(u64, Option<u64>) + Send),
) -> Result<PathBuf, UpdateError> {
    let signature = Signature::decode(&entry.signature).map_err(|_| UpdateError::Signature)?;
    let mut verifier = key
        .verify_stream(&signature)
        .map_err(|_| UpdateError::Signature)?;
    let response = client
        .get(&entry.url)
        .send()
        .await
        .map_err(UpdateError::Network)?;
    if !response.status().is_success() {
        return Err(UpdateError::Status(response.status().as_u16()));
    }
    let total = response.content_length();
    std::fs::create_dir_all(into)?;
    let target = into.join(file_name(&entry.url));
    let partial = into.join(format!("{}.part", file_name(&entry.url)));
    let mut file = std::fs::File::create(&partial)?;
    let mut got = 0u64;
    let mut body = response.bytes_stream();
    loop {
        let chunk = tokio::select! {
            () = cancel.cancelled() => {
                drop(file);
                std::fs::remove_file(&partial)?;
                return Err(UpdateError::Cancelled);
            }
            chunk = body.next() => chunk,
        };
        let Some(chunk) = chunk else { break };
        let chunk = chunk.map_err(UpdateError::Network)?;
        verifier.update(&chunk);
        file.write_all(&chunk)?;
        got += u64::try_from(chunk.len()).unwrap_or(u64::MAX);
        on_bytes(got, total);
    }
    file.sync_all()?;
    drop(file);
    if verifier.finalize().is_err() {
        std::fs::remove_file(&partial)?;
        return Err(UpdateError::Signature);
    }
    std::fs::rename(&partial, &target)?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn feed(version: &str, keys: &[String]) -> Feed {
        Feed {
            version: version.to_owned(),
            notes: "Notes".to_owned(),
            pub_date: String::new(),
            platforms: keys
                .iter()
                .map(|key| {
                    (
                        key.clone(),
                        Entry {
                            url: format!("https://example.test/{key}"),
                            signature: String::new(),
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        }
    }

    #[test]
    fn the_committed_public_key_reads() {
        assert!(public_key().is_ok());
    }

    #[test]
    fn nothing_is_offered_when_the_feed_is_not_newer() {
        let keys = [this_computer(Installer::AppImage)];
        assert_eq!(
            offered(feed("0.1.0", &keys), Installer::AppImage, "0.1.0").unwrap(),
            None
        );
    }

    #[test]
    fn a_self_updating_installer_gets_its_own_file() {
        let keys = [
            this_computer(Installer::Nsis),
            this_computer(Installer::Msi),
        ];
        let available = offered(feed("9.0.0", &keys), Installer::Msi, "0.1.0")
            .unwrap()
            .unwrap();
        assert_eq!(
            available.file.unwrap().url,
            format!("https://example.test/{}", this_computer(Installer::Msi))
        );
    }

    #[test]
    fn a_package_manager_install_gets_only_the_release_page() {
        let available = offered(feed("9.0.0", &[]), Installer::Deb, "0.1.0")
            .unwrap()
            .unwrap();
        assert_eq!(
            (available.file, available.page.as_str()),
            (
                None,
                "https://github.com/bismitpanda/opsis/releases/tag/v9.0.0"
            )
        );
    }

    struct Signed {
        key: PublicKey,
        signature: String,
        forged: String,
    }

    fn signed(bytes: &[u8]) -> Signed {
        let pair = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        let other = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        let sign = |sk: &minisign::SecretKey| {
            minisign::sign(None, sk, bytes, None, None)
                .unwrap()
                .into_string()
        };
        Signed {
            key: PublicKey::decode(&pair.pk.to_box().unwrap().into_string()).unwrap(),
            signature: sign(&pair.sk),
            forged: sign(&other.sk),
        }
    }

    async fn serve(bytes: Vec<u8>) -> wiremock::MockServer {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::path("/Opsis.AppImage"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_bytes(bytes))
            .mount(&server)
            .await;
        server
    }

    async fn fetch_signed(
        server: &wiremock::MockServer,
        key: &PublicKey,
        signature: String,
    ) -> (tempfile::TempDir, Result<PathBuf, UpdateError>) {
        let dir = tempfile::tempdir().unwrap();
        let entry = Entry {
            url: format!("{}/Opsis.AppImage", server.uri()),
            signature,
        };
        let got = download(
            &client().unwrap(),
            &entry,
            key,
            dir.path(),
            &CancellationToken::new(),
            &mut |_, _| {},
        )
        .await;
        (dir, got)
    }

    #[tokio::test]
    async fn a_download_signed_by_the_key_is_kept() {
        let bytes = b"the new program".repeat(1000);
        let signed = signed(&bytes);
        let server = serve(bytes.clone()).await;
        let (_dir, got) = fetch_signed(&server, &signed.key, signed.signature).await;
        assert_eq!(std::fs::read(got.unwrap()).unwrap(), bytes);
    }

    #[tokio::test]
    async fn a_download_signed_by_another_key_is_thrown_away() {
        let bytes = b"the new program".repeat(1000);
        let signed = signed(&bytes);
        let server = serve(bytes).await;
        let (dir, got) = fetch_signed(&server, &signed.key, signed.forged).await;
        assert!(matches!(got, Err(UpdateError::Signature)));
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[tokio::test]
    async fn a_changed_download_is_thrown_away() {
        let bytes = b"the new program".repeat(1000);
        let signed = signed(&bytes);
        let server = serve(b"something else".to_vec()).await;
        let (dir, got) = fetch_signed(&server, &signed.key, signed.signature).await;
        assert!(matches!(got, Err(UpdateError::Signature)));
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[test]
    fn a_missing_file_for_this_computer_is_an_error() {
        assert!(matches!(
            offered(feed("9.0.0", &[]), Installer::AppImage, "0.1.0"),
            Err(UpdateError::NoFile(version)) if version == "9.0.0"
        ));
    }
}
