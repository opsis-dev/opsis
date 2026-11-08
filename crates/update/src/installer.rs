use serde::{Deserialize, Serialize};

pub const MARKER_LEN: usize = 32;
const MARKER_PREFIX: &[u8] = b"opsis-installer:";
const UNMARKED_NAME: &[u8] = b"unmarked";

pub const UNMARKED: [u8; MARKER_LEN] = marker_for(UNMARKED_NAME);

#[used]
static INSTALLED_FROM: [u8; MARKER_LEN] = UNMARKED;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Installer {
    Deb,
    Rpm,
    Pacman,
    AppImage,
    Nsis,
    Msi,
    App,
}

impl Installer {
    pub const ALL: [Self; 7] = [
        Self::Deb,
        Self::Rpm,
        Self::Pacman,
        Self::AppImage,
        Self::Nsis,
        Self::Msi,
        Self::App,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Deb => "deb",
            Self::Rpm => "rpm",
            Self::Pacman => "pacman",
            Self::AppImage => "appimage",
            Self::Nsis => "nsis",
            Self::Msi => "msi",
            Self::App => "app",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Deb => "deb package",
            Self::Rpm => "rpm package",
            Self::Pacman => "pacman package",
            Self::AppImage => "AppImage",
            Self::Nsis => "Windows installer",
            Self::Msi => "Windows Installer package (MSI)",
            Self::App => "macOS app",
        }
    }

    pub const fn updates_itself(self) -> bool {
        matches!(self, Self::AppImage | Self::Nsis | Self::Msi | Self::App)
    }

    pub const fn marker(self) -> [u8; MARKER_LEN] {
        marker_for(self.name().as_bytes())
    }
}

const fn marker_for(name: &[u8]) -> [u8; MARKER_LEN] {
    let mut marker = [b'-'; MARKER_LEN];
    let mut at = 0;
    while at < MARKER_PREFIX.len() {
        marker[at] = MARKER_PREFIX[at];
        at += 1;
    }
    let mut from = 0;
    while from < name.len() {
        marker[MARKER_PREFIX.len() + from] = name[from];
        from += 1;
    }
    marker
}

pub fn installed_from() -> Option<Installer> {
    read_marker(std::hint::black_box(&INSTALLED_FROM))
}

pub fn read_marker(bytes: &[u8; MARKER_LEN]) -> Option<Installer> {
    Installer::ALL
        .into_iter()
        .find(|installer| installer.marker() == *bytes)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn a_marker_reads_back_as_its_installer(installer in prop::sample::select(Installer::ALL.to_vec())) {
            prop_assert_eq!(read_marker(&installer.marker()), Some(installer));
        }
    }

    #[test]
    fn markers_are_distinct_and_never_the_unmarked_one() {
        let mut markers: Vec<_> = Installer::ALL.iter().map(|i| i.marker()).collect();
        markers.push(UNMARKED);
        let count = markers.len();
        markers.sort_unstable();
        markers.dedup();
        assert_eq!(markers.len(), count);
    }

    #[test]
    fn an_unpatched_program_has_no_installer() {
        assert_eq!(installed_from(), None);
    }
}
