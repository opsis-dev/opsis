use std::path::Path;

use opsis_api::address::{API_SOCKET_ENV, ApiAddress, Host, MAX_SOCKET_PATH_BYTES, api_address};

fn host<'a>(custom: Option<&'a str>, user_data: &'a str) -> Host<'a> {
    Host {
        windows: false,
        user_data: Path::new(user_data),
        home: Path::new("/home/ada"),
        username: "ada",
        custom,
    }
}

const USER_DATA: &str = "/home/ada/.config/Opsis";

fn path_of_bytes(bytes: usize) -> String {
    format!("/{}/api.sock", "a".repeat(bytes - "/api.sock".len() - 1))
}

#[test]
fn uses_the_socket_in_the_apps_data_folder_by_default() {
    assert_eq!(
        api_address(&host(None, USER_DATA)),
        ApiAddress {
            address: "/home/ada/.config/Opsis/api.sock".into(),
            error: None
        }
    );
}

#[test]
fn uses_the_path_the_environment_variable_names() {
    assert_eq!(
        api_address(&host(Some(" /run/user/1000/df.sock "), USER_DATA)),
        ApiAddress {
            address: "/run/user/1000/df.sock".into(),
            error: None
        }
    );
}

#[test]
fn allows_exactly_the_maximum_length_and_refuses_one_byte_more() {
    let fits = path_of_bytes(MAX_SOCKET_PATH_BYTES);
    let over = path_of_bytes(MAX_SOCKET_PATH_BYTES + 1);
    assert_eq!(api_address(&host(Some(&fits), USER_DATA)).error, None);
    assert!(
        api_address(&host(Some(&over), USER_DATA))
            .error
            .unwrap()
            .contains(&format!(
                "101 bytes long, and it can be at most {MAX_SOCKET_PATH_BYTES}"
            ))
    );
}

#[test]
fn counts_a_long_default_path_and_says_how_to_fix_it() {
    let deep = format!("/home/ada/{}Opsis", "deep/".repeat(20));
    let error = api_address(&host(None, &deep)).error.unwrap();
    assert!(
        error.contains(&format!(
            "Set {API_SOCKET_ENV} to a shorter full path, like /home/ada/.opsis-api.sock"
        )),
        "{error}"
    );
}

#[test]
fn counts_bytes_not_characters() {
    let address = format!("/home/ada/{}/api.sock", "é".repeat(45));
    assert!(address.chars().count() <= MAX_SOCKET_PATH_BYTES);
    assert!(
        api_address(&host(Some(&address), USER_DATA))
            .error
            .unwrap()
            .contains("bytes long")
    );
}

#[test]
fn refuses_a_relative_path() {
    assert!(
        api_address(&host(Some("df.sock"), USER_DATA))
            .error
            .unwrap()
            .contains("has to be a full path")
    );
}

#[test]
fn uses_the_named_pipe_on_windows_and_ignores_the_variable() {
    let windows = Host {
        windows: true,
        ..host(Some("/x.sock"), USER_DATA)
    };
    assert_eq!(
        api_address(&windows),
        ApiAddress {
            address: r"\\.\pipe\opsis-api-ada".into(),
            error: None
        }
    );
}
