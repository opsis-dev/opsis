use crate::text::{Units, units_of};

pub const INDENT: &str = "  ";
const COMMENT: &str = "--";
const OPENERS: [&str; 4] = ["function", "then", "do", "repeat"];
const CLOSERS: [&str; 2] = ["end", "until"];
const BRANCHES: [&str; 2] = ["else", "elseif"];
const PAIRS: [(char, char); 3] = [('(', ')'), ('[', ']'), ('{', '}')];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextChange {
    pub start: u32,
    pub end: u32,
    pub text: String,
    pub anchor: u32,
    pub cursor: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    Open(char, usize),
    Close(char, usize),
}

fn long_bracket(text: &str) -> Option<usize> {
    let rest = text.strip_prefix('[')?;
    let level = rest.chars().take_while(|c| *c == '=').count();
    rest[level..].starts_with('[').then_some(level)
}

fn skip_long(text: &str, at: usize, level: usize) -> usize {
    let close = format!("]{}]", "=".repeat(level));
    text[at..]
        .find(&close)
        .map_or(text.len(), |i| at + i + close.len())
}

fn tokens(text: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(ch) = text[at..].chars().next() {
        let rest = &text[at..];
        if rest.starts_with(COMMENT) {
            let after = at + COMMENT.len();
            at = match long_bracket(&text[after..]) {
                Some(level) => skip_long(text, after + level + 2, level),
                None => text[after..].find('\n').map_or(text.len(), |i| after + i),
            };
            continue;
        }
        if ch == '"' || ch == '\'' {
            let mut end = at + 1;
            let mut escaped = false;
            for c in text[at + 1..].chars() {
                end += c.len_utf8();
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == ch || c == '\n' {
                    break;
                }
            }
            at = end;
            continue;
        }
        if let Some(level) = long_bracket(rest) {
            at = skip_long(text, at + level + 2, level);
            continue;
        }
        if ch.is_alphabetic() || ch == '_' {
            let word: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            at += word.len();
            out.push(Token::Word(word));
            continue;
        }
        if PAIRS.iter().any(|(open, _)| *open == ch) {
            out.push(Token::Open(ch, at));
        } else if PAIRS.iter().any(|(_, close)| *close == ch) {
            out.push(Token::Close(ch, at));
        }
        at += ch.len_utf8();
    }
    out
}

fn is_closer(token: &Token) -> bool {
    match token {
        Token::Word(word) => CLOSERS.contains(&word.as_str()) || BRANCHES.contains(&word.as_str()),
        Token::Close(..) => true,
        Token::Open(..) => false,
    }
}

fn opens(line: &str) -> bool {
    let found = tokens(line);
    let mut net = 0i32;
    let mut leading = true;
    for token in &found {
        if leading && is_closer(token) {
            if let Token::Word(word) = token
                && BRANCHES.contains(&word.as_str())
            {
                net += 1;
            }
            continue;
        }
        leading = false;
        net += match token {
            Token::Word(word) if OPENERS.contains(&word.as_str()) => 1,
            Token::Word(word) if CLOSERS.contains(&word.as_str()) => -1,
            Token::Open(..) => 1,
            Token::Close(..) => -1,
            Token::Word(_) => 0,
        };
    }
    net > 0
}

fn indent_of(line: &str) -> &str {
    &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
}

fn line_span(text: &str, byte: usize) -> (usize, usize) {
    let start = text[..byte].rfind('\n').map_or(0, |i| i + 1);
    let end = text[byte..].find('\n').map_or(text.len(), |i| byte + i);
    (start, end)
}

fn previous_line(text: &str, line_start: usize) -> Option<&str> {
    text[..line_start.saturating_sub(1)]
        .split('\n')
        .rev()
        .find(|line| !line.trim().is_empty())
        .filter(|_| line_start > 0)
}

fn outdented(indent: &str) -> &str {
    indent
        .strip_suffix(INDENT)
        .or_else(|| indent.strip_suffix('\t'))
        .unwrap_or_else(|| indent.trim_end_matches(' '))
}

fn closing_target(text: &str, line_start: usize) -> &str {
    previous_line(text, line_start).map_or("", |previous| {
        let indent = indent_of(previous);
        if opens(previous) {
            indent
        } else {
            outdented(indent)
        }
    })
}

