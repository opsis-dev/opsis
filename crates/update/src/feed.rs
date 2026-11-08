use std::collections::BTreeMap;

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::installer::Installer;

pub const REPOSITORY: &str = "https://github.com/bismitpanda/opsis";
pub const FEED_FILE: &str = "latest.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Feed {
    pub version: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub pub_date: String,
    #[serde(default)]
    pub platforms: BTreeMap<String, Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub url: String,
    pub signature: String,
}

pub fn feed_url() -> String {
    format!("{REPOSITORY}/releases/latest/download/{FEED_FILE}")
}

pub fn release_page(version: &str) -> String {
    format!("{REPOSITORY}/releases/tag/v{version}")
}

pub fn feed_key(os: &str, arch: &str, installer: Installer) -> String {
    format!("{os}-{arch}-{}", installer.name())
}

pub fn this_computer(installer: Installer) -> String {
    feed_key(std::env::consts::OS, std::env::consts::ARCH, installer)
}

pub fn is_newer(offered: &str, running: &str) -> Result<bool, semver::Error> {
    Ok(Version::parse(offered)? > Version::parse(running)?)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn version() -> impl Strategy<Value = (u64, u64, u64)> {
        (0u64..50, 0u64..50, 0u64..50)
    }

    fn text((major, minor, patch): (u64, u64, u64)) -> String {
        format!("{major}.{minor}.{patch}")
    }

    proptest! {
        #[test]
        fn newer_follows_the_version_order(a in version(), b in version()) {
            prop_assert_eq!(is_newer(&text(a), &text(b)).unwrap(), a > b);
        }

        #[test]
        fn a_feed_reads_back_from_its_json(
            version in version(),
            notes in ".{0,40}",
            installer in prop::sample::select(Installer::ALL.to_vec()),
            url in "https://[a-z]{1,10}\\.test/[a-z]{1,10}",
            signature in "[A-Za-z0-9+/=\n: ]{1,80}",
        ) {
            let feed = Feed {
                version: text(version),
                notes,
                pub_date: String::new(),
                platforms: BTreeMap::from([(this_computer(installer), Entry { url, signature })]),
            };
            let read: Feed = serde_json::from_str(&serde_json::to_string(&feed).unwrap()).unwrap();
            prop_assert_eq!(read, feed);
        }
    }

    #[test]
    fn a_prerelease_is_older_than_its_release() {
        assert!(!is_newer("1.0.0-rc.1", "1.0.0").unwrap());
        assert!(is_newer("1.0.0", "1.0.0-rc.1").unwrap());
    }

    #[test]
    fn keys_name_the_system_the_chip_and_the_installer() {
        assert_eq!(
            feed_key("windows", "x86_64", Installer::Msi),
            "windows-x86_64-msi"
        );
    }
}
