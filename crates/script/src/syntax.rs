use std::sync::LazyLock;

use tree_sitter::{Language, Node, Parser, Query, QueryCursor, StreamingIterator};

use crate::text::Units;

const HIGHLIGHTS: &str = include_str!("highlights.generated.scm");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxRange {
    pub start: u32,
    pub length: u32,
    pub capture: Option<&'static str>,
    pub error: bool,
}

struct Grammar {
    language: Language,
    query: Query,
    names: Vec<&'static str>,
}

static GRAMMAR: LazyLock<Grammar> = LazyLock::new(|| {
    let language: Language = tree_sitter_lua::LANGUAGE.into();
    let query = Query::new(&language, HIGHLIGHTS).expect("the Lua highlight query");
    let names = query
        .capture_names()
        .iter()
        .map(|name| &*Box::leak(name.to_string().into_boxed_str()))
        .collect();
    Grammar {
        language,
        query,
        names,
    }
});

pub fn captures() -> &'static [&'static str] {
    &GRAMMAR.names
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Mark {
    capture: Option<u32>,
    error: bool,
}

fn paint_errors(marks: &mut [Mark], from: usize, to: usize) {
    let to = to.min(marks.len());
    if from < to {
        for mark in &mut marks[from..to] {
            mark.error = true;
        }
    }
}

fn mark_errors(node: Node<'_>, text: &str, marks: &mut [Mark]) {
    if !node.has_error() {
        return;
    }
    if node.is_error() || node.is_missing() {
        let start = node.start_byte();
        let line_end = text[start..]
            .find('\n')
            .map_or(text.len(), |end| start + end);
        let end = node.end_byte().min(line_end);
        if end > start {
            paint_errors(marks, start, end);
        } else {
            let before = text[..start].trim_end();
            let token = before.rfind(char::is_whitespace).map_or(0, |i| i + 1);
            paint_errors(marks, token, before.len());
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        mark_errors(child, text, marks);
    }
}

pub fn highlight(text: &str) -> Vec<SyntaxRange> {
    let grammar = &*GRAMMAR;
    let mut parser = Parser::new();
    if parser.set_language(&grammar.language).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(text, None) else {
        return Vec::new();
    };
    let mut marks = vec![Mark::default(); text.len()];
    let mut cursor = QueryCursor::new();
    let mut found = cursor.captures(&grammar.query, tree.root_node(), text.as_bytes());
    while let Some((matched, index)) = found.next() {
        let capture = matched.captures()[*index];
        let from = capture.node.start_byte();
        let to = capture.node.end_byte().min(marks.len());
        if from < to {
            for mark in &mut marks[from..to] {
                mark.capture = Some(capture.index);
            }
        }
    }
    mark_errors(tree.root_node(), text, &mut marks);
    ranges(text, &marks)
}

fn ranges(text: &str, marks: &[Mark]) -> Vec<SyntaxRange> {
    let names = &GRAMMAR.names;
    let units = Units::new(text);
    let mut out = Vec::new();
    let mut at = 0;
    while at < marks.len() {
        let mark = marks[at];
        let mut end = at + 1;
        while end < marks.len() && marks[end] == mark {
            end += 1;
        }
        let capture = mark
            .capture
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| names.get(index).copied());
        if capture.is_some() || mark.error {
            let start = units.unit(at);
            let stop = units.unit(end);
            if stop > start {
                out.push(SyntaxRange {
                    start,
                    length: stop - start,
                    capture,
                    error: mark.error,
                });
            }
        }
        at = end;
    }
    out
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::text::units_of;

    fn captures_in(text: &str) -> Vec<(String, Option<&'static str>, bool)> {
        let utf16: Vec<u16> = text.encode_utf16().collect();
        highlight(text)
            .into_iter()
            .map(|r| {
                let from = r.start as usize;
                let to = from + r.length as usize;
                (
                    String::from_utf16_lossy(&utf16[from..to]),
                    r.capture,
                    r.error,
                )
            })
            .collect()
    }

    fn captured(
        found: &[(String, Option<&'static str>, bool)],
        text: &str,
    ) -> Option<&'static str> {
        found
            .iter()
            .find(|(t, _, _)| t == text)
            .and_then(|(_, c, _)| *c)
    }

    proptest! {
        #[test]
        fn ranges_lie_inside_the_text_and_come_in_order(text in "(local |function |end|\"é😀\"|--x\n|1\\.5|opsis\\.nodes|[ \n(){}=]|\\PC){0,40}") {
            let length = units_of(&text);
            let mut last = 0;
            for range in highlight(&text) {
                prop_assert!(range.start >= last);
                prop_assert!(range.length > 0);
                prop_assert!(range.start + range.length <= length);
                last = range.start + range.length;
            }
        }
    }

    #[test]
    fn names_lua_with_the_themes_scopes_after_characters_outside_the_basic_plane() {
        let found = captures_in("local s = \"😀\" -- note\nprint(opsis.version, 42, nil)\n");
        assert_eq!(captured(&found, "local"), Some("keyword"), "{found:?}");
        assert_eq!(captured(&found, "\"😀\""), Some("string"), "{found:?}");
        assert_eq!(captured(&found, "-- note"), Some("comment"), "{found:?}");
        assert_eq!(
            captured(&found, "42"),
            Some("constant.numeric.integer"),
            "{found:?}"
        );
        assert_eq!(
            captured(&found, "nil"),
            Some("constant.builtin"),
            "{found:?}"
        );
        assert_eq!(
            captured(&found, "print"),
            Some("function.call"),
            "{found:?}"
        );
    }

    #[test]
    fn marks_a_syntax_error() {
        let found = captures_in("local x = \nif then\n");
        assert!(found.iter().any(|(_, _, error)| *error), "{found:?}");
    }
}
