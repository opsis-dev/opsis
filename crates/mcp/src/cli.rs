pub const USAGE: &str = "Usage: opsis-mcp [stdio]";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Stdio,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub mode: Mode,
}

pub fn parse(args: &[String]) -> Result<Options, String> {
    match args {
        [] => Ok(Options { mode: Mode::Stdio }),
        [mode] if mode == "stdio" => Ok(Options { mode: Mode::Stdio }),
        _ => Err(USAGE.to_owned()),
    }
}