fn starts_with_closer(line: &str) -> bool {
    let trimmed = line.trim();
    let word = trimmed
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .next()
        .unwrap_or_default();
    trimmed == word && (CLOSERS.contains(&word) || BRANCHES.contains(&word))
        || PAIRS.iter().any(|(_, close)| trimmed.starts_with(*close))
}

pub fn enter(text: &str, cursor: u32) -> TextChange {
    let units = Units::new(text);
    let at = units.byte(cursor);
    let (line_start, _) = line_span(text, at);
    let before = &text[line_start..at];
    let indent = indent_of(before).to_owned();
    let deeper = format!("{indent}{INDENT}");
    let previous = text[..at].chars().next_back();
    let next = text[at..].chars().next();
    let between = PAIRS
        .iter()
        .any(|(open, close)| previous == Some(*open) && next == Some(*close));
    let (inserted, cursor_at) = if between {
        let first = format!("\n{deeper}");
        let length = units_of(&first);
        (format!("{first}\n{indent}"), cursor + length)
    } else {
        let line = if opens(before) { deeper } else { indent };
        let inserted = format!("\n{line}");
        let length = units_of(&inserted);
        (inserted, cursor + length)
    };
    TextChange {
        start: cursor,
        end: cursor,
        text: inserted,
        anchor: cursor_at,
        cursor: cursor_at,
    }
}

fn moves_past(text: &str, at: usize, typed: char) -> bool {
    text[at..].starts_with(typed)
        && (PAIRS.iter().any(|(_, close)| *close == typed) || typed == '"' || typed == '\'')
}

fn pairs_here(text: &str, at: usize, typed: char) -> Option<char> {
    let next = text[at..].chars().next();
    let free = next.is_none_or(|c| c.is_whitespace() || ")]},".contains(c));
    if !free {
        return None;
    }
    if let Some((_, close)) = PAIRS.iter().find(|(open, _)| *open == typed) {
        return Some(*close);
    }
    let previous = text[..at].chars().next_back();
    let after_word = previous.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '\\');
    ((typed == '"' || typed == '\'') && !after_word).then_some(typed)
}

pub fn typed(text: &str, cursor: u32, typed: char) -> TextChange {
    let units = Units::new(text);
    let at = units.byte(cursor);
    let step = units_of(typed.encode_utf8(&mut [0; 4]));
    if moves_past(text, at, typed) {
        return TextChange {
            start: cursor,
            end: cursor,
            text: String::new(),
            anchor: cursor + step,
            cursor: cursor + step,
        };
    }
    if let Some(close) = pairs_here(text, at, typed) {
        return TextChange {
            start: cursor,
            end: cursor,
            text: format!("{typed}{close}"),
            anchor: cursor + step,
            cursor: cursor + step,
        };
    }
    let (line_start, line_end) = line_span(text, at);
    let mut line = text[line_start..line_end].to_owned();
    let offset = at - line_start;
    line.insert(offset, typed);
    let indent = indent_of(&line).to_owned();
    let target = closing_target(text, line_start);
    let typed_end = offset + typed.len_utf8();
    if starts_with_closer(&line[..typed_end]) && target.len() < indent.len() {
        let rest = line[indent.len()..].to_owned();
        let shift = units_of(&indent) - units_of(target);
        let start = units.unit(line_start);
        let after = cursor + step - shift;
        return TextChange {
            start,
            end: units.unit(line_end),
            text: format!("{target}{rest}"),
            anchor: after,
            cursor: after,
        };
    }
    TextChange {
        start: cursor,
        end: cursor,
        text: typed.to_string(),
        anchor: cursor + step,
        cursor: cursor + step,
    }
}

pub fn backspace(text: &str, cursor: u32) -> Option<TextChange> {
    let units = Units::new(text);
    let at = units.byte(cursor);
    let previous = text[..at].chars().next_back()?;
    let next = text[at..].chars().next()?;
    let paired = PAIRS
        .iter()
        .any(|(open, close)| *open == previous && *close == next)
        || (previous == next && (previous == '"' || previous == '\''));
    paired.then(|| TextChange {
        start: cursor - 1,
        end: cursor + 1,
        text: String::new(),
        anchor: cursor - 1,
        cursor: cursor - 1,
    })
}

