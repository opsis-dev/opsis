use std::path::Path;

use opsis_api::address::{API_SOCKET_ENV, ApiAddress, Host, MAX_SOCKET_PATH_BYTES, api_address};
use proptest::prelude::*;

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

proptest! {
    #[test]
    fn takes_any_full_path_up_to_the_byte_limit_and_explains_the_rest(
        relative in any::<bool>(),
        name in "[a-zé_.-][a-zé_ .-]{0,79}",
        depth in 0usize..4,
        padding in "[ ]{0,3}",
    ) {
        let path = format!("{}{}{}", if relative { "" } else { "/" }, format!("{name}/").repeat(depth), name);
        let typed = format!("{padding}{path}{padding}");
        let trimmed = typed.trim();
        let found = api_address(&host(Some(&typed), USER_DATA));
        prop_assert_eq!(&found.address, trimmed);
        match found.error {
            None => prop_assert!(trimmed.starts_with('/') && trimmed.len() <= MAX_SOCKET_PATH_BYTES),
            Some(error) if !trimmed.starts_with('/') => prop_assert!(error.contains("has to be a full path"), "{}", error),
            Some(error) => {
                prop_assert!(trimmed.len() > MAX_SOCKET_PATH_BYTES);
                prop_assert!(error.contains(&format!("{} bytes long", trimmed.len())), "{}", error);
            }
        }
    }
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
