use std::sync::LazyLock;

use tree_sitter::{Language, Node, Parser, Query, QueryCursor, StreamingIterator};

use crate::text::Units;

const MODULE: &str = "opsis";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntaxKind {
    Keyword,
    String,
    Number,
    Comment,
    Function,
    Module,
    Operator,
    Constant,
    Error,
}

impl SyntaxKind {
    pub const ALL: [Self; 9] = [
        Self::Keyword,
        Self::String,
        Self::Number,
        Self::Comment,
        Self::Function,
        Self::Module,
        Self::Operator,
        Self::Constant,
        Self::Error,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Keyword => "keyword",
            Self::String => "string",
            Self::Number => "number",
            Self::Comment => "comment",
            Self::Function => "function",
            Self::Module => "module",
            Self::Operator => "operator",
            Self::Constant => "constant",
            Self::Error => "error",
        }
    }

    fn of_capture(name: &str) -> Option<Self> {
        let head = name.split('.').next().unwrap_or(name);
        match (head, name) {
            ("keyword" | "conditional" | "repeat" | "label" | "preproc", _) => Some(Self::Keyword),
            ("string", _) => Some(Self::String),
            ("number", _) => Some(Self::Number),
            ("boolean" | "constant", _) => Some(Self::Constant),
            ("comment", _) => Some(Self::Comment),
            ("function" | "method" | "constructor", _) => Some(Self::Function),
            ("operator", _) => Some(Self::Operator),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxRange {
    pub start: u32,
    pub length: u32,
    pub kind: SyntaxKind,
}

struct Grammar {
    language: Language,
    query: Query,
    kinds: Vec<Option<SyntaxKind>>,
}

static GRAMMAR: LazyLock<Grammar> = LazyLock::new(|| {
    let language: Language = tree_sitter_lua::LANGUAGE.into();
    let query = Query::new(&language, tree_sitter_lua::HIGHLIGHTS_QUERY)
        .expect("the Lua grammar's highlight query");
    let kinds = query
        .capture_names()
        .iter()
        .map(|name| SyntaxKind::of_capture(name))
        .collect();
    Grammar {
        language,
        query,
        kinds,
    }
});

fn paint(kinds: &mut [Option<SyntaxKind>], from: usize, to: usize, kind: Option<SyntaxKind>) {
    let to = to.min(kinds.len());
    if from < to {
        kinds[from..to].fill(kind);
    }
}

fn mark_errors(node: Node<'_>, text: &str, kinds: &mut [Option<SyntaxKind>]) {
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
            paint(kinds, start, end, Some(SyntaxKind::Error));
        } else {
            let before = text[..start].trim_end();
            let token = before.rfind(char::is_whitespace).map_or(0, |i| i + 1);
            paint(kinds, token, before.len(), Some(SyntaxKind::Error));
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        mark_errors(child, text, kinds);
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
    let mut kinds = vec![None; text.len()];
    let mut cursor = QueryCursor::new();
    let mut captures = cursor.captures(&grammar.query, tree.root_node(), text.as_bytes());
    while let Some((found, index)) = captures.next() {
        let capture = found.captures()[*index];
        let node = capture.node;
        let kind = grammar.kinds[usize::try_from(capture.index)
            .unwrap_or(usize::MAX)
            .min(grammar.kinds.len() - 1)];
        let kind = if kind.is_none()
            && node.kind() == "identifier"
            && node.utf8_text(text.as_bytes()) == Ok(MODULE)
        {
            Some(SyntaxKind::Module)
        } else {
            kind
        };
        paint(&mut kinds, node.start_byte(), node.end_byte(), kind);
    }
    mark_errors(tree.root_node(), text, &mut kinds);
    ranges(text, &kinds)
}

fn ranges(text: &str, kinds: &[Option<SyntaxKind>]) -> Vec<SyntaxRange> {
    let units = Units::new(text);
    let mut out = Vec::new();
    let mut at = 0;
    while at < kinds.len() {
        let kind = kinds[at];
        let mut end = at + 1;
        while end < kinds.len() && kinds[end] == kind {
            end += 1;
        }
        if let Some(kind) = kind {
            let start = units.unit(at);
            let stop = units.unit(end);
            if stop > start {
                out.push(SyntaxRange {
                    start,
                    length: stop - start,
                    kind,
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

    fn kinds_in(text: &str) -> Vec<(String, SyntaxKind)> {
        let utf16: Vec<u16> = text.encode_utf16().collect();
        highlight(text)
            .into_iter()
            .map(|r| {
                let from = r.start as usize;
                let to = from + r.length as usize;
                (String::from_utf16_lossy(&utf16[from..to]), r.kind)
            })
            .collect()
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
    fn colours_lua_and_the_opsis_module_after_characters_outside_the_basic_plane() {
        let found = kinds_in("local s = \"😀\" -- note\nprint(opsis.version, 42, nil)\n");
        assert!(
            found.contains(&("local".to_owned(), SyntaxKind::Keyword)),
            "{found:?}"
        );
        assert!(
            found.contains(&("\"😀\"".to_owned(), SyntaxKind::String)),
            "{found:?}"
        );
        assert!(
            found.contains(&("-- note".to_owned(), SyntaxKind::Comment)),
            "{found:?}"
        );
        assert!(
            found.contains(&("opsis".to_owned(), SyntaxKind::Module)),
            "{found:?}"
        );
        assert!(
            found.contains(&("42".to_owned(), SyntaxKind::Number)),
            "{found:?}"
        );
        assert!(
            found.contains(&("nil".to_owned(), SyntaxKind::Constant)),
            "{found:?}"
        );
        assert!(
            found.contains(&("print".to_owned(), SyntaxKind::Function)),
            "{found:?}"
        );
    }

    #[test]
    fn marks_a_syntax_error() {
        let found = kinds_in("local x = \nif then\n");
        assert!(
            found.iter().any(|(_, kind)| *kind == SyntaxKind::Error),
            "{found:?}"
        );
    }
}
