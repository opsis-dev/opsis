use opsis_scene::node::{Node, span};
use opsis_scene::style::{Style, WhiteSpace};

use crate::elements::draw::css_color;
use crate::elements::units::{utf16_len, utf16_slice};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodeTokenKind {
    Plain,
    Comment,
    String,
    Keyword,
    Number,
    Type,
    Call,
    Prop,
    Punct,
    Tag,
}

impl CodeTokenKind {
    pub const ALL: [Self; 10] = [
        Self::Plain,
        Self::Comment,
        Self::String,
        Self::Keyword,
        Self::Number,
        Self::Type,
        Self::Call,
        Self::Prop,
        Self::Punct,
        Self::Tag,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Comment => "comment",
            Self::String => "string",
            Self::Keyword => "keyword",
            Self::Number => "number",
            Self::Type => "type",
            Self::Call => "call",
            Self::Prop => "prop",
            Self::Punct => "punct",
            Self::Tag => "tag",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeToken {
    pub text: String,
    pub kind: CodeTokenKind,
}

const KEYWORDS: [&str; 53] = [
    "import",
    "from",
    "export",
    "default",
    "const",
    "let",
    "var",
    "function",
    "return",
    "async",
    "await",
    "type",
    "interface",
    "class",
    "extends",
    "implements",
    "new",
    "if",
    "else",
    "for",
    "while",
    "switch",
    "case",
    "break",
    "continue",
    "try",
    "catch",
    "finally",
    "throw",
    "typeof",
    "instanceof",
    "in",
    "of",
    "as",
    "null",
    "undefined",
    "true",
    "false",
    "this",
    "void",
    "yield",
    "public",
    "private",
    "readonly",
    "static",
    "def",
    "fn",
    "pub",
    "use",
    "struct",
    "enum",
    "match",
    "impl",
];

const fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

const fn is_name(c: char) -> bool {
    is_word(c) || c == '$'
}

const fn is_space(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

fn byte_len(chars: &[char]) -> usize {
    chars.iter().map(|c| c.len_utf8()).sum()
}

fn comment(rest: &[char]) -> Option<usize> {
    let line = rest.starts_with(&['/', '/']) || rest.first() == Some(&'#');
    line.then_some(rest.len())
}

fn block_comment(rest: &[char]) -> Option<usize> {
    if !rest.starts_with(&['/', '*']) {
        return None;
    }
    let close = (2..rest.len().saturating_sub(1)).find(|&i| rest[i] == '*' && rest[i + 1] == '/');
    Some(close.map_or(rest.len(), |i| i + 2))
}

fn quoted(rest: &[char]) -> Option<usize> {
    let quote = *rest.first().filter(|c| matches!(c, '"' | '\'' | '`'))?;
    let mut at = 1;
    while at < rest.len() {
        match rest[at] {
            '\\' if at + 1 < rest.len() => at += 2,
            '\\' => return None,
            c if c == quote => return Some(at + 1),
            _ => at += 1,
        }
    }
    Some(at)
}

fn digits_from(rest: &[char], at: usize) -> usize {
    rest[at..].iter().take_while(|c| c.is_ascii_digit()).count()
}

fn bounded(rest: &[char], end: usize) -> bool {
    rest.get(end).is_none_or(|c| !is_word(*c))
}

fn number(rest: &[char]) -> Option<usize> {
    if rest.starts_with(&['0', 'x']) {
        let hex = rest[2..]
            .iter()
            .take_while(|c| c.is_ascii_hexdigit())
            .count();
        if hex > 0 && bounded(rest, 2 + hex) {
            return Some(2 + hex);
        }
    }
    let whole = digits_from(rest, 0);
    if whole == 0 {
        return None;
    }
    let fraction = (rest.get(whole) == Some(&'.'))
        .then(|| digits_from(rest, whole + 1))
        .filter(|n| *n > 0)
        .map(|n| whole + 1 + n);
    let exponent = |from: usize| {
        if rest.get(from) != Some(&'e') {
            return None;
        }
        let sign = usize::from(matches!(rest.get(from + 1), Some('+' | '-')));
        let n = digits_from(rest, from + 1 + sign);
        (n > 0).then_some(from + 1 + sign + n)
    };
    let candidates = [
        fraction.and_then(exponent),
        fraction,
        exponent(whole),
        Some(whole),
    ];
    candidates
        .into_iter()
        .flatten()
        .find(|&end| bounded(rest, end))
}

fn name(rest: &[char]) -> Option<usize> {
    let first = rest.first()?;
    if !(first.is_ascii_alphabetic() || *first == '_' || *first == '$') {
        return None;
    }
    Some(1 + rest[1..].iter().take_while(|c| is_name(**c)).count())
}

fn punct(rest: &[char]) -> Option<usize> {
    rest.first()
        .filter(|c| !is_space(**c) && !is_name(**c))
        .map(|_| 1)
}

fn spaces(rest: &[char]) -> Option<usize> {
    let n = rest.iter().take_while(|c| is_space(**c)).count();
    (n > 0).then_some(n)
}

type Scanner = fn(&[char]) -> Option<usize>;

const SCANNERS: [(CodeTokenKind, Scanner); 7] = [
    (CodeTokenKind::Comment, comment),
    (CodeTokenKind::Comment, block_comment),
    (CodeTokenKind::String, quoted),
    (CodeTokenKind::Number, number),
    (CodeTokenKind::Plain, name),
    (CodeTokenKind::Punct, punct),
    (CodeTokenKind::Plain, spaces),
];

fn classify_word(word: &str, before: &str, after: &str) -> CodeTokenKind {
    let capital = word.starts_with(|c: char| c.is_ascii_uppercase());
    if KEYWORDS.contains(&word) {
        return CodeTokenKind::Keyword;
    }
    if before.trim_end().ends_with(['<', '/']) && capital {
        return CodeTokenKind::Tag;
    }
    if after.starts_with('(') {
        return CodeTokenKind::Call;
    }
    if after.starts_with('=') && !after.starts_with("==") {
        return CodeTokenKind::Prop;
    }
    if capital {
        return CodeTokenKind::Type;
    }
    CodeTokenKind::Plain
}

fn tokenize_line(line: &str, in_block: bool) -> (Vec<CodeToken>, bool) {
    let chars: Vec<char> = line.chars().collect();
    let mut tokens = Vec::new();
    let mut at = 0;
    let mut block = in_block;
    if block {
        let Some(end) = line.find("*/") else {
            return (
                vec![CodeToken {
                    text: line.to_owned(),
                    kind: CodeTokenKind::Comment,
                }],
                true,
            );
        };
        tokens.push(CodeToken {
            text: line[..end + 2].to_owned(),
            kind: CodeTokenKind::Comment,
        });
        at = line[..end + 2].chars().count();
        block = false;
    }
    while at < chars.len() {
        let rest = &chars[at..];
        let (kind, length) = SCANNERS
            .iter()
            .find_map(|(kind, scan)| scan(rest).filter(|n| *n > 0).map(|n| (*kind, n)))
            .unwrap_or((CodeTokenKind::Plain, 1));
        let text: String = rest[..length].iter().collect();
        if kind == CodeTokenKind::Comment && text.starts_with("/*") && !text.ends_with("*/") {
            block = true;
        }
        let kind = if kind == CodeTokenKind::Plain && name(rest).is_some() {
            let start = byte_len(&chars[..at]);
            let end = start + text.len();
            classify_word(&text, &line[..start], &line[end..])
        } else {
            kind
        };
        tokens.push(CodeToken { text, kind });
        at += length;
    }
    (tokens, block)
}

pub fn tokenize_code(lines: &[&str]) -> Vec<Vec<CodeToken>> {
    let mut block = false;
    lines
        .iter()
        .map(|line| {
            let (tokens, still) = tokenize_line(line, block);
            block = still;
            tokens
        })
        .collect()
}

pub fn line_length(tokens: &[CodeToken]) -> usize {
    tokens.iter().map(|t| utf16_len(&t.text)).sum()
}

pub fn code_line(
    tokens: &[CodeToken],
    color: &dyn Fn(CodeTokenKind) -> String,
    reveal: Option<usize>,
) -> Node {
    let mut start = 0;
    let spans: Vec<Node> = tokens
        .iter()
        .map(|token| {
            let length = utf16_len(&token.text);
            let shown = reveal.map_or(length, |r| r.saturating_sub(start).min(length));
            start += length;
            let style = Style::default().color(css_color(&color(token.kind)));
            if shown == length {
                return span(style).child(token.text.clone()).into();
            }
            let mut out = span(style);
            if shown > 0 {
                out = out.child(utf16_slice(&token.text, 0, shown));
            }
            out.child(span(Style::default().opacity(0.0)).child(utf16_slice(
                &token.text,
                shown,
                length,
            )))
            .into()
        })
        .collect();
    span(Style::default().white_space(WhiteSpace::Pre))
        .children(spans)
        .into()
}
