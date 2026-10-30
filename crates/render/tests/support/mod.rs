use std::ffi::OsString;
use std::path::PathBuf;

pub fn sidecar_program() -> PathBuf {
    std::env::var_os("OPSIS_SIDECAR").map_or_else(|| PathBuf::from("bun"), PathBuf::from)
}

pub fn sidecar_args() -> Vec<OsString> {
    if std::env::var_os("OPSIS_SIDECAR").is_some() {
        Vec::new()
    } else {
        ["run", "apps/sidecar/src/main.ts"]
            .map(OsString::from)
            .to_vec()
    }
}
