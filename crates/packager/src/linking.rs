use std::path::Path;

use goblin::Object;
use goblin::mach::Mach;

use crate::error::{PackagerError, io_error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Linked {
    pub libraries: Vec<String>,
    pub runpaths: Vec<String>,
    pub wide: bool,
}

const BUNDLED_LIBRARIES: [&str; 4] = ["qt6", "qtcore", "libicu", "kddockwidgets"];
const RELATIVE_RUNPATHS: [&str; 3] = ["$ORIGIN", "@executable_path", "@loader_path"];

pub fn read_linked(program: &Path) -> Result<Linked, PackagerError> {
    let bytes = std::fs::read(program).map_err(io_error(program))?;
    let parsed = Object::parse(&bytes).map_err(|source| PackagerError::Parse {
        path: program.to_owned(),
        source,
    })?;
    match parsed {
        Object::Elf(elf) => Ok(Linked {
            libraries: elf
                .libraries
                .iter()
                .map(|name| (*name).to_owned())
                .collect(),
            runpaths: elf
                .runpaths
                .iter()
                .chain(&elf.rpaths)
                .flat_map(|list| list.split(':'))
                .map(str::to_owned)
                .collect(),
            wide: elf.is_64,
        }),
        Object::Mach(Mach::Binary(macho)) => Ok(Linked {
            libraries: macho
                .libs
                .iter()
                .filter(|name| **name != "self")
                .map(|name| (*name).to_owned())
                .collect(),
            runpaths: macho.rpaths.iter().map(|path| (*path).to_owned()).collect(),
            wide: macho.is_64,
        }),
        Object::PE(pe) => Ok(Linked {
            libraries: pe.libraries.iter().map(|name| (*name).to_owned()).collect(),
            runpaths: Vec::new(),
            wide: pe.is_64,
        }),
        _ => Err(PackagerError::NotAProgram(program.to_owned())),
    }
}

pub fn check_self_contained(program: &Path, linked: &Linked) -> Result<(), PackagerError> {
    let bundled: Vec<String> = linked
        .libraries
        .iter()
        .filter(|library| {
            let lower = library.to_lowercase();
            BUNDLED_LIBRARIES.iter().any(|name| lower.contains(name))
        })
        .cloned()
        .collect();
    if !bundled.is_empty() {
        return Err(PackagerError::LinksQt {
            program: program.to_owned(),
            libraries: bundled,
        });
    }
    let absolute: Vec<String> = linked
        .runpaths
        .iter()
        .filter(|path| {
            !path.is_empty()
                && !RELATIVE_RUNPATHS
                    .iter()
                    .any(|prefix| path.starts_with(prefix))
        })
        .cloned()
        .collect();
    if absolute.is_empty() {
        Ok(())
    } else {
        Err(PackagerError::AbsoluteRunpath {
            program: program.to_owned(),
            runpaths: absolute,
        })
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    const SYSTEM: [&str; 6] = [
        "libc.so.6",
        "libstdc++.so.6",
        "libX11.so.6",
        "libxcb-cursor.so.0",
        "libfontconfig.so.1",
        "KERNEL32.dll",
    ];
    const BUNDLED: [&str; 5] = [
        "libQt6Core.so.6",
        "libQt6Widgets.so.6",
        "@rpath/QtCore.framework/Versions/A/QtCore",
        "libicudata.so.73",
        "libkddockwidgets-qt6.so.2.5",
    ];

    fn linked(libraries: Vec<&str>, runpaths: Vec<&str>) -> Linked {
        Linked {
            libraries: libraries.into_iter().map(str::to_owned).collect(),
            runpaths: runpaths.into_iter().map(str::to_owned).collect(),
            wide: true,
        }
    }

    proptest! {
        #[test]
        fn a_program_loading_only_system_libraries_passes(
            libraries in prop::sample::subsequence(SYSTEM.to_vec(), 0..=SYSTEM.len()),
            runpaths in prop::sample::subsequence(vec!["$ORIGIN", "$ORIGIN/../lib", "@executable_path/../Frameworks"], 0..=3),
        ) {
            prop_assert!(check_self_contained(Path::new("opsis"), &linked(libraries, runpaths)).is_ok());
        }

        #[test]
        fn a_program_loading_qt_icu_or_the_docks_is_refused_naming_them(
            system in prop::sample::subsequence(SYSTEM.to_vec(), 0..=SYSTEM.len()),
            bundled in prop::sample::subsequence(BUNDLED.to_vec(), 1..=BUNDLED.len()),
        ) {
            let libraries: Vec<&str> = system.iter().chain(&bundled).copied().collect();
            match check_self_contained(Path::new("opsis"), &linked(libraries, Vec::new())) {
                Err(PackagerError::LinksQt { libraries, .. }) => {
                    prop_assert_eq!(libraries, bundled.iter().map(|name| (*name).to_owned()).collect::<Vec<_>>());
                }
                other => prop_assert!(false, "{other:?}"),
            }
        }

        #[test]
        fn an_absolute_runpath_is_refused(folder in "/[a-z]{1,8}(/[a-z]{1,8}){0,3}") {
            let result = check_self_contained(
                Path::new("opsis"),
                &linked(vec!["libc.so.6"], vec!["$ORIGIN", &folder]),
            );
            let refused = matches!(result, Err(PackagerError::AbsoluteRunpath { runpaths, .. }) if runpaths == vec![folder]);
            prop_assert!(refused);
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn reads_the_libraries_a_real_program_loads() {
        let linked = read_linked(Path::new("/usr/bin/true")).expect("an ELF program");
        assert!(
            linked.libraries.iter().any(|name| name == "libc.so.6"),
            "{linked:?}"
        );
    }
}