fn lines_of(text: &str, anchor: u32, cursor: u32) -> (usize, usize) {
    let units = Units::new(text);
    let (low, high) = (anchor.min(cursor), anchor.max(cursor));
    let from = units.byte(low);
    let mut to = units.byte(high);
    if high > low && to > 0 && text[..to].ends_with('\n') {
        to -= 1;
    }
    let start = line_span(text, from).0;
    let end = line_span(text, to.max(from)).1;
    (start, end)
}

fn replaced_lines(text: &str, start: usize, end: usize, lines: &[String]) -> TextChange {
    let units = Units::new(text);
    let joined = lines.join("\n");
    let from = units.unit(start);
    TextChange {
        start: from,
        end: units.unit(end),
        anchor: from,
        cursor: from + units_of(&joined),
        text: joined,
    }
}

pub fn indent_lines(text: &str, anchor: u32, cursor: u32, outdent: bool) -> TextChange {
    if anchor == cursor && !outdent {
        return TextChange {
            start: cursor,
            end: cursor,
            text: INDENT.to_owned(),
            anchor: cursor + units_of(INDENT),
            cursor: cursor + units_of(INDENT),
        };
    }
    let (start, end) = lines_of(text, anchor, cursor);
    let lines: Vec<String> = text[start..end]
        .split('\n')
        .map(|line| {
            if line.trim().is_empty() {
                line.to_owned()
            } else if outdent {
                let indent = indent_of(line);
                format!("{}{}", outdented(indent), &line[indent.len()..])
            } else {
                format!("{INDENT}{line}")
            }
        })
        .collect();
    replaced_lines(text, start, end, &lines)
}

pub fn toggle_comment(text: &str, anchor: u32, cursor: u32) -> TextChange {
    let (start, end) = lines_of(text, anchor, cursor);
    let original: Vec<&str> = text[start..end].split('\n').collect();
    let filled: Vec<&&str> = original.iter().filter(|l| !l.trim().is_empty()).collect();
    let commented = !filled.is_empty()
        && filled
            .iter()
            .all(|line| line.trim_start().starts_with(COMMENT));
    let column = filled
        .iter()
        .map(|line| indent_of(line).len())
        .min()
        .unwrap_or(0);
    let lines: Vec<String> = original
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                (*line).to_owned()
            } else if commented {
                let indent = indent_of(line);
                let rest = &line[indent.len() + COMMENT.len()..];
                format!("{indent}{}", rest.strip_prefix(' ').unwrap_or(rest))
            } else {
                format!("{}{COMMENT} {}", &line[..column], &line[column..])
            }
        })
        .collect();
    replaced_lines(text, start, end, &lines)
}

pub fn matching_bracket(text: &str, cursor: u32) -> Option<(u32, u32)> {
    let units = Units::new(text);
    let at = units.byte(cursor);
    let mut open: Vec<(char, usize)> = Vec::new();
    let mut pairs = Vec::new();
    for token in tokens(text) {
        match token {
            Token::Open(ch, start) => open.push((ch, start)),
            Token::Close(ch, end) => {
                if let Some(&(opened, start)) = open.last()
                    && PAIRS.contains(&(opened, ch))
                {
                    open.pop();
                    pairs.push((start, end));
                }
            }
            Token::Word(_) => {}
        }
    }
    let touches = |byte: usize| byte == at || byte + 1 == at;
    pairs
        .into_iter()
        .find(|(start, end)| touches(*start) || touches(*end))
        .map(|(start, end)| (units.unit(start), units.unit(end)))
}

