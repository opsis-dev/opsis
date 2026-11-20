use crate::http::DEFAULT_HOST;

pub const USAGE: &str =
    "Usage: opsis-mcp [stdio]\n       opsis-mcp http [--host <host>] [--port <port>]";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Stdio,
    Http { host: String, port: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub mode: Mode,
}

fn http(flags: &[String]) -> Option<Mode> {
    let mut host = DEFAULT_HOST.to_owned();
    let mut port = 0;
    let mut rest = flags.iter();
    while let Some(flag) = rest.next() {
        let value = rest.next()?;
        match flag.as_str() {
            "--host" if !value.is_empty() => value.clone_into(&mut host),
            "--port" => port = value.parse().ok()?,
            _ => return None,
        }
    }
    Some(Mode::Http { host, port })
}

pub fn parse(args: &[String]) -> Result<Options, String> {
    let mode = match args {
        [] => Some(Mode::Stdio),
        [mode] if mode == "stdio" => Some(Mode::Stdio),
        [mode, flags @ ..] if mode == "http" => http(flags),
        _ => None,
    };
    mode.map(|mode| Options { mode })
        .ok_or_else(|| USAGE.to_owned())
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
        assert_eq!(parsed(&[]).unwrap().mode, Mode::Stdio);
        assert_eq!(parsed(&["stdio"]).unwrap().mode, Mode::Stdio);
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
    fn refuses_anything_else_with_the_usage() {
        for list in [
            &["web"][..],
            &["stdio", "--port", "1"],
            &["http", "--port"],
            &["http", "--port", "x"],
            &["http", "--host", ""],
            &["http", "--token", "a"],
        ] {
            assert_eq!(parsed(list), Err(USAGE.to_owned()), "{list:?}");
        }
    }
}
