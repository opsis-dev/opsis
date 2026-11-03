use opsis_types::terminal::LookIn;

use super::lexer::{Kind, Lexer, Token};
use super::model::{Command, Key, Tape, TapeError};

const SETTINGS: [&str; 23] = [
    "Shell",
    "FontFamily",
    "FontSize",
    "LetterSpacing",
    "LineHeight",
    "Framerate",
    "TypingSpeed",
    "Theme",
    "PlaybackSpeed",
    "Height",
    "Width",
    "Rows",
    "Columns",
    "Padding",
    "LoopOffset",
    "MarginFill",
    "Margin",
    "WindowBar",
    "WindowBarSize",
    "BorderRadius",
    "CursorBlink",
    "WaitTimeout",
    "WaitPattern",
];

const COMMANDS: [&str; 16] = [
    "Set",
    "Sleep",
    "Type",
    "Ctrl",
    "Alt",
    "Shift",
    "Hide",
    "Require",
    "Show",
    "Output",
    "Wait",
    "Source",
    "Screenshot",
    "Copy",
    "Paste",
    "Env",
];

const OTHER_WORDS: [&str; 7] = ["em", "px", "ms", "s", "m", "true", "false"];

fn is_keyword(word: &str) -> bool {
    COMMANDS.contains(&word)
        || SETTINGS.contains(&word)
        || OTHER_WORDS.contains(&word)
        || Key::from_word(word).is_some()
}

fn is_text(token: &Token) -> bool {
    token.kind == Kind::Str || (token.kind == Kind::Word && !is_keyword(&token.text))
}

const TIME_UNITS: [(&str, f64); 3] = [("ms", 1.0), ("s", 1000.0), ("m", 60_000.0)];

struct Parser {
    lexer: Lexer,
    cur: Token,
    peek: Token,
    errors: Vec<TapeError>,
}

fn whole_ms(value: f64) -> u32 {
    opsis_types::numbers::whole(value)
}

impl Parser {
    fn new(text: &str) -> Self {
        let mut lexer = Lexer::new(text);
        let cur = lexer.next_token();
        let peek = lexer.next_token();
        Self {
            lexer,
            cur,
            peek,
            errors: Vec::new(),
        }
    }

    fn advance(&mut self) {
        let next = self.lexer.next_token();
        self.cur = std::mem::replace(&mut self.peek, next);
    }

    fn fail(&mut self, at: &Token, message: impl Into<String>) {
        self.errors.push(TapeError {
            line: at.line,
            column: at.column,
            message: message.into(),
        });
    }

    fn time(&mut self) -> Option<u32> {
        if self.peek.kind != Kind::Number {
            let at = self.cur.clone();
            self.fail(&at, format!("Expected a time after {}", at.text));
            return None;
        }
        self.advance();
        let number: f64 = self.cur.text.parse().unwrap_or(0.0);
        let mut scale = 1000.0;
        if self.peek.kind == Kind::Word
            && let Some((_, unit)) = TIME_UNITS.iter().find(|(word, _)| *word == self.peek.text)
        {
            scale = *unit;
            self.advance();
        }
        Some(whole_ms(number * scale))
    }

    fn speed(&mut self) -> Option<u32> {
        if self.peek.kind == Kind::At {
            self.advance();
            return self.time();
        }
        None
    }

    fn repeat(&mut self) -> u32 {
        if self.peek.kind == Kind::Number {
            self.advance();
            return self.cur.text.parse().unwrap_or(1);
        }
        1
    }

    fn strings(&mut self) -> String {
        let mut text = String::new();
        if !is_text(&self.peek) {
            let at = self.peek.clone();
            self.fail(&at, format!("{} expects a string", self.cur.text));
        }
        while is_text(&self.peek) {
            self.advance();
            text.push_str(&self.cur.text);
            if is_text(&self.peek) {
                text.push(' ');
            }
        }
        text
    }

    fn one_string(&mut self) -> String {
        if self.peek.kind != Kind::Str && self.peek.kind != Kind::Word {
            let at = self.peek.clone();
            self.fail(&at, format!("{} expects a string", self.cur.text));
            return String::new();
        }
        self.advance();
        self.cur.text.clone()
    }

    fn chord_part(&self) -> Option<String> {
        let peek = &self.peek;
        let usable = match peek.kind {
            Kind::Minus
            | Kind::At
            | Kind::LeftBracket
            | Kind::RightBracket
            | Kind::Caret
            | Kind::Backslash
            | Kind::Word
            | Kind::Number => true,
            Kind::Str => peek.text.chars().count() == 1,
            _ => false,
        };
        usable.then(|| peek.text.clone())
    }