pub fn apply(text: &str, change: &TextChange) -> String {
    let units = Units::new(text);
    let mut out = text[..units.byte(change.start)].to_owned();
    out.push_str(&change.text);
    out.push_str(&text[units.byte(change.end)..]);
    out
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn lua_lines() -> impl Strategy<Value = String> {
        proptest::collection::vec(
            "( {0,6})(local x = 1|print\\(\"é😀\"\\)|if a then|end|else|f\\(|\\}|x = \\{|)",
            1..8,
        )
        .prop_map(|lines| lines.join("\n"))
    }

    proptest! {
        #[test]
        fn indenting_then_outdenting_a_selection_returns_it(text in lua_lines(), a in 0u32..200, b in 0u32..200) {
            let length = units_of(&text);
            let (a, b) = (a.min(length), b.min(length));
            if a != b {
                let indented = indent_lines(&text, a, b, false);
                let once = apply(&text, &indented);
                let back = indent_lines(&once, indented.anchor, indented.cursor, true);
                prop_assert_eq!(apply(&once, &back), text);
            }
        }

        #[test]
        fn commenting_twice_returns_it(text in lua_lines(), a in 0u32..200, b in 0u32..200) {
            let length = units_of(&text);
            let (a, b) = (a.min(length), b.min(length));
            let once = toggle_comment(&text, a, b);
            let commented = apply(&text, &once);
            let twice = toggle_comment(&commented, once.anchor, once.cursor);
            prop_assert_eq!(apply(&commented, &twice), text);
        }

        #[test]
        fn a_typed_closer_never_deepens_the_line_or_leaves_the_text(text in lua_lines(), at in 0u32..200, key in "[edn}\\)\\]\"(a]") {
            let at = at.min(units_of(&text));
            let key = key.chars().next().unwrap();
            let change = typed(&text, at, key);
            prop_assert!(change.start <= change.end && change.end <= units_of(&text));
            let after = apply(&text, &change);
            prop_assert!(change.cursor <= units_of(&after));
            let units = Units::new(&text);
            let (start, end) = line_span(&text, units.byte(at));
            let before_indent = indent_of(&text[start..end]).len();
            let new_units = Units::new(&after);
            let (s, e) = line_span(&after, new_units.byte(change.cursor.min(units_of(&after))));
            prop_assert!(indent_of(&after[s..e]).len() <= before_indent);
        }

        #[test]
        fn a_matched_bracket_pair_is_a_pair(text in lua_lines(), at in 0u32..200) {
            let at = at.min(units_of(&text));
            if let Some((open, close)) = matching_bracket(&text, at) {
                let utf16: Vec<u16> = text.encode_utf16().collect();
                let pair = (char::from_u32(u32::from(utf16[open as usize])), char::from_u32(u32::from(utf16[close as usize])));
                prop_assert!(PAIRS.iter().any(|(o, c)| pair == (Some(*o), Some(*c))));
                prop_assert!(open < close);
            }
        }
    }

    #[test]
    fn keeps_the_indent_and_adds_a_level_after_a_block_opener() {
        let text = "  if a then";
        let change = enter(text, units_of(text));
        assert_eq!(change.text, "\n    ");
        let text = "  print(1)";
        assert_eq!(enter(text, units_of(text)).text, "\n  ");
        let text = "local t = {}";
        let change = enter(text, 11);
        assert_eq!(apply(text, &change), "local t = {\n  \n}");
        assert_eq!(change.cursor, 14);
    }

    #[test]
    fn steps_a_closing_word_back_a_level() {
        let text = "if a then\n  x()\n  en";
        let change = typed(text, units_of(text), 'd');
        assert_eq!(apply(text, &change), "if a then\n  x()\nend");
        let text = "function f()\n  ";
        let change = typed(text, units_of(text), '}');
        assert_eq!(apply(text, &change), "function f()\n}");
        let text = "if a then\n  x()\n  els";
        let once = apply(text, &typed(text, units_of(text), 'e'));
        assert_eq!(once, "if a then\n  x()\nelse");
        let twice = apply(&once, &typed(&once, units_of(&once), 'i'));
        assert_eq!(twice, "if a then\n  x()\nelsei");
    }

    #[test]
    fn closes_brackets_and_quotes_and_moves_past_the_closer() {
        assert_eq!(apply("x = ", &typed("x = ", 4, '(')), "x = ()");
        assert_eq!(typed("x = ()", 5, ')').cursor, 6);
        assert_eq!(apply("x = ", &typed("x = ", 4, '"')), "x = \"\"");
        assert_eq!(apply("don", &typed("don", 3, '\'')), "don'");
        assert_eq!(apply("x = ()", &backspace("x = ()", 5).unwrap()), "x = ");
        assert_eq!(matching_bracket("f(a[1])", 2), Some((1, 6)));
        assert_eq!(matching_bracket("f(\"(\")", 1), Some((1, 5)));
    }
}
