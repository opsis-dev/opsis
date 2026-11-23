use std::path::{Path, PathBuf};

const VERBATIM: &str = r"\\?\";
const VERBATIM_SHARE: &str = r"\\?\UNC\";

pub fn real_folder(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).map_or_else(|_| path.to_path_buf(), |real| plain(&real))
}

pub fn plain(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(share) = text.strip_prefix(VERBATIM_SHARE) {
        return PathBuf::from(format!(r"\\{share}"));
    }
    text.strip_prefix(VERBATIM)
        .map_or_else(|| path.to_path_buf(), PathBuf::from)
}
