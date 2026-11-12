use std::collections::HashMap;

use opsis_scene::easing::Easing;
use opsis_types::numbers::{exact_eq, float};

const TAB: &str = "  ";
const LINE_COST: f64 = 10.0;
const COLUMN_COST: f64 = 0.5;
const MAX_COST: f64 = 1000.0;
const PUNCT_VALUE: f64 = 12.0;
const BASE: f64 = 36.0;
const SHRINK: f64 = 0.08;
const GROW_FROM: f64 = 0.96;
const EXIT_SPREAD: f64 = 4.0;
const EXIT_DURATION: f64 = 12.0;
const MOVE_START: f64 = 6.0;
const MOVE_DURATION: f64 = 22.0;
const CASCADE: f64 = 4.0;
const ENTER_START: f64 = 20.0;
const ENTER_SPREAD: f64 = 4.0;
const ENTER_DURATION: f64 = 12.0;
const ARC: f64 = 0.08;
const RISE: f64 = 8.0;
const FALL: f64 = 14.0;
const HOLD: f64 = 24.0;
const MIN_MORPH: f64 = 12.0;
const MAX_MORPH: f64 = 120.0;
const GLIDE: Easing = Easing::Bezier(0.6, 0.0, 0.2, 1.0);

const KEYWORDS: [&str; 56] = [
    "abstract",
    "as",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "declare",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "from",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "keyof",
    "let",
    "namespace",
    "new",
    "null",
    "of",
    "private",
    "protected",
    "public",
    "readonly",
    "return",
    "satisfies",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "type",
    "typeof",
    "undefined",
    "var",
    "void",
    "while",
    "yield",
];

