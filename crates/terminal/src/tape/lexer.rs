#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    At,
    Equal,
    Plus,
    Percent,
    Minus,
    LeftBracket,
    RightBracket,
    Caret,
    Backslash,
    Comment,
    Json,
    Str,
    Regex,
    Number,
    Word,
    Illegal,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: Kind,
    pub text: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub struct Lexer {
    chars: Vec<char>,
    at: usize,
    line: usize,
    column: usize,
}

const fn is_letter(c: char) -> bool {
    c.is_ascii_alphabetic()
}

const fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '/' | '%')
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            at: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.at + 1).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.at += 1;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    fn take_while(&mut self, keep: impl Fn(char) -> bool) -> String {
        let mut out = String::new();
        while let Some(c) = self.peek().filter(|c| keep(*c)) {
            out.push(c);
            self.bump();
        }
        out
    }

    fn quoted(&mut self, end: char) -> String {
        self.bump();
        let text = self.take_while(|c| c != end && c != '\n' && c != '\r');
        if self.peek() == Some(end) {
            self.bump();
        }
        text
    }

    fn regex(&mut self) -> String {
        self.bump();
        let mut text = String::new();
        let mut escaped = false;
        while let Some(c) = self.peek() {
            if c == '\n' || c == '\r' {
                break;
            }
            if c == '/' && !escaped {
                self.bump();
                break;
            }
            escaped = c == '\\' && !escaped;
            text.push(c);
            self.bump();
        }
        text
    }

    pub fn next_token(&mut self) -> Token {
        self.take_while(char::is_whitespace);
        let (line, column) = (self.line, self.column);
        let token = |kind: Kind, text: String| Token {
            kind,
            text,
            line,
            column,
        };
        let Some(c) = self.peek() else {
            return token(Kind::Eof, String::new());
        };
        let single = match c {
            '@' => Some(Kind::At),
            '=' => Some(Kind::Equal),
            '+' => Some(Kind::Plus),
            '%' => Some(Kind::Percent),
            '-' => Some(Kind::Minus),
            '[' => Some(Kind::LeftBracket),
            ']' => Some(Kind::RightBracket),
            '^' => Some(Kind::Caret),
            '\\' => Some(Kind::Backslash),
            _ => None,
        };
        if let Some(kind) = single {
            self.bump();
            return token(kind, c.to_string());
        }
        match c {
            '#' => {
                self.bump();
                token(Kind::Comment, self.take_while(|c| c != '\n' && c != '\r'))
            }
            '{' => {
                self.bump();
                let inner = self.take_while(|c| c != '}');
                self.bump();
                token(Kind::Json, format!("{{{inner}}}"))
            }
            '"' | '\'' | '`' => token(Kind::Str, self.quoted(c)),
            '/' => token(Kind::Regex, self.regex()),
            c if c.is_ascii_digit()
                || (c == '.' && self.peek_next().is_some_and(|n| n.is_ascii_digit())) =>
            {
                token(
                    Kind::Number,
                    self.take_while(|c| c.is_ascii_digit() || c == '.'),
                )
            }
            c if is_letter(c) || c == '.' => token(Kind::Word, self.take_while(is_word)),
            _ => {
                self.bump();
                token(Kind::Illegal, c.to_string())
            }
        }
    }
}
