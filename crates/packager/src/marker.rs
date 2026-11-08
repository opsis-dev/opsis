use std::path::{Path, PathBuf};

use opsis_update::installer::{Installer, UNMARKED};

use crate::error::{PackagerError, io_error};
use crate::tools::copy;

const PROGRAM_MODE: u32 = 0o755;

fn positions(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    haystack
        .windows(needle.len())
        .enumerate()
        .filter(|(_, window)| *window == needle)
        .map(|(at, _)| at)
        .collect()
}

pub fn mark(bytes: &mut [u8], installer: Installer) -> Result<(), usize> {
    let found = positions(bytes, &UNMARKED);
    let [at] = found[..] else {
        return Err(found.len());
    };
    bytes[at..at + UNMARKED.len()].copy_from_slice(&installer.marker());
    Ok(())
}

pub fn marked_program(
    program: &Path,
    installer: Installer,
    into: &Path,
) -> Result<PathBuf, PackagerError> {
    let name = program
        .file_name()
        .ok_or_else(|| PackagerError::Missing(program.to_owned()))?;
    let target = into.join(name);
    copy(program, &target, PROGRAM_MODE)?;
    let mut bytes = std::fs::read(&target).map_err(io_error(&target))?;
    mark(&mut bytes, installer).map_err(|found| PackagerError::Marker {
        program: program.to_owned(),
        found,
    })?;
    std::fs::write(&target, bytes).map_err(io_error(&target))?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use opsis_update::installer::{MARKER_LEN, read_marker};
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn a_marked_program_reads_back_its_installer_and_keeps_every_other_byte(
            before in prop::collection::vec(any::<u8>(), 0..200),
            after in prop::collection::vec(any::<u8>(), 0..200),
            installer in prop::sample::select(Installer::ALL.to_vec()),
        ) {
            prop_assume!(positions(&[before.as_slice(), &UNMARKED[..], after.as_slice()].concat(), &UNMARKED).len() == 1);
            let mut bytes = [before.as_slice(), &UNMARKED[..], after.as_slice()].concat();
            mark(&mut bytes, installer).unwrap();
            let marker: [u8; MARKER_LEN] = bytes[before.len()..before.len() + MARKER_LEN].try_into().unwrap();
            prop_assert_eq!(read_marker(&marker), Some(installer));
            prop_assert_eq!(&bytes[..before.len()], before.as_slice());
            prop_assert_eq!(&bytes[before.len() + MARKER_LEN..], after.as_slice());
        }
    }

    #[test]
    fn a_program_without_the_marker_or_with_two_is_refused() {
        assert_eq!(
            mark(&mut b"no marker here".to_vec(), Installer::Deb),
            Err(0)
        );
        let mut twice = [&UNMARKED[..], &UNMARKED[..]].concat();
        assert_eq!(mark(&mut twice, Installer::Deb), Err(2));
    }
}
