use crate::http::DEFAULT_HOST;

pub const USAGE: &str = "Usage: opsis-mcp [stdio] [--full] [--session <id>]\n       opsis-mcp http [--host <host>] [--port <port>] [--full] [--session <id>]";

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

fn options(http: bool, flags: &[String]) -> Option<Options> {
    let mut host = DEFAULT_HOST.to_owned();
    let mut port = 0;
    let mut full = false;
    let mut session = None;
    let mut rest = flags.iter();
    while let Some(flag) = rest.next() {
        match flag.as_str() {
            "--full" if !full => full = true,
            "--session" if session.is_none() => {
                session = Some(rest.next().filter(|v| !v.is_empty())?.clone());
            }
            "--host" if http => rest.next().filter(|v| !v.is_empty())?.clone_into(&mut host),
            "--port" if http => port = rest.next()?.parse().ok()?,
            _ => return None,
        }
    }
    let mode = if http {
        Mode::Http { host, port }
    } else {
        Mode::Stdio
    };
    Some(Options {
        mode,
        full,
        session,
    })
}

pub fn parse(args: &[String]) -> Result<Options, String> {
    let parsed = match args.split_first() {
        Some((mode, flags)) if mode == "http" => options(true, flags),
        Some((mode, flags)) if mode == "stdio" => options(false, flags),
        _ => options(false, args),
    };
    parsed.ok_or_else(|| USAGE.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(list: &[&str]) -> Result<Options, String> {
        let args: Vec<String> = list.iter().map(|a| (*a).to_owned()).collect();
        parse(&args)
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
    fn refuses_anything_else_with_the_usage() {
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
            assert_eq!(parsed(list), Err(USAGE.to_owned()), "{list:?}");
        }
    }
}