const OPERATORS: [&str; 26] = [
    "===", "!==", "**=", "...", "&&=", "||=", "??=", "=>", "==", "!=", "<=", ">=", "&&", "||",
    "??", "?.", "++", "--", "+=", "-=", "*=", "/=", "%=", "**", "</", "/>",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MorphKind {
    Keyword,
    String,
    Number,
    Comment,
    Function,
    Type,
    Text,
    Punct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorphRole {
    Rest,
    Stay,
    Move,
    Exit,
    Enter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MorphToken {
    pub text: String,
    pub kind: MorphKind,
    pub line: usize,
    pub col: usize,
    pub word: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub tokens: Vec<MorphToken>,
    pub lines: usize,
    pub columns: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Open {
    Comment,
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lexeme {
    Comment,
    String,
    Number,
    Word,
    Punct,
}

const fn is_space(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

fn is_word_start(c: char) -> bool {
    c.is_ascii_alphabetic()
        || c == '_'
        || c == '$'
        || ('\u{c0}'..='\u{24f}').contains(&c)
        || ('\u{370}'..='\u{3ff}').contains(&c)
}

fn is_word_part(c: char) -> bool {
    is_word_start(c) || c.is_ascii_digit()
}

pub fn normalize(code: &str) -> String {
    let unified = code
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\t', TAB);
    let lines: Vec<&str> = unified.split('\n').map(str::trim_end).collect();
    let start = lines
        .iter()
        .position(|l| !l.is_empty())
        .unwrap_or(lines.len());
    let end = lines
        .iter()
        .rposition(|l| !l.is_empty())
        .map_or(start, |e| e + 1);
    let body = &lines[start..end.max(start)];
    let cut = body
        .iter()
        .filter(|l| !l.is_empty())
        .map(|l| l.chars().count() - l.trim_start().chars().count())
        .min()
        .unwrap_or(0);
    body.iter()
        .map(|l| l.chars().skip(cut).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

const fn closing(line: &[char], from: usize, quote: char) -> Option<usize> {
    let mut at = from;
    while at < line.len() {
        if line[at] == quote {
            return Some(at);
        }
        at += if line[at] == '\\' { 2 } else { 1 };
    }
    None
}

fn find(line: &[char], from: usize, needle: &[char]) -> Option<usize> {
    (from..line.len()).find(|&i| line[i..].starts_with(needle))
}

fn number_at(line: &[char], at: usize) -> usize {
    let rest = &line[at..];
    let digits = |from: usize, extra: &dyn Fn(char) -> bool| {
        rest[from..].iter().take_while(|c| extra(**c)).count()
    };
    if rest.len() > 2 && rest[0] == '0' && matches!(rest[1], 'x' | 'X' | 'b' | 'B' | 'o' | 'O') {
        let n = digits(2, &|c| c.is_ascii_hexdigit() || c == '_');
        if n > 0 {
            let end = 2 + n;
            return end + usize::from(rest.get(end) == Some(&'n'));
        }
    }
    if !rest.first().is_some_and(char::is_ascii_digit) {
        return 0;
    }
    let mut end = 1 + digits(1, &|c| c.is_ascii_digit() || c == '_');
    if rest.get(end) == Some(&'.') && rest.get(end + 1).is_some_and(char::is_ascii_digit) {
        end += 2 + digits(end + 2, &|c| c.is_ascii_digit() || c == '_');
    }
    if matches!(rest.get(end), Some('e' | 'E')) {
        let sign = usize::from(matches!(rest.get(end + 1), Some('+' | '-')));
        let n = digits(end + 1 + sign, &|c| c.is_ascii_digit());
        if n > 0 {
            end += 1 + sign + n;
        }
    }
    end + usize::from(rest.get(end) == Some(&'n'))
}

fn word_at(line: &[char], at: usize) -> usize {
    if !is_word_start(line[at]) {
        return 0;
    }
    1 + line[at + 1..]
        .iter()
        .take_while(|c| is_word_part(**c))
        .count()
}

fn scan_line(line: &[char], open: Option<Open>) -> (Vec<(usize, usize, Lexeme)>, Option<Open>) {
    let mut pieces = Vec::new();
    let mut at = 0;
    if let Some(kind) = open {
        let comment = kind == Open::Comment;
        let end = if comment {
            find(line, 0, &['*', '/'])
        } else {
            closing(line, 0, '`')
        };
        let lexeme = if comment {
            Lexeme::Comment
        } else {
            Lexeme::String
        };
        let Some(end) = end else {
            pieces.push((0, line.len(), lexeme));
            return (pieces, open);
        };
        at = end + if comment { 2 } else { 1 };
        pieces.push((0, at, lexeme));
    }
    while at < line.len() {
        let c = line[at];
        if is_space(c) {
            at += 1;
            continue;
        }
        if line[at..].starts_with(&['/', '/']) {
            pieces.push((at, line.len(), Lexeme::Comment));
            break;
        }
        if line[at..].starts_with(&['/', '*']) {
            let Some(end) = find(line, at + 2, &['*', '/']) else {
                pieces.push((at, line.len(), Lexeme::Comment));
                return (pieces, Some(Open::Comment));
            };
            pieces.push((at, end + 2, Lexeme::Comment));
            at = end + 2;
            continue;
        }
        if matches!(c, '"' | '\'' | '`') {
            let end = closing(line, at + 1, c);
            if end.is_none() && c == '`' {
                pieces.push((at, line.len(), Lexeme::String));
                return (pieces, Some(Open::String));
            }
            let stop = end.map_or(line.len(), |e| e + 1);
            pieces.push((at, stop, Lexeme::String));
            at = stop;
            continue;
        }
        let number = number_at(line, at);
        if number > 0 {
            pieces.push((at, at + number, Lexeme::Number));
            at += number;
            continue;
        }
        let word = word_at(line, at);
        if word > 0 {
            pieces.push((at, at + word, Lexeme::Word));
            at += word;
            continue;
        }
        let length = OPERATORS
            .iter()
            .find(|op| {
                let op: Vec<char> = op.chars().collect();
                line[at..].starts_with(&op)
            })
            .map_or(1, |op| op.chars().count());
        pieces.push((at, (at + length).min(line.len()), Lexeme::Punct));
        at += length;
    }
    (pieces, None)
}

struct Lexed {
    text: String,
    lexeme: Lexeme,
    line: usize,
    col: usize,
}

fn kind_of(lexed: &[Lexed], index: usize) -> MorphKind {
    let item = &lexed[index];
    match item.lexeme {
        Lexeme::Comment => return MorphKind::Comment,
        Lexeme::String => return MorphKind::String,
        Lexeme::Number => return MorphKind::Number,
        Lexeme::Punct => return MorphKind::Punct,
        Lexeme::Word => {}
    }
    let previous = index.checked_sub(1).map_or("", |p| lexed[p].text.as_str());
    let next = lexed.get(index + 1).map_or("", |n| n.text.as_str());
    let member = previous == "." || previous == "?.";
    if !member && KEYWORDS.contains(&item.text.as_str()) {
        return MorphKind::Keyword;
    }
    if next == "(" {
        return MorphKind::Function;
    }
    if item.text.starts_with(|c: char| c.is_ascii_uppercase()) {
        return MorphKind::Type;
    }
    MorphKind::Text
}

pub fn version_of(source: &str) -> Version {
    let code = normalize(source);
    let lines: Vec<&str> = code.split('\n').collect();
    let mut lexed = Vec::new();
    let mut open = None;
    let mut columns = 0;
    for (line, text) in lines.iter().enumerate() {
        let chars: Vec<char> = text.chars().collect();
        columns = columns.max(chars.len());
        let (pieces, still) = scan_line(&chars, open);
        open = still;
        for (mut start, mut end, lexeme) in pieces {
            while start < end && is_space(chars[start]) {
                start += 1;
            }
            while end > start && is_space(chars[end - 1]) {
                end -= 1;
            }
            if end == start {
                continue;
            }
            lexed.push(Lexed {
                text: chars[start..end].iter().collect(),
                lexeme,
                line,
                col: start,
            });
        }
    }
    let tokens = (0..lexed.len())
        .map(|index| {
            let kind = kind_of(&lexed, index);
            MorphToken {
                text: lexed[index].text.clone(),
                kind,
                line: lexed[index].line,
                col: lexed[index].col,
                word: kind != MorphKind::Punct,
            }
        })
        .collect();
    Version {
        tokens,
        lines: lines.len(),
        columns,
    }
}

type Pair = (isize, isize);

#[derive(Debug, Clone, Copy)]
struct Shift {
    line: f64,
    lines: f64,
    cols: f64,
}

const STILL: Shift = Shift {
    line: -1.0,
    lines: 0.0,
    cols: 0.0,
};

struct Sides<'a> {
    from: &'a [MorphToken],
    to: &'a [MorphToken],
    from_widths: HashMap<usize, usize>,
    to_widths: HashMap<usize, usize>,
}

fn widths_of(tokens: &[MorphToken]) -> HashMap<usize, usize> {
    let mut widths = HashMap::new();
    for token in tokens {
        let end = token.col + token.text.chars().count();
        let slot = widths.entry(token.line).or_insert(0);
        *slot = (*slot).max(end);
    }
    widths
}

fn idx(value: isize) -> usize {
    usize::try_from(value).unwrap_or(0)
}

fn shift_of(sides: &Sides<'_>, pair: Pair) -> Shift {
    let anchor = &sides.from[idx(pair.0)];
    let landed = &sides.to[idx(pair.1)];
    Shift {
        line: float(anchor.line),
        lines: float(landed.line) - float(anchor.line),
        cols: float(landed.col) - float(anchor.col),
    }
}

fn expected_cost(sides: &Sides<'_>, i: usize, j: usize, shift: Shift) -> f64 {
    let source = &sides.from[i];
    let target = &sides.to[j];
    let line_off = (float(target.line) - (float(source.line) + shift.lines)).abs();
    if !exact_eq(float(source.line), shift.line) {
        let col_off = (float(target.col) - float(source.col)).abs();
        return line_off.mul_add(LINE_COST, col_off * COLUMN_COST);
    }
    let left = float(source.col) + shift.cols;
    let tail = float(sides.from_widths.get(&source.line).copied().unwrap_or(0)) - float(source.col);
    let right = float(sides.to_widths.get(&target.line).copied().unwrap_or(0)) - tail;
    let col_off = (float(target.col) - left)
        .abs()
        .min((float(target.col) - right).abs());
    line_off.mul_add(LINE_COST, col_off * COLUMN_COST)
}

fn cost_of(
    sides: &Sides<'_>,
    i: usize,
    j: usize,
    before: Option<Shift>,
    after: Option<Shift>,
) -> f64 {
    if before.is_none() && after.is_none() {
        return MAX_COST.min(expected_cost(sides, i, j, STILL));
    }
    let via = |shift: Option<Shift>| shift.map_or(MAX_COST, |s| expected_cost(sides, i, j, s));
    MAX_COST.min(via(before)).min(via(after))
}

fn align(sources: &[usize], targets: &[usize], value: &dyn Fn(usize, usize) -> f64) -> Vec<Pair> {
    let (rows, cols) = (sources.len(), targets.len());
    if rows == 0 || cols == 0 {
        return Vec::new();
    }
    let width = cols + 1;
    let mut worth = vec![0.0_f64; rows * cols];
    for a in 0..rows {
        for b in 0..cols {
            worth[a * cols + b] = value(sources[a], targets[b]);
        }
    }
    let mut table = vec![0.0_f64; (rows + 1) * width];
    for a in (0..rows).rev() {
        for b in (0..cols).rev() {
            let gain = worth[a * cols + b];
            let skip = table[(a + 1) * width + b].max(table[a * width + b + 1]);
            let take = if gain > 0.0 {
                table[(a + 1) * width + b + 1] + gain
            } else {
                0.0
            };
            table[a * width + b] = skip.max(take);
        }
    }
    let at = |a: usize, b: usize| table[a * width + b];
    let mut pairs = Vec::new();
    let (mut p, mut q) = (0, 0);
    while p < rows && q < cols {
        let gain = worth[p * cols + q];
        if gain > 0.0 && exact_eq(at(p, q), at(p + 1, q + 1) + gain) {
            pairs.push((signed(sources[p]), signed(targets[q])));
            p += 1;
            q += 1;
        } else if exact_eq(at(p, q), at(p + 1, q)) {
            p += 1;
        } else {
            q += 1;
        }
    }
    pairs
}

fn signed(value: usize) -> isize {
    isize::try_from(value).unwrap_or(isize::MAX)
}

fn increasing(candidates: &[Pair]) -> Vec<Pair> {
    let mut length = vec![1_usize; candidates.len()];
    let mut previous: Vec<Option<usize>> = vec![None; candidates.len()];
    let mut best: Option<usize> = None;
    for k in 0..candidates.len() {
        for p in 0..k {
            if candidates[p].1 < candidates[k].1 && length[p] + 1 > length[k] {
                length[k] = length[p] + 1;
                previous[k] = Some(p);
            }
        }
        if best.is_none_or(|b| length[k] > length[b]) {
            best = Some(k);
        }
    }
    let mut chain = Vec::new();
    let mut cursor = best;
    while let Some(k) = cursor {
        chain.push(candidates[k]);
        cursor = previous[k];
    }
    chain.reverse();
    chain
}

fn anchors_of(from: &[MorphToken], to: &[MorphToken]) -> Vec<Pair> {
    let mut from_count: HashMap<&str, usize> = HashMap::new();
    let mut to_count: HashMap<&str, usize> = HashMap::new();
    let mut to_index: HashMap<&str, usize> = HashMap::new();
    for token in from.iter().filter(|t| t.word) {
        *from_count.entry(&token.text).or_insert(0) += 1;
    }
    for (j, token) in to.iter().enumerate().filter(|(_, t)| t.word) {
        *to_count.entry(&token.text).or_insert(0) += 1;
        to_index.insert(&token.text, j);
    }
    let candidates: Vec<Pair> = from
        .iter()
        .enumerate()
        .filter_map(|(i, token)| {
            let j = to_index.get(token.text.as_str())?;
            let unique = from_count.get(token.text.as_str()) == Some(&1)
                && to_count.get(token.text.as_str()) == Some(&1);
            (token.word && unique).then(|| (signed(i), signed(*j)))
        })
        .collect();
    increasing(&candidates)
}

struct Guides {
    before: Vec<Option<Shift>>,
    after: Vec<Option<Shift>>,
}

fn guides_of(sides: &Sides<'_>, pairs: &[Pair]) -> Guides {
    let size = sides.from.len();
    let mut before = Vec::with_capacity(size);
    let mut last = None;
    let mut cursor = 0;
    for i in 0..size {
        while cursor < pairs.len() && pairs[cursor].0 <= signed(i) {
            last = Some(shift_of(sides, pairs[cursor]));
            cursor += 1;
        }
        before.push(last);
    }
    let mut after = vec![None; size];
    let mut next = None;
    let mut cursor = pairs.len();
    for i in (0..size).rev() {
        while cursor > 0 && pairs[cursor - 1].0 >= signed(i) {
            next = Some(shift_of(sides, pairs[cursor - 1]));
            cursor -= 1;
        }
        after[i] = next;
    }
    Guides { before, after }
}

fn partners_of(tokens: &[MorphToken]) -> Vec<Option<usize>> {
    let mut partner = vec![None; tokens.len()];
    let mut stack: Vec<usize> = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != MorphKind::Punct {
            continue;
        }
        if matches!(token.text.as_str(), "(" | "[" | "{") {
            stack.push(index);
            continue;
        }
        let opener = match token.text.as_str() {
            ")" => "(",
            "]" => "[",
            "}" => "{",
            _ => continue,
        };
        let Some(depth) = stack.iter().rposition(|&s| tokens[s].text == opener) else {
            continue;
        };
        partner[stack[depth]] = Some(index);
        partner[index] = Some(stack[depth]);
        stack.truncate(depth);
    }
    partner
}

fn lines_of(tokens: &[MorphToken]) -> Vec<(usize, Vec<usize>)> {
    let mut lines: Vec<(usize, Vec<usize>)> = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        match lines.iter_mut().find(|(line, _)| *line == token.line) {
            Some((_, list)) => list.push(index),
            None => lines.push((token.line, vec![index])),
        }
    }
    lines
}

type Signature = Vec<(usize, String)>;

fn signature_of(tokens: &[MorphToken], indices: &[usize]) -> Signature {
    let base = tokens[indices[0]].col;
    indices
        .iter()
        .map(|&i| (tokens[i].col - base, tokens[i].text.clone()))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    pub from_to: Vec<Option<usize>>,
    pub to_from: Vec<Option<usize>>,
}

const fn link(
    (i, j): (usize, usize),
    from_to: &mut [Option<usize>],
    to_from: &mut [Option<usize>],
) {
    from_to[i] = Some(j);
    to_from[j] = Some(i);
}

#[expect(
    clippy::too_many_lines,
    reason = "the film's matching passes in their order, kept together so the port reads against it"
)]
pub fn diff_tokens(from: &[MorphToken], to: &[MorphToken]) -> Diff {
    let sides = Sides {
        from,
        to,
        from_widths: widths_of(from),
        to_widths: widths_of(to),
    };
    let guides = guides_of(&sides, &anchors_of(from, to));
    let from_words: Vec<usize> = (0..from.len()).filter(|&i| from[i].word).collect();
    let to_words: Vec<usize> = (0..to.len()).filter(|&j| to[j].word).collect();
    let weight = (float(from_words.len().min(to_words.len())) + 1.0) * MAX_COST;
    let words = align(&from_words, &to_words, &|i, j| {
        if from[i].text != to[j].text {
            return 0.0;
        }
        weight - cost_of(&sides, i, j, guides.before[i], guides.after[i])
    });
    let mut from_to = vec![None; from.len()];
    let mut to_from = vec![None; to.len()];
    let mut bounds: Vec<Pair> = vec![(-1, -1)];
    bounds.extend(&words);
    bounds.push((signed(from.len()), signed(to.len())));
    for k in 0..bounds.len() - 1 {
        let (a0, b0) = bounds[k];
        let (a1, b1) = bounds[k + 1];
        let inner = k + 2 < bounds.len();
        let before = (k > 0).then(|| shift_of(&sides, bounds[k]));
        let after = inner.then(|| shift_of(&sides, bounds[k + 1]));
        let sources: Vec<usize> = (a0 + 1..a1).map(idx).filter(|&i| !from[i].word).collect();
        let targets: Vec<usize> = (b0 + 1..b1).map(idx).filter(|&j| !to[j].word).collect();
        let found = align(&sources, &targets, &|i, j| {
            if from[i].text != to[j].text {
                return 0.0;
            }
            PUNCT_VALUE - cost_of(&sides, i, j, before, after)
        });
        for (i, j) in found {
            link((idx(i), idx(j)), &mut from_to, &mut to_from);
        }
        if inner {
            link(
                (idx(bounds[k + 1].0), idx(bounds[k + 1].1)),
                &mut from_to,
                &mut to_from,
            );
        }
    }
    let from_partner = partners_of(from);
    let to_partner = partners_of(to);
    for (i, token) in from.iter().enumerate() {
        let Some(j) = from_to[i] else {
            continue;
        };
        if !matches!(token.text.as_str(), "(" | "[" | "{") {
            continue;
        }
        let (Some(close), Some(target)) = (from_partner[i], to_partner[j]) else {
            continue;
        };
        if from_to[close] == Some(target) {
            continue;
        }
        if let Some(old) = from_to[close] {
            to_from[old] = None;
        }
        if let Some(old) = to_from[target] {
            from_to[old] = None;
        }
        link((close, target), &mut from_to, &mut to_from);
    }
    let from_lines = lines_of(from);
    let mut pool: Vec<(Signature, Vec<usize>)> = Vec::new();
    for (line, indices) in &from_lines {
        let removed = indices.iter().all(|&i| from_to[i].is_none());
        if !removed || !indices.iter().any(|&i| from[i].word) {
            continue;
        }
        let key = signature_of(from, indices);
        match pool.iter_mut().find(|(k, _)| *k == key) {
            Some((_, list)) => list.push(*line),
            None => pool.push((key, vec![*line])),
        }
    }
    for (line, indices) in lines_of(to) {
        if !indices.iter().all(|&j| to_from[j].is_none()) {
            continue;
        }
        let key = signature_of(to, &indices);
        let Some((_, list)) = pool.iter_mut().find(|(k, _)| *k == key) else {
            continue;
        };
        if list.is_empty() {
            continue;
        }
        let mut best = 0;
        for (k, candidate) in list.iter().enumerate() {
            if candidate.abs_diff(line) < list[best].abs_diff(line) {
                best = k;
            }
        }
        let source = list.remove(best);
        let origin = from_lines
            .iter()
            .find(|(l, _)| *l == source)
            .map_or(&[][..], |(_, v)| v.as_slice());
        for (k, &j) in indices.iter().enumerate() {
            if let Some(&i) = origin.get(k) {
                link((i, j), &mut from_to, &mut to_from);
            }
        }
    }
    Diff { from_to, to_from }
}

fn ranks(lines: &[usize]) -> HashMap<usize, f64> {
    let mut distinct = lines.to_vec();
    distinct.sort_unstable();
    distinct.dedup();
    let last = distinct.len().saturating_sub(1);
    distinct
        .iter()
        .enumerate()
        .map(|(i, line)| {
            (
                *line,
                if last > 0 {
                    float(i) / float(last)
                } else {
                    0.0
                },
            )
        })
        .collect()
}

fn changed_lines(from: &Version, to: &Version, diff: &Diff) -> Vec<usize> {
    let mut sizes: HashMap<usize, usize> = HashMap::new();
    for token in &from.tokens {
        *sizes.entry(token.line).or_insert(0) += 1;
    }
    let mut changed = Vec::new();
    for (line, indices) in lines_of(&to.tokens) {
        let sources: Vec<Option<usize>> = indices.iter().map(|&j| diff.to_from[j]).collect();
        let origin = sources[0].map(|i| from.tokens[i].line);
        let copied = sources
            .iter()
            .all(|s| s.is_some_and(|i| Some(from.tokens[i].line) == origin));
        let same_size = origin.is_some_and(|o| sizes.get(&o) == Some(&indices.len()));
        if !(copied && same_size) {
            changed.push(line);
        }
    }
    changed.sort_unstable();
    changed
}

#[derive(Debug, Clone)]
pub struct Morph {
    pub diff: Diff,
    exit_rank: HashMap<usize, f64>,
    move_rank: HashMap<usize, f64>,
    enter_rank: HashMap<usize, f64>,
    pub changed: Vec<usize>,
}

pub fn morph_of(from: &Version, to: &Version) -> Morph {
    let diff = diff_tokens(&from.tokens, &to.tokens);
    let leaving: Vec<usize> = from
        .tokens
        .iter()
        .enumerate()
        .filter(|(i, _)| diff.from_to[*i].is_none())
        .map(|(_, t)| t.line)
        .collect();
    let mut moving = Vec::new();
    let mut entering = Vec::new();
    for (j, token) in to.tokens.iter().enumerate() {
        match diff.to_from[j] {
            None => entering.push(token.line),
            Some(i) => {
                let source = &from.tokens[i];
                if source.line != token.line || source.col != token.col {
                    moving.push(token.line);
                }
            }
        }
    }
    Morph {
        exit_rank: ranks(&leaving),
        move_rank: ranks(&moving),
        enter_rank: ranks(&entering),
        changed: changed_lines(from, to, &diff),
        diff,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MorphTiming {
    pub start: f64,
    pub end: f64,
    pub highlight_start: f64,
    pub fall_start: f64,
    pub highlight_end: f64,
}

#[derive(Debug, Clone)]
pub struct Timeline {
    pub morph: f64,
    pub highlight: bool,
    pub morphs: Vec<MorphTiming>,
    pub duration: f64,
}

fn phase(value: f64, morph: f64) -> f64 {
    value * morph / BASE
}

pub fn timeline(steps: &[f64], morph_duration: f64, highlight: bool, hold: f64) -> Timeline {
    let morph = morph_duration.clamp(MIN_MORPH, MAX_MORPH);
    let mut starts = Vec::new();
    let mut ready = 0.0_f64;
    for at in steps.iter().skip(1) {
        let start = at.max(ready);
        starts.push(start);
        ready = start + morph;
    }
    let morphs: Vec<MorphTiming> = starts
        .iter()
        .enumerate()
        .map(|(index, start)| {
            let highlight_start = start + phase(ENTER_START, morph);
            let natural = highlight_start + RISE + hold;
            let next = starts.get(index + 1).copied().unwrap_or(natural);
            let fall_start = natural.min(next);
            MorphTiming {
                start: *start,
                end: start + morph,
                highlight_start,
                fall_start,
                highlight_end: fall_start + FALL,
            }
        })
        .collect();
    let settled = morphs.last().map_or(0.0, |last| {
        if highlight {
            last.end.max(last.highlight_end)
        } else {
            last.end
        }
    });
    Timeline {
        morph,
        highlight,
        morphs,
        duration: settled + HOLD,
    }
}

#[derive(Debug, Clone)]
pub struct RenderToken {
    pub key: String,
    pub text: String,
    pub kind: MorphKind,
    pub role: MorphRole,
    pub line: f64,
    pub col: f64,
    pub opacity: f64,
    pub scale: f64,
    pub blur: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Row {
    pub row: usize,
    pub opacity: f64,
}

#[derive(Debug, Clone)]
pub struct State {
    pub lines: f64,
    pub tokens: Vec<RenderToken>,
    pub line_numbers: Vec<Row>,
    pub highlights: Vec<Row>,
}

const fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn ease_out_cubic(value: f64) -> f64 {
    1.0 - (1.0 - clamp01(value)).powi(3)
}

fn ease_out_quad(value: f64) -> f64 {
    (1.0 - clamp01(value)).mul_add(-(1.0 - clamp01(value)), 1.0)
}

fn smoothstep(value: f64) -> f64 {
    let t = clamp01(value);
    t * t * 2.0f64.mul_add(-t, 3.0)
}

fn resting(version: &Version, index: usize) -> Vec<RenderToken> {
    version
        .tokens
        .iter()
        .enumerate()
        .map(|(j, token)| RenderToken {
            key: format!("{index}:{j}"),
            text: token.text.clone(),
            kind: token.kind,
            role: MorphRole::Rest,
            line: float(token.line),
            col: float(token.col),
            opacity: 1.0,
            scale: 1.0,
            blur: 0.0,
        })
        .collect()
}

fn rows_of(count: usize, opacity: &dyn Fn(usize) -> f64) -> Vec<Row> {
    (0..count)
        .map(|row| Row {
            row,
            opacity: opacity(row),
        })
        .collect()
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film eases each token with separate multiplies and adds, and the frames have to match it"
)]
fn morph_frame(
    versions: &[Version],
    morphs: &[Morph],
    index: usize,
    local: f64,
    morph: f64,
    blur: f64,
) -> State {
    let data = &morphs[index];
    let (from, to) = (&versions[index], &versions[index + 1]);
    let mut tokens = Vec::new();
    for (i, token) in from.tokens.iter().enumerate() {
        if data.diff.from_to[i].is_some() {
            continue;
        }
        let rank = data.exit_rank.get(&token.line).copied().unwrap_or(0.0);
        let delay = phase(EXIT_SPREAD, morph) * rank;
        let progress = (local - delay) / phase(EXIT_DURATION, morph);
        if progress >= 1.0 {
            continue;
        }
        let eased = ease_out_cubic(progress);
        tokens.push(RenderToken {
            key: format!("{index}>{i}"),
            text: token.text.clone(),
            kind: token.kind,
            role: MorphRole::Exit,
            line: float(token.line),
            col: float(token.col),
            opacity: 1.0 - eased,
            scale: 1.0 - SHRINK * eased,
            blur: blur * eased,
        });
    }
    let travel =
        |delay: f64| (local - phase(MOVE_START, morph) - delay) / phase(MOVE_DURATION, morph);
    for (j, token) in to.tokens.iter().enumerate() {
        let Some(i) = data.diff.to_from[j] else {
            continue;
        };
        let source = &from.tokens[i];
        let still = source.line == token.line && source.col == token.col;
        let rank = data.move_rank.get(&token.line).copied().unwrap_or(0.0);
        let progress = travel(phase(CASCADE, morph) * rank);
        let vertical = GLIDE.at(clamp01(progress / (1.0 - ARC)));
        let horizontal = GLIDE.at(clamp01((progress - ARC) / (1.0 - ARC)));
        tokens.push(RenderToken {
            key: format!("{}:{j}", index + 1),
            text: token.text.clone(),
            kind: token.kind,
            role: if still {
                MorphRole::Stay
            } else {
                MorphRole::Move
            },
            line: float(source.line) + (float(token.line) - float(source.line)) * vertical,
            col: float(source.col) + (float(token.col) - float(source.col)) * horizontal,
            opacity: 1.0,
            scale: 1.0,
            blur: 0.0,
        });
    }
    for (j, token) in to.tokens.iter().enumerate() {
        if data.diff.to_from[j].is_some() {
            continue;
        }
        let rank = data.enter_rank.get(&token.line).copied().unwrap_or(0.0);
        let start = phase(ENTER_START + ENTER_SPREAD * rank, morph);
        let progress = (local - start) / phase(ENTER_DURATION, morph);
        if progress <= 0.0 {
            continue;
        }
        let eased = ease_out_cubic(progress);
        tokens.push(RenderToken {
            key: format!("{}:{j}", index + 1),
            text: token.text.clone(),
            kind: token.kind,
            role: MorphRole::Enter,
            line: float(token.line),
            col: float(token.col),
            opacity: eased,
            scale: GROW_FROM + (1.0 - GROW_FROM) * eased,
            blur: blur * (1.0 - eased),
        });
    }
    let shrinking = to.lines < from.lines;
    let lag = if shrinking {
        phase(CASCADE, morph)
    } else {
        0.0
    };
    let grown = GLIDE.at(clamp01(travel(lag) / (1.0 - ARC)));
    let lines = float(from.lines) + (float(to.lines) - float(from.lines)) * grown;
    let shared = from.lines.min(to.lines);
    let line_numbers = rows_of(from.lines.max(to.lines), &|row| {
        if row < shared {
            1.0
        } else if shrinking {
            1.0 - grown
        } else {
            grown
        }
    });
    State {
        lines,
        tokens,
        line_numbers,
        highlights: Vec::new(),
    }
}

fn highlights_at(
    time: f64,
    timeline: &Timeline,
    morphs: &[Morph],
    active: Option<usize>,
) -> Vec<Row> {
    let mut rows = Vec::new();
    if !timeline.highlight {
        return rows;
    }
    let Some(active) = active else {
        return rows;
    };
    for index in [active.checked_sub(1), Some(active)].into_iter().flatten() {
        let morph = &timeline.morphs[index];
        let rise = ease_out_quad((time - morph.highlight_start) / RISE);
        let fall = smoothstep((time - morph.fall_start) / FALL);
        let opacity = rise * (1.0 - fall);
        if opacity <= 0.0 {
            continue;
        }
        rows.extend(
            morphs[index]
                .changed
                .iter()
                .map(|&row| Row { row, opacity }),
        );
    }
    rows
}

pub fn state_at(
    time: f64,
    timeline: &Timeline,
    versions: &[Version],
    morphs: &[Morph],
    blur: f64,
) -> State {
    let active = timeline.morphs.iter().rposition(|m| m.start <= time);
    let highlights = highlights_at(time, timeline, morphs, active);
    let rest = |version: usize| State {
        lines: float(versions[version].lines),
        tokens: resting(&versions[version], version),
        line_numbers: rows_of(versions[version].lines, &|_| 1.0),
        highlights: highlights.clone(),
    };
    let Some(active) = active else {
        return rest(0);
    };
    let local = time - timeline.morphs[active].start;
    if local >= timeline.morph {
        return rest(active + 1);
    }
    State {
        highlights: highlights.clone(),
        ..morph_frame(versions, morphs, active, local, timeline.morph, blur)
    }
}
