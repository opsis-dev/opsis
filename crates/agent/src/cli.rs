pub const USAGE: &str = "Usage: opsis-agent          speak ACP on standard input and output\n       opsis-agent login    choose the model service, the model and its key";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Serve,
    Login,
}

pub fn parse(args: &[String]) -> Result<Command, String> {
    match args {
        [] => Ok(Command::Serve),
        [word] if word == "login" => Ok(Command::Login),
        _ => Err(USAGE.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(list: &[&str]) -> Result<Command, String> {
        parse(&list.iter().map(|a| (*a).to_owned()).collect::<Vec<_>>())
    }

    #[test]
    fn serves_with_nothing_and_logs_in_when_asked() {
        assert_eq!(parsed(&[]), Ok(Command::Serve));
        assert_eq!(parsed(&["login"]), Ok(Command::Login));
    }

    #[test]
    fn refuses_anything_else_with_the_usage() {
        for list in [&["serve"][..], &["set"], &["login", "openai"], &["--login"]] {
            assert_eq!(parsed(list), Err(USAGE.to_owned()), "{list:?}");
        }
    }
}
