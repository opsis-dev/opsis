use std::collections::BTreeSet;

use crate::error::PackagerError;
use crate::linking::Linked;

struct Provider {
    soname: &'static str,
    deb: &'static str,
    pacman: &'static str,
}

const fn provider(soname: &'static str, deb: &'static str, pacman: &'static str) -> Provider {
    Provider {
        soname,
        deb,
        pacman,
    }
}

const PROVIDERS: [Provider; 34] = [
    provider("ld-linux-x86-64.so.2", "libc6", "glibc"),
    provider("ld-linux-aarch64.so.1", "libc6", "glibc"),
    provider("libc.so.6", "libc6", "glibc"),
    provider("libm.so.6", "libc6", "glibc"),
    provider("libdl.so.2", "libc6", "glibc"),
    provider("libpthread.so.0", "libc6", "glibc"),
    provider("librt.so.1", "libc6", "glibc"),
    provider("libgcc_s.so.1", "libgcc-s1", "gcc-libs"),
    provider("libstdc++.so.6", "libstdc++6", "gcc-libs"),
    provider("libasound.so.2", "libasound2t64 | libasound2", "alsa-lib"),
    provider("libdbus-1.so.3", "libdbus-1-3", "dbus"),
    provider("libfontconfig.so.1", "libfontconfig1", "fontconfig"),
    provider("libfreetype.so.6", "libfreetype6", "freetype2"),
    provider("libwayland-client.so.0", "libwayland-client0", "wayland"),
    provider("libwayland-cursor.so.0", "libwayland-cursor0", "wayland"),
    provider("libX11.so.6", "libx11-6", "libx11"),
    provider("libX11-xcb.so.1", "libx11-xcb1", "libx11"),
    provider("libxcb.so.1", "libxcb1", "libxcb"),
    provider("libxcb-cursor.so.0", "libxcb-cursor0", "xcb-util-cursor"),
    provider("libxcb-icccm.so.4", "libxcb-icccm4", "xcb-util-wm"),
    provider("libxcb-image.so.0", "libxcb-image0", "xcb-util-image"),
    provider("libxcb-keysyms.so.1", "libxcb-keysyms1", "xcb-util-keysyms"),
    provider("libxcb-randr.so.0", "libxcb-randr0", "libxcb"),
    provider("libxcb-render.so.0", "libxcb-render0", "libxcb"),
    provider(
        "libxcb-render-util.so.0",
        "libxcb-render-util0",
        "xcb-util-renderutil",
    ),
    provider("libxcb-shape.so.0", "libxcb-shape0", "libxcb"),
    provider("libxcb-shm.so.0", "libxcb-shm0", "libxcb"),
    provider("libxcb-sync.so.1", "libxcb-sync1", "libxcb"),
    provider("libxcb-util.so.1", "libxcb-util1", "xcb-util"),
    provider("libxcb-xfixes.so.0", "libxcb-xfixes0", "libxcb"),
    provider("libxcb-xinput.so.0", "libxcb-xinput0", "libxcb"),
    provider("libxcb-xkb.so.1", "libxcb-xkb1", "libxcb"),
    provider("libxkbcommon.so.0", "libxkbcommon0", "libxkbcommon"),
    provider(
        "libxkbcommon-x11.so.0",
        "libxkbcommon-x11-0",
        "libxkbcommon-x11",
    ),
];

const CERTIFICATES: &str = "ca-certificates";

fn providers<'a>(linked: &'a Linked) -> Result<Vec<&'static Provider>, PackagerError> {
    let mut found = Vec::new();
    let mut unknown: Vec<&'a str> = Vec::new();
    for library in &linked.libraries {
        match PROVIDERS.iter().find(|entry| entry.soname == library) {
            Some(entry) => found.push(entry),
            None => unknown.push(library),
        }
    }
    if unknown.is_empty() {
        Ok(found)
    } else {
        Err(PackagerError::UnknownLibraries(
            unknown.into_iter().map(str::to_owned).collect(),
        ))
    }
}

pub fn deb_depends(linked: &Linked) -> Result<Vec<String>, PackagerError> {
    Ok(providers(linked)?
        .into_iter()
        .map(|entry| entry.deb)
        .chain([CERTIFICATES])
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect())
}

pub fn pacman_depends(linked: &Linked) -> Result<Vec<String>, PackagerError> {
    Ok(providers(linked)?
        .into_iter()
        .map(|entry| entry.pacman)
        .chain([CERTIFICATES])
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect())
}

pub fn rpm_requires(linked: &Linked) -> Result<Vec<String>, PackagerError> {
    providers(linked)?;
    let suffix = if linked.wide { "()(64bit)" } else { "" };
    Ok(linked
        .libraries
        .iter()
        .map(|library| format!("{library}{suffix}"))
        .chain([CERTIFICATES.to_owned()])
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect())
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn linked(libraries: &[&str]) -> Linked {
        Linked {
            libraries: libraries.iter().map(|name| (*name).to_owned()).collect(),
            runpaths: Vec::new(),
            wide: true,
        }
    }

    fn known() -> Vec<&'static str> {
        PROVIDERS.iter().map(|entry| entry.soname).collect()
    }

    #[test]
    fn each_library_has_one_provider() {
        let sonames: BTreeSet<&str> = known().into_iter().collect();
        assert_eq!(sonames.len(), PROVIDERS.len());
    }

    proptest! {
        #[test]
        fn every_library_a_program_loads_is_depended_on_once_in_order(
            libraries in prop::collection::vec(prop::sample::select(known()), 0..20),
        ) {
            let linked = linked(&libraries);
            for depends in [deb_depends(&linked).unwrap(), pacman_depends(&linked).unwrap()] {
                let mut sorted = depends.clone();
                sorted.sort();
                sorted.dedup();
                prop_assert_eq!(&depends, &sorted);
            }
            let deb = deb_depends(&linked).unwrap();
            let pacman = pacman_depends(&linked).unwrap();
            for library in &libraries {
                let entry = PROVIDERS.iter().find(|entry| entry.soname == *library).unwrap();
                prop_assert!(deb.iter().any(|name| name == entry.deb));
                prop_assert!(pacman.iter().any(|name| name == entry.pacman));
            }
            let requires = rpm_requires(&linked).unwrap();
            prop_assert_eq!(requires.len(), libraries.iter().collect::<BTreeSet<_>>().len() + 1);
            prop_assert!(requires.iter().all(|name| name == CERTIFICATES || name.ends_with("()(64bit)")));
            for depends in [&deb, &pacman, &requires] {
                prop_assert!(depends.iter().any(|name| name == CERTIFICATES));
            }
        }

        #[test]
        fn a_library_with_no_known_package_is_refused_by_name(
            libraries in prop::collection::vec(prop::sample::select(known()), 0..6),
            stranger in "lib[a-z]{3,8}\\.so\\.[0-9]",
        ) {
            prop_assume!(!known().contains(&stranger.as_str()));
            let mut all = libraries.clone();
            all.push(&stranger);
            let failed = deb_depends(&linked(&all));
            let refused = matches!(failed, Err(PackagerError::UnknownLibraries(names)) if names == vec![stranger.clone()]);
            prop_assert!(refused);
        }
    }
}
