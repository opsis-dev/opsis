use clap::{Args, Parser, Subcommand};

use crate::http::DEFAULT_HOST;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Stdio,
    Http { host: String, port: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub mode: Mode,
    pub full: bool,
    pub session: Option<String>,
}

fn non_empty(text: &str) -> Result<String, String> {
    if text.is_empty() {
        Err("it can't be empty".to_owned())
    } else {
        Ok(text.to_owned())
    }
}

#[derive(Debug, Parser)]
#[command(name = "opsis-mcp", version)]
#[doc = "Opsis's tools for coding agents over MCP. It reaches the running Opsis, which needs Settings → Assistant turned on."]
struct Cli {
    #[command(subcommand)]
    mode: Option<ModeCommand>,
    #[command(flatten)]
    shared: Shared,
}

#[derive(Debug, Args)]
struct Shared {
    #[arg(long, global = true)]
    #[doc = "Offer every Local API route as a tool, beside the four usual ones"]
    full: bool,
    #[arg(long, global = true, value_name = "ID", value_parser = non_empty)]
    #[doc = "Tag every request with this Agent panel session, so a run it starts reports back to it"]
    session: Option<String>,
}

#[derive(Debug, Subcommand)]
enum ModeCommand {
    #[doc = "Serve over standard input and output (the default)"]
    Stdio,
    #[doc = "Serve over Streamable HTTP at /mcp, behind a token printed at start"]
    Http {
        #[arg(long, default_value = DEFAULT_HOST, value_parser = non_empty)]
        #[doc = "The address to listen on; anything but loopback opens it to the network"]
        host: String,
        #[arg(long, default_value_t = 0)]
        #[doc = "The port to listen on; 0 picks a free one"]
        port: u16,
    },
}

pub fn parse<I, T>(args: I) -> Result<Options, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::try_parse_from(args)?;
    let mode = match cli.mode {
        None | Some(ModeCommand::Stdio) => Mode::Stdio,
        Some(ModeCommand::Http { host, port }) => Mode::Http { host, port },
    };
    Ok(Options {
        mode,
        full: cli.shared.full,
        session: cli.shared.session,
    })
}

#[cfg(test)]
mod tests {
    use clap::error::ErrorKind;

    use super::*;

    fn parsed(list: &[&str]) -> Result<Options, clap::Error> {
        parse(std::iter::once("opsis-mcp").chain(list.iter().copied()))
    }

    #[test]
    fn serves_stdio_by_default() {
        for list in [&[][..], &["stdio"]] {
            let options = parsed(list).unwrap();
            assert_eq!(options.mode, Mode::Stdio);
            assert!(!options.full);
            assert_eq!(options.session, None);
        }
    }

    #[test]
    fn serves_http_on_loopback_and_a_free_port_unless_told() {
        let mode = |list: &[&str]| parsed(list).unwrap().mode;
        assert_eq!(
            mode(&["http"]),
            Mode::Http {
                host: "127.0.0.1".into(),
                port: 0
            }
        );
        assert_eq!(
            mode(&["http", "--port", "4100", "--host", "0.0.0.0"]),
            Mode::Http {
                host: "0.0.0.0".into(),
                port: 4100
            }
        );
    }

    #[test]
    fn takes_full_and_a_session_in_either_mode() {
        for list in [
            &["--full", "--session", "s1"][..],
            &["stdio", "--session", "s1", "--full"],
            &["http", "--full", "--session", "s1"],
        ] {
            let options = parsed(list).unwrap();
            assert!(options.full, "{list:?}");
            assert_eq!(options.session.as_deref(), Some("s1"), "{list:?}");
        }
    }

    #[test]
    fn refuses_anything_else() {
        for list in [
            &["web"][..],
            &["stdio", "--port", "1"],
            &["--host", "x"],
            &["http", "--port"],
            &["http", "--port", "x"],
            &["http", "--host", ""],
            &["http", "--token", "a"],
            &["--session"],
            &["--session", ""],
            &["--full", "--full"],
        ] {
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
