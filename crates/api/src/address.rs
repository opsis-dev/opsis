use std::path::{Path, PathBuf};

pub const API_SOCKET_ENV: &str = "OPSIS_API_SOCKET";
pub const MAX_SOCKET_PATH_BYTES: usize = 100;
const SOCKET_FILE: &str = "api.sock";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiAddress {
    pub address: String,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct Host<'a> {
    pub windows: bool,
    pub user_data: &'a Path,
    pub home: &'a Path,
    pub username: &'a str,
    pub custom: Option<&'a str>,
}

fn example_socket(home: &Path) -> String {
    home.join(".opsis-api.sock").to_string_lossy().into_owned()
}

pub fn api_address(host: &Host<'_>) -> ApiAddress {
    if host.windows {
        return ApiAddress {
            address: format!(r"\\.\pipe\opsis-api-{}", host.username),
            error: None,
        };
    }
    let custom = host.custom.map(str::trim).filter(|c| !c.is_empty());
    let address = custom
        .map_or_else(|| host.user_data.join(SOCKET_FILE), PathBuf::from)
        .to_string_lossy()
        .into_owned();
    if custom.is_some_and(|c| !Path::new(c).is_absolute()) {
        let error = format!(
            "{API_SOCKET_ENV} has to be a full path, like {}. Change it and restart Opsis.",
            example_socket(host.home)
        );
        return ApiAddress {
            address,
            error: Some(error),
        };
    }
    let bytes = address.len();
    if bytes > MAX_SOCKET_PATH_BYTES {
        let error = format!(
            "The API address is {bytes} bytes long, and it can be at most {MAX_SOCKET_PATH_BYTES}. Set {API_SOCKET_ENV} to a shorter full path, like {}, and restart Opsis.",
            example_socket(host.home)
        );
        return ApiAddress {
            address,
            error: Some(error),
        };
    }
    ApiAddress {
        address,
        error: None,
    }
}
