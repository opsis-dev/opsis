use clap::{Parser, Subcommand};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum Command {
    #[doc = "Choose the model service, the model and its key"]
    Login,
    #[doc = "Forget the chosen model and remove the keys from the system keyring"]
    Logout,
    #[doc = "Show the model service, model and address it uses, and whether a key is stored"]
    Status,
}

#[derive(Debug, Parser)]
#[command(name = "opsis-agent", version)]
#[doc = "Opsis's own agent. With no command it speaks ACP on standard input and output, for the Agent panel or any ACP client."]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub fn parse<I, T>(args: I) -> Result<Option<Command>, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    Ok(Cli::try_parse_from(args)?.command)
}

#[cfg(test)]
mod tests {
    use clap::error::ErrorKind;

    use super::*;

    fn parsed(list: &[&str]) -> Result<Option<Command>, clap::Error> {
        parse(std::iter::once("opsis-agent").chain(list.iter().copied()))
    }

    #[test]
    fn serves_with_nothing_and_runs_the_command_it_is_given() {
        assert_eq!(parsed(&[]).unwrap(), None);
        assert_eq!(parsed(&["login"]).unwrap(), Some(Command::Login));
        assert_eq!(parsed(&["logout"]).unwrap(), Some(Command::Logout));
        assert_eq!(parsed(&["status"]).unwrap(), Some(Command::Status));
    }

    #[test]
    fn refuses_anything_else() {
        for list in [&["serve"][..], &["set"], &["login", "openai"], &["--login"]] {
            assert!(parsed(list).is_err(), "{list:?}");
        }
    }

    #[test]
    fn answers_help_and_version() {
        assert_eq!(
            parsed(&["--help"]).unwrap_err().kind(),
            ErrorKind::DisplayHelp
        );
        assert_eq!(
            parsed(&["--version"]).unwrap_err().kind(),
            ErrorKind::DisplayVersion
        );
    }
}
