use std::path::Path;

use opsis_folders::{plain, real_folder};
use proptest::prelude::*;

#[test]
fn every_spelling_of_one_folder_is_the_same_real_folder() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("projects").join("Reopen");
    std::fs::create_dir_all(&project).unwrap();
    let real = real_folder(&project);
    for spelling in [
        root.path().join("projects").join(".").join("Reopen"),
        root.path()
            .join("projects")
            .join("Reopen")
            .join("..")
            .join("Reopen"),
        root.path().join("projects/Reopen"),
    ] {
        assert_eq!(real_folder(&spelling), real, "{}", spelling.display());
    }
    assert!(!real.to_string_lossy().starts_with(r"\\?\"));
}

#[test]
fn a_folder_that_isnt_there_stays_as_given() {
    let gone = Path::new("/no/such/opsis/folder");
    assert_eq!(real_folder(gone), gone);
}

proptest! {
    #[test]
    fn a_plain_path_loses_only_the_verbatim_prefix(
        drive in "[A-Z]",
        parts in prop::collection::vec("[a-zA-Z0-9 _.-]{1,10}", 0..4),
    ) {
        let rest = parts.join(r"\");
        let local = format!(r"{drive}:\{rest}");
        prop_assert_eq!(plain(Path::new(&format!(r"\\?\{local}"))), Path::new(&local));
        prop_assert_eq!(plain(Path::new(&local)), Path::new(&local));
        let share = format!(r"server\share\{rest}");
        let plain_share = format!(r"\\{share}");
        prop_assert_eq!(plain(Path::new(&format!(r"\\?\UNC\{share}"))), Path::new(&plain_share));
    }
}