    fn ctrl(&mut self) -> Command {
        let mut keys = Vec::new();
        while self.peek.kind == Kind::Plus {
            self.advance();
            let Some(part) = self.chord_part() else {
                let at = self.peek.clone();
                self.fail(&at, format!("{} can't follow Ctrl+", at.text));
                break;
            };
            keys.push(part);
            self.advance();
        }
        if keys.is_empty() {
            let at = self.cur.clone();
            self.fail(&at, "Ctrl needs a key, such as Ctrl+C");
        }
        Command::Ctrl { keys }
    }

    fn modified(&mut self, name: &str) -> String {
        if self.peek.kind == Kind::Plus {
            self.advance();
            if let Some(part) = self.chord_part() {
                self.advance();
                return part;
            }
        }
        let at = self.cur.clone();
        self.fail(&at, format!("{name} needs a key, such as {name}+Enter"));
        String::new()
    }

    fn wait(&mut self) -> Command {
        let mut scope = LookIn::Line;
        if self.peek.kind == Kind::Plus {
            self.advance();
            match self.peek.text.as_str() {
                "Line" => scope = LookIn::Line,
                "Screen" => scope = LookIn::Screen,
                _ => {
                    let at = self.peek.clone();
                    self.fail(&at, "Wait+ takes Line or Screen");
                }
            }
            self.advance();
        }
        let timeout_ms = self.speed();
        let pattern = if self.peek.kind == Kind::Regex {
            self.advance();
            if regex::Regex::new(&self.cur.text).is_err() {
                let at = self.cur.clone();
                self.fail(
                    &at,
                    format!("“{}” isn't a pattern Opsis can search for", at.text),
                );
            }
            Some(self.cur.text.clone())
        } else {
            None
        };
        Command::Wait {
            scope,
            timeout_ms,
            pattern,
        }
    }

    fn set(&mut self) -> Command {
        if !SETTINGS.contains(&self.peek.text.as_str()) {
            let at = self.peek.clone();
            self.fail(&at, format!("There's no setting “{}”", at.text));
        }
        self.advance();
        let name = self.cur.text.clone();
        let value = match name.as_str() {
            "TypingSpeed" | "WaitTimeout" => {
                self.time().map_or_else(String::new, |ms| format!("{ms}ms"))
            }
            "LoopOffset" => {
                self.advance();
                let value = self.cur.text.clone();
                if self.peek.kind == Kind::Percent {
                    self.advance();
                }
                format!("{value}%")
            }
            _ => {
                self.advance();
                self.cur.text.clone()
            }
        };
        Command::Set { name, value }
    }

    fn command(&mut self) -> Option<Command> {
        let word = self.cur.text.clone();
        if self.cur.kind != Kind::Word {
            let at = self.cur.clone();
            self.fail(&at, format!("“{}” isn't a command", at.text));
            return None;
        }
        if let Some(key) = Key::from_word(&word) {
            let speed_ms = self.speed();
            let repeat = self.repeat();
            return Some(Command::Key {
                key,
                speed_ms,
                repeat,
            });
        }
        Some(match word.as_str() {
            "Type" => {
                let speed_ms = self.speed();
                Command::Type {
                    speed_ms,
                    text: self.strings(),
                }
            }
            "Sleep" => Command::Sleep {
                ms: self.time().unwrap_or(0),
            },
            "Ctrl" => self.ctrl(),
            "Alt" => Command::Alt {
                key: self.modified("Alt"),
            },
            "Shift" => Command::Shift {
                key: self.modified("Shift"),
            },
            "Hide" => Command::Hide,
            "Show" => Command::Show,
            "Wait" => self.wait(),
            "Require" => Command::Require {
                program: self.one_string(),
            },
            "Screenshot" => Command::Screenshot {
                path: self.one_string(),
            },
            "Copy" => Command::Copy {
                text: self.strings(),
            },
            "Paste" => Command::Paste,
            "Env" => {
                let name = self.one_string();
                Command::Env {
                    name,
                    value: self.one_string(),
                }
            }
            "Source" => Command::Source {
                path: self.one_string(),
            },
            "Output" => Command::Output {
                path: self.one_string(),
            },
            "Set" => self.set(),
            _ => {
                let at = self.cur.clone();
                self.fail(&at, format!("“{word}” isn't a command"));
                return None;
            }
        })
    }
}

pub fn parse(text: &str) -> Result<Tape, Vec<TapeError>> {
    let mut parser = Parser::new(text);
    let mut commands = Vec::new();
    while parser.cur.kind != Kind::Eof {
        if parser.cur.kind != Kind::Comment
            && let Some(command) = parser.command()
        {
            commands.push(command);
        }
        parser.advance();
    }
    if parser.errors.is_empty() {
        Ok(Tape { commands })
    } else {
        Err(parser.errors)
    }
}
