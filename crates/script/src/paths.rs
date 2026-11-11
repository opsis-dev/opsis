use std::path::{Component, Path, PathBuf};

use opsis_types::project::{META_DIR, OUT_DIR};
use opsis_types::workflow::WORKFLOW_FILE;

const PROJECT_SCHEME: &str = "project:";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PathRefusal {
    #[error("{0}: this path leads outside the project.")]
    Outside(String),
    #[error(
        "{0}: scripts can't change the project file or the project's .opsis folder, except .opsis/out."
    )]
    ReadOnly(String),
    #[error("A path can't be empty.")]
    Empty,
}

pub fn inside_project(path: &str, root: &Path) -> Result<PathBuf, PathRefusal> {
    if path.is_empty() {
        return Err(PathRefusal::Empty);
    }
    let named = path
        .strip_prefix(PROJECT_SCHEME)
        .map_or(path, |rest| rest.trim_start_matches('/'));
    let given = Path::new(named);
    let relative = if given.has_root() || given.is_absolute() {
        given
            .strip_prefix(root)
            .map_err(|_| PathRefusal::Outside(path.to_owned()))?
    } else {
        given
    };
    let mut kept = PathBuf::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => kept.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                if !kept.pop() {
                    return Err(PathRefusal::Outside(path.to_owned()));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(PathRefusal::Outside(path.to_owned()));
            }
        }
    }
    if kept.as_os_str().is_empty() {
        kept.push(".");
    }
    Ok(kept)
}

fn normal_parts(path: &Path) -> Vec<&std::ffi::OsStr> {
    path.components()
        .filter_map(|c| match c {
            Component::Normal(part) => Some(part),
            _ => None,
        })
        .collect()
}

pub fn writable(resolved: &Path) -> bool {
    let parts = normal_parts(resolved);
    if parts == normal_parts(Path::new(WORKFLOW_FILE)) {
        return false;
    }
    if !parts.starts_with(&normal_parts(Path::new(META_DIR))) {
        return true;
    }
    let out = normal_parts(Path::new(OUT_DIR));
    parts.starts_with(&out) && parts.len() > out.len()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn part() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("..".to_owned()),
            Just(".".to_owned()),
            Just(".opsis".to_owned()),
            Just("out".to_owned()),
            Just("cache".to_owned()),
            Just("project.opsis.json".to_owned()),
            "[a-z]{1,3}",
        ]
    }

    fn path() -> impl Strategy<Value = String> {
        (
            prop_oneof![Just(""), Just("project:"), Just("/p/"), Just("/elsewhere/")],
            proptest::collection::vec(part(), 1..6),
        )
            .prop_map(|(start, parts)| format!("{start}{}", parts.join("/")))
    }

    proptest! {
        #[test]
        fn no_accepted_path_leaves_the_root(given in path()) {
            if let Ok(kept) = inside_project(&given, Path::new("/p")) {
                prop_assert!(kept.components().all(|c| matches!(c, Component::Normal(_) | Component::CurDir)));
                prop_assert!(!kept.is_absolute());
            }
        }

        #[test]
        fn no_accepted_write_reaches_the_project_file_or_the_meta_folder_outside_out(given in path()) {
            if let Ok(kept) = inside_project(&given, Path::new("/p"))
                && writable(&kept)
            {
                let text = kept.to_string_lossy().replace('\\', "/");
                prop_assert_ne!(text.as_str(), WORKFLOW_FILE);
                prop_assert!(!text.starts_with(".opsis/") && text != ".opsis" || text.starts_with(".opsis/out/"));
            }
        }

        #[test]
        fn a_path_inside_reads_the_same_as_its_absolute_form(parts in proptest::collection::vec("[a-z]{1,3}", 1..4)) {
            let relative = parts.join("/");
            let absolute = format!("/p/{relative}");
            prop_assert_eq!(
                inside_project(&relative, Path::new("/p")),
                inside_project(&absolute, Path::new("/p"))
            );
        }
    }

    #[test]
    fn refuses_paths_above_the_root_and_outside_it() {
        let root = Path::new("/p");
        for given in ["../x", "a/../../x", "/etc/hostname", "/elsewhere/p/x"] {
            assert!(
                matches!(inside_project(given, root), Err(PathRefusal::Outside(_))),
                "{given}"
            );
        }
        assert_eq!(
            inside_project("project:/notes/a.txt", root).unwrap(),
            Path::new("notes/a.txt")
        );
    }
}
