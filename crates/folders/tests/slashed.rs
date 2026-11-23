use std::path::PathBuf;

use opsis_folders::slashed;
use proptest::prelude::*;

proptest! {
    #[test]
    fn writes_a_relative_path_with_slashes_on_every_system(
        parts in prop::collection::vec("[a-zA-Z0-9_. -]{1,12}", 1..5),
    ) {
        let path: PathBuf = parts.iter().collect();
        prop_assert_eq!(slashed(&path), parts.join("/"));
    }
}
