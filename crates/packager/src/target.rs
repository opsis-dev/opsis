use crate::error::PackagerError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub triple: String,
    pub os: &'static str,
    pub arch: &'static str,
}

impl Target {
    pub fn parse(triple: &str) -> Result<Self, PackagerError> {
        let unknown = || PackagerError::UnknownTarget(triple.to_owned());
        let arch = match triple.split('-').next() {
            Some("x86_64") => "x86_64",
            Some("aarch64") => "aarch64",
            _ => return Err(unknown()),
        };
        let os = if triple.contains("-linux-") {
            "linux"
        } else if triple.contains("-windows-") {
            "windows"
        } else if triple.contains("-apple-darwin") {
            "macos"
        } else {
            return Err(unknown());
        };
        Ok(Self {
            triple: triple.to_owned(),
            os,
            arch,
        })
    }

    pub const fn deb_arch(&self) -> &'static str {
        match self.arch.as_bytes() {
            b"aarch64" => "arm64",
            _ => "amd64",
        }
    }

    pub const fn windows_arch(&self) -> &'static str {
        match self.arch.as_bytes() {
            b"aarch64" => "arm64",
            _ => "x64",
        }
    }

    pub const fn exe(&self) -> &'static str {
        match self.os.as_bytes() {
            b"windows" => ".exe",
            _ => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_targets_opsis_ships() {
        let read = |triple: &str| {
            let target = Target::parse(triple).unwrap();
            (target.os, target.arch)
        };
        assert_eq!(read("x86_64-unknown-linux-gnu"), ("linux", "x86_64"));
        assert_eq!(read("aarch64-unknown-linux-gnu"), ("linux", "aarch64"));
        assert_eq!(read("x86_64-pc-windows-msvc"), ("windows", "x86_64"));
        assert_eq!(read("aarch64-apple-darwin"), ("macos", "aarch64"));
        assert_eq!(read("x86_64-apple-darwin"), ("macos", "x86_64"));
        assert!(Target::parse("riscv64gc-unknown-linux-gnu").is_err());
    }
}
