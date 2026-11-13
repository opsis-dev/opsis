use opsis_scene::easing::Easing;
use opsis_scene::random::random;
use opsis_types::numbers::{float, truncate};

use crate::elements::units::js_fixed;

pub const PANEL_IN: f64 = 14.0;
pub const PANEL_RISE: f64 = 14.0;
pub const BUBBLE_AT: f64 = 4.0;
pub const BUBBLE_IN: f64 = 10.0;
pub const BUBBLE_RISE: f64 = 8.0;
pub const RUN_START: f64 = 16.0;
pub const STATUS_IN: f64 = 6.0;
const THINK_LEAD: f64 = 14.0;
const THINK_TAIL: f64 = 8.0;
pub const SWAP: f64 = 8.0;
pub const SWAP_RISE: f64 = 4.0;
const TOOL_GAP: f64 = 5.0;
const TOOL_MIN: f64 = 12.0;
const TOOL_MAX: f64 = 120.0;
const ANSWER_GAP: f64 = 8.0;
const ROW_IN: f64 = 8.0;
const ROW_FADE: f64 = 6.0;
const LABEL_DELAY: f64 = 2.0;
const LABEL_FADE: f64 = 6.0;
const SPIN: f64 = 12.0;
const ARC: f64 = 0.28;
const RESOLVE: f64 = 8.0;
const CHECK_FROM: f64 = 3.0;
const CHECK_TO: f64 = 10.0;
const CHIP_LEAD: f64 = 4.0;
const CHIP_SPAN: f64 = 0.7;
const FADE: f64 = 5.0;
const MORPH: f64 = 8.0;
const POP: f64 = 8.0;
const CODE_OPEN: f64 = 8.0;
const LINE_GROW: f64 = 5.0;
const SCROLL: f64 = 24.0;
const SCROLL_LEAD: f64 = 16.0;
const SHIMMER_PERIOD: f64 = 30.0;
const SHIMMER_BAND: f64 = 3.0;
const SHIMMER_BASE: f64 = 0.45;
const DOT_BACK: f64 = 1.7;
const CITE_BACK: f64 = 2.0;
const CHIP_BACK: f64 = 1.7;
const MIN_COMPRESSION: f64 = 0.25;
const MAX_COMPRESSION: f64 = 20.0;
const SIZE_WEIGHTS: [f64; 6] = [8.0, 16.0, 22.0, 22.0, 17.0, 15.0];
const GAP_WEIGHTS: [f64; 4] = [42.0, 33.0, 16.0, 9.0];
const PAUSE_CHANCE: f64 = 0.06;
const PAUSE_MIN: f64 = 4.0;
const PAUSE_SPREAD: f64 = 4.0;
const BLOCK_MIN: f64 = 2.0;
const BLOCK_SPREAD: f64 = 3.0;
const REF_HEIGHT: f64 = 720.0;
pub const PANEL_W: f64 = 800.0;
const MARGIN_X: f64 = 40.0;
const MARGIN_Y: f64 = 40.0;
const MIN_PANEL_H: f64 = 360.0;
const MAX_PANEL_H: f64 = 960.0;
pub const HEADER_H: f64 = 48.0;
pub const PAD_X: f64 = 32.0;
const PAD_TOP: f64 = 24.0;
const SAFE: f64 = 40.0;
pub const COLUMN: f64 = 2.0f64.mul_add(-PAD_X, PANEL_W);
const MEASURE: f64 = 680.0;
pub const BODY_SIZE: f64 = 16.0;
pub const BODY_LINE: f64 = 26.0;
pub const BUBBLE_SIZE: f64 = 16.0;
pub const BUBBLE_LINE: f64 = 24.0;
pub const BUBBLE_PAD_Y: f64 = 10.0;
pub const BUBBLE_PAD_X: f64 = 16.0;
const BUBBLE_WIDTH: f64 = 520.0;
const BUBBLE_GAP: f64 = 22.0;
pub const STATUS_H: f64 = 22.0;
const STATUS_GAP: f64 = 10.0;
pub const PLAN_SIZE: f64 = 15.0;
pub const PLAN_LINE: f64 = 24.0;
const PLAN_GAP: f64 = 4.0;
pub const PLAN_INDENT: f64 = 36.0;
const PLAN_WIDTH: f64 = 640.0;
const PLAN_AFTER: f64 = 20.0;
pub const ROW_H: f64 = 40.0;
pub const CHIPS_H: f64 = 34.0;
const TOOLS_AFTER: f64 = 24.0;
pub const ITEM_INDENT: f64 = 22.0;
const ITEM_GAP: f64 = 4.0;
const BLOCK_GAP: f64 = 12.0;
const HEADING_GAP: f64 = 8.0;
const CODE_GAP: f64 = 14.0;
pub const CODE_SIZE: f64 = 14.0;
pub const CODE_LINE: f64 = 22.0;
pub const CODE_HEAD: f64 = 34.0;
pub const CODE_PAD: f64 = 10.0;
pub const CHIP_SIZE: f64 = 12.5;
pub const CHIP_GAP: f64 = 6.0;
const CHIP_MAX: usize = 6;
const CHIP_ROOM: f64 = COLUMN - 2.0 - 40.0 - 14.0;
const EXPO: Easing = Easing::Bezier(0.16, 1.0, 0.3, 1.0);

pub type Units = Vec<u16>;

pub fn units(text: &str) -> Units {
    text.encode_utf16().collect()
}

pub fn text_of(units: &[u16]) -> String {
    String::from_utf16_lossy(units)
}

pub fn round3(value: f64) -> f64 {
    value.mul_add(1000.0, 0.5).floor() / 1000.0
}

const fn progress(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

pub fn expo(value: f64) -> f64 {
    EXPO.at(progress(value))
}

pub fn ease_out(value: f64) -> f64 {
    1.0 - (1.0 - progress(value)).powi(3)
}

pub fn eased(value: f64) -> f64 {
    let t = progress(value);
    t * t * 2.0f64.mul_add(-t, 3.0)
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film's back easing adds its terms separately, and the frames have to match it"
)]
pub fn back_out(value: f64, overshoot: f64) -> f64 {
    let t = progress(value) - 1.0;
    1.0 + (overshoot + 1.0) * t.powi(3) + overshoot * t.powi(2)
}

pub fn format_seconds(seconds: f64) -> String {
    let value = seconds.max(0.0);
    if value < 60.0 {
        return format!("{}s", js_fixed(value, 1));
    }
    let minutes = (value / 60.0).floor();
    let rest = minutes.mul_add(-60.0, value).floor();
    format!("{minutes}m {rest:02}s")
}

fn is_letter(code: u16) -> bool {
    (65..=90).contains(&code) || (97..=122).contains(&code) || code >= 192
}

const fn is_digit(code: u16) -> bool {
    code >= 48 && code <= 57
}

const NEWLINE: u16 = 10;
const SPACE: u16 = 32;
const TAB: u16 = 9;

fn tokenize(text: &[u16]) -> Vec<usize> {
    let mut ends = Vec::new();
    let mut at = 0;
    while at < text.len() {
        if text[at] == NEWLINE {
            at += 1;
            ends.push(at);
            continue;
        }
        while at < text.len() && (text[at] == SPACE || text[at] == TAB) {
            at += 1;
        }
        if at < text.len() && text[at] != NEWLINE {
            let code = text[at];
            if is_letter(code) {
                while at < text.len() && is_letter(text[at]) {
                    at += 1;
                }
            } else if is_digit(code) {
                let stop = text.len().min(at + 3);
                while at < stop && is_digit(text[at]) {
                    at += 1;
                }
            } else {
                at += 1;
            }
        }
        ends.push(at);
    }
    ends
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chunk {
    pub at: f64,
    pub start: usize,
    pub end: usize,
}

fn pick(weights: &[f64], value: f64) -> usize {
    let total: f64 = weights.iter().sum();
    let mut cursor = value * total;
    for (index, weight) in weights.iter().enumerate() {
        if cursor < *weight {
            return index;
        }
        cursor -= weight;
    }
    weights.len() - 1
}

pub fn chunks_of(text: &[u16], seed: u32, stream: &str) -> Vec<Chunk> {
    let ends = tokenize(text);
    let start_of = |token: usize| if token > 0 { ends[token - 1] } else { 0 };
    let is_newline =
        |token: usize| token < ends.len() && text.get(start_of(token)) == Some(&NEWLINE);
    let key = format!("agent-run-{seed}-{stream}");
    let mut chunks = Vec::new();
    let (mut token, mut at, mut block) = (0, 0.0, false);
    while token < ends.len() {
        let index = chunks.len();
        let size = pick(&SIZE_WEIGHTS, random(format!("{key}-size-{index}"))) + 1;
        let mut stop = ends.len().min(token + size);
        let pair = (token..stop).find(|&c| is_newline(c) && is_newline(c + 1));
        let mut cut = pair.is_some();
        if let Some(cursor) = pair {
            stop = cursor + 2;
        }
        if !cut && is_newline(stop - 1) && is_newline(stop) {
            stop += 1;
            cut = true;
        }
        if cut {
            while is_newline(stop) {
                stop += 1;
            }
        }
        if index > 0 {
            at += float(pick(&GAP_WEIGHTS, random(format!("{key}-gap-{index}"))) + 1);
            if random(format!("{key}-pause-{index}")) < PAUSE_CHANCE {
                at += PAUSE_MIN + (random(format!("{key}-hold-{index}")) * PAUSE_SPREAD).floor();
            }
            if block {
                at += BLOCK_MIN + (random(format!("{key}-block-{index}")) * BLOCK_SPREAD).floor();
            }
        }
        chunks.push(Chunk {
            at,
            start: start_of(token),
            end: ends[stop.min(ends.len()) - 1],
        });
        block = cut;
        token = stop;
    }
    chunks
}

fn last_arrival(chunks: &[Chunk]) -> f64 {
    chunks.last().map_or(0.0, |c| c.at)
}

pub fn chunk_index_at(chunks: &[Chunk], offset: usize) -> usize {
    chunks.partition_point(|c| c.end <= offset)
}

pub fn arrival(chunks: &[Chunk], offset: usize) -> f64 {
    chunks
        .get(chunk_index_at(chunks, offset))
        .map_or(f64::INFINITY, |c| c.at)
}

fn arrival_signed(chunks: &[Chunk], offset: isize) -> f64 {
    usize::try_from(offset).map_or_else(
        |_| chunks.first().map_or(f64::INFINITY, |c| c.at),
        |o| arrival(chunks, o),
    )
}

pub fn reveal(chunks: &[Chunk], local: f64) -> usize {
    let count = chunks.partition_point(|c| c.at <= local);
    count.checked_sub(1).map_or(0, |i| chunks[i].end)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceKind {
    Text,
    Bold,
    Code,
    Cite,
    Punct,
    String,
    Comment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    pub kind: PieceKind,
    pub text: Units,
    pub raw: usize,
    pub end: usize,
}

fn find(source: &[u16], needle: &[u16], from: usize) -> Option<usize> {
    (from..source.len()).find(|&i| source[i..].starts_with(needle))
}

fn parse_inline(source: &[u16], base: usize) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut text: Units = Vec::new();
    let mut text_start = 0;
    let flush = |out: &mut Vec<Piece>, text: &mut Units, text_start: usize, at: usize| {
        if !text.is_empty() {
            out.push(Piece {
                kind: PieceKind::Text,
                text: std::mem::take(text),
                raw: base + text_start,
                end: base + at,
            });
        }
    };
    let stars = units("**");
    let mut at = 0;
    while at < source.len() {
        if source[at..].starts_with(&stars)
            && let Some(close) = find(source, &stars, at + 2).filter(|c| *c > at + 2)
        {
            flush(&mut out, &mut text, text_start, at);
            out.push(Piece {
                kind: PieceKind::Bold,
                text: source[at + 2..close].to_vec(),
                raw: base + at + 2,
                end: base + close + 2,
            });
            at = close + 2;
            text_start = at;
            continue;
        }
        if source[at] == u16::from(b'`')
            && let Some(close) = find(source, &[u16::from(b'`')], at + 1).filter(|c| *c > at + 1)
        {
            flush(&mut out, &mut text, text_start, at);
            out.push(Piece {
                kind: PieceKind::Code,
                text: source[at + 1..close].to_vec(),
                raw: base + at + 1,
                end: base + close + 1,
            });
            at = close + 1;
            text_start = at;
            continue;
        }
        if source[at] == u16::from(b'[') {
            let digits = source[at + 1..]
                .iter()
                .take(3)
                .take_while(|c| is_digit(**c))
                .count();
            if digits > 0 && source.get(at + 1 + digits) == Some(&u16::from(b']')) {
                flush(&mut out, &mut text, text_start, at);
                out.push(Piece {
                    kind: PieceKind::Cite,
                    text: source[at + 1..at + 1 + digits].to_vec(),
                    raw: base + at,
                    end: base + at + digits + 2,
                });
                at += digits + 2;
                text_start = at;
                continue;
            }
        }
        if text.is_empty() {
            text_start = at;
        }
        text.push(source[at]);
        at += 1;
    }
    flush(&mut out, &mut text, text_start, at);
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading {
        level: u8,
        raw: usize,
        body: usize,
        end: usize,
        inline: Vec<Piece>,
    },
    Paragraph {
        raw: usize,
        end: usize,
        inline: Vec<Piece>,
    },
    Item {
        raw: usize,
        body: usize,
        end: usize,
        inline: Vec<Piece>,
    },
    Code {
        raw: usize,
        open: usize,
        end: usize,
        lang: Units,
        lang_raw: usize,
        lines: Vec<(Units, usize)>,
    },
}

fn is_blank(text: &[u16]) -> bool {
    text_of(text).trim().is_empty()
}

fn trim_start_len(text: &[u16]) -> usize {
    let decoded = text_of(text);
    units(decoded.trim_start()).len()
}

fn heading_line(text: &[u16]) -> Option<(usize, usize)> {
    let hashes = text.iter().take_while(|c| **c == u16::from(b'#')).count();
    if !(1..=3).contains(&hashes) {
        return None;
    }
    let spaces = text[hashes..]
        .iter()
        .take_while(|c| **c == SPACE || **c == TAB)
        .count();
    (spaces > 0).then_some((hashes, hashes + spaces))
}

fn item_line(text: &[u16]) -> Option<usize> {
    let first = *text.first()?;
    if first != u16::from(b'-') && first != u16::from(b'*') {
        return None;
    }
    let spaces = text[1..]
        .iter()
        .take_while(|c| **c == SPACE || **c == TAB)
        .count();
    (spaces > 0).then_some(1 + spaces)
}

fn fence(text: &[u16]) -> bool {
    text.starts_with(&units("```"))
}

fn is_structural(text: &[u16]) -> bool {
    fence(text) || heading_line(text).is_some() || item_line(text).is_some()
}

pub fn parse_markdown(markdown: &[u16]) -> Vec<Block> {
    let mut lines: Vec<(Units, usize)> = Vec::new();
    let mut offset = 0;
    for line in markdown.split(|c| *c == NEWLINE) {
        lines.push((line.to_vec(), offset));
        offset += line.len() + 1;
    }
    let mut blocks = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let (text, raw) = &lines[index];
        if is_blank(text) {
            index += 1;
            continue;
        }
        if fence(text) {
            let rest = &text[3..];
            let mut body = Vec::new();
            let mut cursor = index + 1;
            while cursor < lines.len() && !fence(&lines[cursor].0) {
                body.push(lines[cursor].clone());
                cursor += 1;
            }
            let closed = cursor < lines.len();
            let last = if closed {
                &lines[cursor]
            } else {
                body.last().unwrap_or(&lines[index])
            };
            let lang = units(text_of(rest).trim());
            blocks.push(Block::Code {
                raw: *raw,
                open: raw + 3,
                end: last.1 + last.0.len(),
                lang,
                lang_raw: raw + 3 + (rest.len() - trim_start_len(rest)),
                lines: body,
            });
            index = if closed { cursor + 1 } else { cursor };
            continue;
        }
        if let Some((hashes, start)) = heading_line(text) {
            let rest = &text[start..];
            let body = raw + text.len() - rest.len();
            blocks.push(Block::Heading {
                level: if hashes == 1 { 1 } else { 2 },
                raw: *raw,
                body,
                end: raw + text.len(),
                inline: parse_inline(rest, body),
            });
            index += 1;
            continue;
        }
        if let Some(start) = item_line(text) {
            let rest = &text[start..];
            let body = raw + text.len() - rest.len();
            blocks.push(Block::Item {
                raw: *raw,
                body,
                end: raw + text.len(),
                inline: parse_inline(rest, body),
            });
            index += 1;
            continue;
        }
        let mut cursor = index;
        while cursor < lines.len()
            && !is_blank(&lines[cursor].0)
            && (cursor == index || !is_structural(&lines[cursor].0))
        {
            cursor += 1;
        }
        let (last, last_raw) = &lines[cursor - 1];
        let end = last_raw + last.len();
        let source: Units = markdown[*raw..end]
            .iter()
            .map(|c| if *c == NEWLINE { SPACE } else { *c })
            .collect();
        blocks.push(Block::Paragraph {
            raw: *raw,
            end,
            inline: parse_inline(&source, *raw),
        });
        index = cursor;
    }
    blocks
}

fn char_width(c: char) -> f64 {
    if " .,:;'!|iljI`".contains(c) {
        0.3
    } else if "ftr()[]{}/\\-".contains(c) {
        0.44
    } else if "MW@%\u{2014}".contains(c) {
        1.0
    } else if "mw\u{2026}".contains(c) {
        0.9
    } else if "<>=+~#$&\u{2192}".contains(c) {
        0.68
    } else if c.is_ascii_digit() {
        0.66
    } else if c.is_ascii_uppercase() {
        0.74
    } else {
        0.62
    }
}

pub fn estimate_width(text: &[u16], size: f64, mono: bool) -> f64 {
    let em: f64 = text_of(text)
        .chars()
        .map(|c| if mono { 0.6 } else { char_width(c) })
        .sum();
    em * size
}

fn piece_width(piece: &Piece, size: f64) -> f64 {
    match piece.kind {
        PieceKind::Cite => 7.0f64.mul_add(float(piece.text.len().saturating_sub(1)), 20.0),
        PieceKind::Code => estimate_width(&piece.text, size * 0.875, true) + 10.0,
        _ => estimate_width(&piece.text, size, false),
    }
}

struct Atom {
    pieces: Vec<Piece>,
    width: f64,
    space: bool,
}

fn atoms_of(inline: &[Piece], size: f64) -> Vec<Atom> {
    let mut atoms = Vec::new();
    let mut word: Vec<Piece> = Vec::new();
    let mut word_width = 0.0;
    for segment in inline {
        if matches!(segment.kind, PieceKind::Cite | PieceKind::Code) {
            word_width += piece_width(segment, size);
            word.push(segment.clone());
            continue;
        }
        let mut at = 0;
        while at < segment.text.len() {
            let space = segment.text[at] == SPACE;
            let stop = at
                + segment.text[at..]
                    .iter()
                    .take_while(|c| (**c == SPACE) == space)
                    .count();
            let piece = Piece {
                kind: segment.kind,
                text: segment.text[at..stop].to_vec(),
                raw: segment.raw + at,
                end: segment.raw + stop,
            };
            if space {
                if !word.is_empty() {
                    atoms.push(Atom {
                        pieces: std::mem::take(&mut word),
                        width: word_width,
                        space: false,
                    });
                }
                word_width = 0.0;
                atoms.push(Atom {
                    width: piece_width(&piece, size),
                    pieces: vec![piece],
                    space: true,
                });
            } else {
                word_width += piece_width(&piece, size);
                word.push(piece);
            }
            at = stop;
        }
    }
    if !word.is_empty() {
        atoms.push(Atom {
            pieces: word,
            width: word_width,
            space: false,
        });
    }
    atoms
}

#[expect(
    clippy::suspicious_operation_groupings,
    reason = "a piece joins the one before when it starts where that one ends"
)]
fn merge_pieces(pieces: Vec<Piece>) -> Vec<Piece> {
    let mut merged: Vec<Piece> = Vec::new();
    for piece in pieces {
        let text = matches!(piece.kind, PieceKind::Text | PieceKind::Bold);
        if let Some(last) = merged.last_mut()
            && text
            && last.kind == piece.kind
            && last.end == piece.raw
        {
            last.text.extend(&piece.text);
            last.end = piece.end;
            continue;
        }
        merged.push(piece);
    }
    merged
}

pub fn wrap_inline(inline: &[Piece], size: f64, max_width: f64) -> Vec<Vec<Piece>> {
    let mut lines = Vec::new();
    let mut line: Vec<Piece> = Vec::new();
    let mut width = 0.0;
    let mut pending: Option<Atom> = None;
    for atom in atoms_of(inline, size) {
        if atom.space {
            if !line.is_empty() {
                pending = Some(atom);
            }
            continue;
        }
        let gap = pending.as_ref().map_or(0.0, |p| p.width);
        if !line.is_empty() && width + gap + atom.width > max_width {
            lines.push(std::mem::take(&mut line));
            line.extend(atom.pieces);
            width = atom.width;
        } else {
            if let Some(p) = pending.take()
                && !line.is_empty()
            {
                line.extend(p.pieces);
                width += gap;
            }
            line.extend(atom.pieces);
            width += atom.width;
        }
        pending = None;
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines.into_iter().map(merge_pieces).collect()
}

fn plain_inline(text: &[u16], raw: usize) -> Vec<Piece> {
    if text.is_empty() {
        return Vec::new();
    }
    vec![Piece {
        kind: PieceKind::Text,
        text: text.to_vec(),
        raw,
        end: raw + text.len(),
    }]
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    pub y: f64,
    pub height: f64,
    pub raw: usize,
    pub pieces: Vec<Piece>,
}

fn text_lines(pieces: Vec<Vec<Piece>>, line_height: f64, raw: usize) -> Vec<TextLine> {
    if pieces.is_empty() {
        return vec![TextLine {
            y: 0.0,
            height: line_height,
            raw,
            pieces: Vec::new(),
        }];
    }
    pieces
        .into_iter()
        .enumerate()
        .map(|(index, line)| TextLine {
            y: float(index) * line_height,
            height: line_height,
            raw: line.first().map_or(raw, |p| p.raw),
            pieces: line,
        })
        .collect()
}

fn code_pieces(text: &[u16], raw: usize) -> Vec<Piece> {
    let mut pieces: Vec<Piece> = Vec::new();
    let mut push = |kind: PieceKind, from: usize, to: usize| {
        if to <= from {
            return;
        }
        if let Some(last) = pieces.last_mut()
            && last.kind == kind
            && last.end == raw + from
        {
            last.text.extend(&text[from..to]);
            last.end = raw + to;
            return;
        }
        pieces.push(Piece {
            kind,
            text: text[from..to].to_vec(),
            raw: raw + from,
            end: raw + to,
        });
    };
    let punct = units("<>{}()[]/=;,.:+-*&|!?");
    let mut at = 0;
    while at < text.len() {
        let c = text[at];
        if text[at..].starts_with(&units("//")) {
            push(PieceKind::Comment, at, text.len());
            break;
        }
        if c == u16::from(b'"') || c == u16::from(b'\'') || c == u16::from(b'`') {
            let stop = find(text, &[c], at + 1).map_or(text.len(), |close| close + 1);
            push(PieceKind::String, at, stop);
            at = stop;
            continue;
        }
        let kind = if punct.contains(&c) {
            PieceKind::Punct
        } else {
            PieceKind::Text
        };
        push(kind, at, at + 1);
        at += 1;
    }
    pieces
}

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub unit: f64,
    pub panel_height: f64,
    pub viewport: f64,
}

pub fn frame_of(width: f64, height: f64) -> Frame {
    let scale = (height / REF_HEIGHT).min(width / MARGIN_X.mul_add(2.0, PANEL_W));
    let panel_height = 2.0f64
        .mul_add(-MARGIN_Y, height / scale)
        .clamp(MIN_PANEL_H, MAX_PANEL_H);
    Frame {
        unit: round3(scale),
        panel_height: round3(panel_height),
        viewport: round3(panel_height - HEADER_H - 2.0),
    }
}

#[derive(Debug, Clone)]
pub struct Step {
    pub name: String,
    pub detail: String,
    pub duration: f64,
    pub icon: Option<&'static str>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Run {
    pub title: String,
    pub prompt: Units,
    pub plan: Vec<Units>,
    pub steps: Vec<Step>,
    pub answer: Units,
}

#[derive(Debug, Clone, Copy)]
struct Segment {
    from: f64,
    to: f64,
    real_from: f64,
    real: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct ToolSpan {
    pub start: f64,
    pub end: f64,
    pub span: f64,
}

#[derive(Debug, Clone)]
pub struct Timeline {
    pub plan_start: f64,
    pub think_end: f64,
    pub think_seconds: f64,
    pub tools: Vec<ToolSpan>,
    pub answer_start: f64,
    pub run_end: f64,
    real_total: f64,
    segments: Vec<Segment>,
    pub plan_chunks: Vec<Chunk>,
    pub answer_chunks: Vec<Chunk>,
}

pub fn timeline_of(run: &Run, seed: u32, compression: f64) -> Timeline {
    let compression = compression.clamp(MIN_COMPRESSION, MAX_COMPRESSION);
    let per_frame = compression / 30.0;
    let plan_text: Units = run.plan.join(&NEWLINE);
    let plan_chunks = if plan_text.is_empty() {
        Vec::new()
    } else {
        chunks_of(&plan_text, seed, "plan")
    };
    let plan_start = RUN_START + THINK_LEAD;
    let plan_end = plan_start + last_arrival(&plan_chunks);
    let think_end = plan_end + THINK_TAIL;
    let mut segments = Vec::new();
    let mut real = 0.0;
    let mut push = |from: f64, to: f64, seconds: f64, real: &mut f64| {
        segments.push(Segment {
            from,
            to,
            real_from: *real,
            real: seconds,
        });
        *real += seconds;
    };
    push(
        RUN_START,
        think_end,
        (think_end - RUN_START) * per_frame,
        &mut real,
    );
    let think_seconds = real;
    let mut tools = Vec::new();
    let mut cursor = think_end;
    for step in &run.steps {
        push(cursor, cursor + TOOL_GAP, TOOL_GAP * per_frame, &mut real);
        cursor += TOOL_GAP;
        let duration = step.duration.max(0.0);
        let span = (duration / per_frame + 0.5)
            .floor()
            .clamp(TOOL_MIN, TOOL_MAX);
        push(cursor, cursor + span, duration, &mut real);
        tools.push(ToolSpan {
            start: cursor,
            end: cursor + span,
            span,
        });
        cursor += span;
    }
    push(
        cursor,
        cursor + ANSWER_GAP,
        ANSWER_GAP * per_frame,
        &mut real,
    );
    let answer_start = cursor + ANSWER_GAP;
    let answer_chunks = chunks_of(&run.answer, seed, "answer");
    let answer_end = answer_start + last_arrival(&answer_chunks);
    push(
        answer_start,
        answer_end,
        (answer_end - answer_start) * per_frame,
        &mut real,
    );
    Timeline {
        plan_start,
        think_end,
        think_seconds,
        tools,
        answer_start,
        run_end: answer_end,
        real_total: real,
        segments,
        plan_chunks,
        answer_chunks,
    }
}

pub fn elapsed_at(time: f64, timeline: &Timeline) -> f64 {
    if time.is_nan() || time <= RUN_START {
        return 0.0;
    }
    for segment in &timeline.segments {
        if segment.to > segment.from && time < segment.to {
            let share = (time - segment.from) / (segment.to - segment.from);
            return share.mul_add(segment.real, segment.real_from);
        }
    }
    timeline.real_total
}

#[derive(Debug, Clone)]
pub struct Chip {
    pub label: String,
    pub icon: &'static str,
}

#[derive(Debug, Clone)]
pub struct RowLayout {
    pub y: f64,
    pub height: f64,
    pub divider: bool,
    pub icon: &'static str,
    pub name: String,
    pub detail: String,
    pub duration: String,
    pub chips: Vec<Chip>,
    pub more: usize,
}

#[derive(Debug, Clone)]
pub struct PlanItemLayout {
    pub y: f64,
    pub raw: usize,
    pub lines: Vec<TextLine>,
}

#[derive(Debug, Clone)]
pub struct PlanLayout {
    pub y: f64,
    pub items: Vec<PlanItemLayout>,
}

#[derive(Debug, Clone)]
pub struct ToolsLayout {
    pub y: f64,
    pub rows: Vec<RowLayout>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Heading(u8),
    Paragraph,
    Item,
    Code,
}

#[derive(Debug, Clone)]
pub struct BlockLayout {
    pub kind: BlockKind,
    pub index: usize,
    pub y: f64,
    pub raw: usize,
    pub end: usize,
    pub mark: Units,
    pub mark_width: f64,
    pub open: usize,
    pub lang: Units,
    pub lang_raw: usize,
    pub lines: Vec<TextLine>,
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub bubble_y: f64,
    pub bubble_height: f64,
    pub bubble_lines: Vec<String>,
    pub status_y: f64,
    pub plan: Option<PlanLayout>,
    pub tools: Option<ToolsLayout>,
    pub blocks: Vec<BlockLayout>,
}

fn is_domain(label: &str) -> bool {
    let (host, path) = label
        .split_once('/')
        .map_or((label, None), |(h, p)| (h, Some(p)));
    let parts: Vec<&str> = host.split('.').collect();
    let label_ok =
        |p: &&str| !p.is_empty() && p.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    parts.len() >= 2
        && parts.iter().all(label_ok)
        && path.is_none_or(|p| !p.chars().any(char::is_whitespace))
}

fn chip_width(label: &str) -> f64 {
    7.0 + 12.0 + 5.0 + estimate_width(&units(label), CHIP_SIZE, false) + 9.0 + 2.0
}

fn more_width(count: usize) -> f64 {
    9.0 + estimate_width(&units(&format!("+{count}")), CHIP_SIZE, false) + 9.0 + 2.0
}

fn fit_chips(sources: &[String]) -> (Vec<Chip>, usize) {
    let mut chips = Vec::new();
    let mut used = 0.0;
    for (index, label) in sources.iter().enumerate() {
        if chips.len() >= CHIP_MAX {
            break;
        }
        let width = chip_width(label) + if chips.is_empty() { 0.0 } else { CHIP_GAP };
        let rest = sources.len() - index - 1;
        let reserve = if rest > 0 {
            CHIP_GAP + more_width(rest)
        } else {
            0.0
        };
        if used + width + reserve > CHIP_ROOM {
            break;
        }
        chips.push(Chip {
            label: label.clone(),
            icon: if is_domain(label) { "globe" } else { "file" },
        });
        used += width;
    }
    let more = sources.len() - chips.len();
    (chips, more)
}

const fn block_gap(previous: Option<BlockKind>, next: BlockKind) -> f64 {
    match (previous, next) {
        (None, _) => 0.0,
        (Some(BlockKind::Item), BlockKind::Item) => ITEM_GAP,
        (Some(BlockKind::Code), _) | (_, BlockKind::Code) => CODE_GAP,
        (Some(BlockKind::Heading(_)), _) => HEADING_GAP,
        _ => BLOCK_GAP,
    }
}

pub const fn heading_spec(level: u8) -> (f64, f64) {
    if level == 1 {
        (22.0, 32.0)
    } else {
        (20.0, 30.0)
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "the film lays the conversation out top to bottom in one pass, kept together so the port reads against it"
)]
pub fn layout_of(run: &Run) -> Layout {
    let mut y = PAD_TOP;
    let prompt = wrap_inline(&plain_inline(&run.prompt, 0), BUBBLE_SIZE, BUBBLE_WIDTH);
    let bubble_lines: Vec<String> = prompt
        .iter()
        .map(|line| line.iter().map(|p| text_of(&p.text)).collect())
        .collect();
    let bubble_y = y;
    let bubble_height = float(bubble_lines.len().max(1)).mul_add(BUBBLE_LINE, 2.0 * BUBBLE_PAD_Y);
    y += bubble_height + BUBBLE_GAP;
    let status_y = y;
    y += STATUS_H;
    let mut plan = None;
    if !run.plan.is_empty() {
        y += STATUS_GAP;
        let mut items = Vec::new();
        let mut cursor = 0.0;
        let mut raw = 0;
        for (index, text) in run.plan.iter().enumerate() {
            if index > 0 {
                cursor += PLAN_GAP;
            }
            let wrapped = wrap_inline(&plain_inline(text, raw), PLAN_SIZE, PLAN_WIDTH);
            let lines = text_lines(wrapped, PLAN_LINE, raw);
            let count = float(lines.len());
            items.push(PlanItemLayout {
                y: cursor,
                raw,
                lines,
            });
            cursor = count.mul_add(PLAN_LINE, cursor);
            raw += text.len() + 1;
        }
        plan = Some(PlanLayout { y, items });
        y += cursor;
    }
    let mut tools = None;
    if !run.steps.is_empty() {
        y += PLAN_AFTER;
        let mut rows = Vec::new();
        let mut cursor = 0.0;
        for (index, step) in run.steps.iter().enumerate() {
            let (chips, more) = fit_chips(&step.sources);
            let height = ROW_H
                + if index > 0 { 1.0 } else { 0.0 }
                + if step.sources.is_empty() {
                    0.0
                } else {
                    CHIPS_H
                };
            let icon = step.icon.unwrap_or(if step.sources.is_empty() {
                "terminal"
            } else {
                "book"
            });
            rows.push(RowLayout {
                y: cursor,
                height,
                divider: index > 0,
                icon,
                name: step.name.clone(),
                detail: step.detail.clone(),
                duration: format_seconds(step.duration),
                chips,
                more,
            });
            cursor += height;
        }
        tools = Some(ToolsLayout { y, rows });
        y += cursor + 2.0;
    }
    let parsed = parse_markdown(&run.answer);
    if !parsed.is_empty() {
        y += if tools.is_some() {
            TOOLS_AFTER
        } else {
            PLAN_AFTER
        };
    }
    let mut blocks = Vec::new();
    let mut previous = None;
    for (index, block) in parsed.into_iter().enumerate() {
        let kind = match &block {
            Block::Heading { level, .. } => BlockKind::Heading(*level),
            Block::Paragraph { .. } => BlockKind::Paragraph,
            Block::Item { .. } => BlockKind::Item,
            Block::Code { .. } => BlockKind::Code,
        };
        y += block_gap(previous, kind);
        previous = Some(kind);
        let empty = BlockLayout {
            kind,
            index,
            y,
            raw: 0,
            end: 0,
            mark: Vec::new(),
            mark_width: 0.0,
            open: 0,
            lang: Vec::new(),
            lang_raw: 0,
            lines: Vec::new(),
        };
        match block {
            Block::Heading {
                level,
                raw,
                body,
                end,
                inline,
            } => {
                let (size, line) = heading_spec(level);
                let lines = text_lines(wrap_inline(&inline, size, MEASURE), line, body);
                let mark = run.answer[raw..body].to_vec();
                let height = float(lines.len()) * line;
                blocks.push(BlockLayout {
                    raw,
                    end,
                    mark_width: round3(estimate_width(&mark, BODY_SIZE, false)),
                    mark,
                    lines,
                    ..empty
                });
                y += height;
            }
            Block::Code {
                raw,
                open,
                lang,
                lang_raw,
                lines,
                ..
            } => {
                let lines: Vec<TextLine> = lines
                    .iter()
                    .enumerate()
                    .map(|(row, (text, line_raw))| TextLine {
                        y: float(row) * CODE_LINE,
                        height: CODE_LINE,
                        raw: *line_raw,
                        pieces: code_pieces(text, *line_raw),
                    })
                    .collect();
                let height =
                    float(lines.len()).mul_add(CODE_LINE, 2.0f64.mul_add(CODE_PAD, CODE_HEAD));
                blocks.push(BlockLayout {
                    raw,
                    open,
                    lang,
                    lang_raw,
                    lines,
                    ..empty
                });
                y += height;
            }
            Block::Paragraph { raw, inline, .. } => {
                let lines = text_lines(wrap_inline(&inline, BODY_SIZE, MEASURE), BODY_LINE, raw);
                let height = float(lines.len()) * BODY_LINE;
                blocks.push(BlockLayout {
                    raw,
                    lines,
                    ..empty
                });
                y += height;
            }
            Block::Item {
                raw, body, inline, ..
            } => {
                let lines = text_lines(
                    wrap_inline(&inline, BODY_SIZE, MEASURE - ITEM_INDENT),
                    BODY_LINE,
                    body,
                );
                let height = float(lines.len()) * BODY_LINE;
                blocks.push(BlockLayout {
                    raw,
                    lines,
                    ..empty
                });
                y += height;
            }
        }
    }
    Layout {
        bubble_y,
        bubble_height,
        bubble_lines,
        status_y,
        plan,
        tools,
        blocks,
    }
}

pub fn block_height(block: &BlockLayout) -> f64 {
    match block.kind {
        BlockKind::Code => {
            float(block.lines.len()).mul_add(CODE_LINE, 2.0f64.mul_add(CODE_PAD, CODE_HEAD))
        }
        _ => block.lines.iter().map(|l| l.height).sum(),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollStep {
    at: f64,
    delta: f64,
}

pub fn scroll_steps(layout: &Layout, timeline: &Timeline, viewport: f64) -> Vec<ScrollStep> {
    let mut events: Vec<(f64, f64)> = vec![
        (BUBBLE_AT, layout.bubble_y + layout.bubble_height),
        (RUN_START, layout.status_y + STATUS_H),
    ];
    if let Some(plan) = &layout.plan {
        for item in &plan.items {
            for line in &item.lines {
                let at = arrival(&timeline.plan_chunks, line.raw);
                events.push((
                    timeline.plan_start + at,
                    plan.y + item.y + line.y + line.height,
                ));
            }
        }
    }
    if let Some(tools) = &layout.tools {
        for (index, row) in tools.rows.iter().enumerate() {
            events.push((
                timeline.tools[index].start,
                tools.y + row.y + row.height + 2.0,
            ));
        }
    }
    let arrive =
        |offset: isize| timeline.answer_start + arrival_signed(&timeline.answer_chunks, offset);
    for block in &layout.blocks {
        if block.kind == BlockKind::Code {
            let head = 2.0f64.mul_add(CODE_PAD, block.y + CODE_HEAD);
            events.push((arrive(isize::try_from(block.open).unwrap_or(0) - 1), head));
            for line in &block.lines {
                events.push((
                    arrive(isize::try_from(line.raw).unwrap_or(0)),
                    head + line.y + line.height,
                ));
            }
            continue;
        }
        for (row, line) in block.lines.iter().enumerate() {
            let start = if row == 0 { block.raw } else { line.raw };
            events.push((
                arrive(isize::try_from(start).unwrap_or(0)),
                block.y + line.y + line.height,
            ));
        }
    }
    events.retain(|(at, _)| at.is_finite());
    events.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut steps = Vec::new();
    let mut target = 0.0_f64;
    for (at, bottom) in events {
        let next = target.max(bottom + SAFE - viewport);
        if next > target {
            steps.push(ScrollStep {
                at,
                delta: round3(next - target),
            });
        }
        target = next;
    }
    steps
}

pub fn scroll_at(time: f64, steps: &[ScrollStep]) -> f64 {
    let mut total = 0.0;
    for step in steps {
        let since = time - step.at + SCROLL_LEAD;
        if since <= 0.0 {
            break;
        }
        total = f64::mul_add(step.delta, eased(since / SCROLL), total);
    }
    total
}

#[derive(Debug, Clone, Copy)]
pub struct Spinner {
    pub angle: f64,
    pub arc: f64,
    pub done: f64,
    pub check: f64,
}

pub fn spinner_at(time: f64, start: f64, end: f64) -> Spinner {
    let spinning = (time - start).clamp(0.0, (end - start).max(0.0));
    let since = time - end;
    let settle = progress(since / RESOLVE);
    let coast = SPIN * RESOLVE * (settle - settle * settle / 2.0);
    Spinner {
        angle: round3(spinning.mul_add(SPIN, coast) - 90.0),
        arc: round3((1.0 - ARC).mul_add(expo(since / RESOLVE), ARC)),
        done: round3(eased(since / RESOLVE)),
        check: round3(ease_out((since - CHECK_FROM) / (CHECK_TO - CHECK_FROM))),
    }
}

pub fn shimmer_at(time: f64, length: usize) -> Vec<f64> {
    let cycle = (time - RUN_START) / SHIMMER_PERIOD;
    let phase = cycle - cycle.floor();
    let center = phase.mul_add(2.0f64.mul_add(SHIMMER_BAND, float(length)), -SHIMMER_BAND);
    (0..length)
        .map(|index| {
            let distance = (float(index) + 0.5 - center).abs() / SHIMMER_BAND;
            let band = if distance < 1.0 {
                distance.mul_add(-distance, 1.0).powi(2)
            } else {
                0.0
            };
            round3((1.0 - SHIMMER_BASE).mul_add(band, SHIMMER_BASE))
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct Fragment {
    pub text: String,
    pub opacity: f64,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub kind: PieceKind,
    pub fragments: Vec<Fragment>,
    pub opacity: f64,
    pub scale: f64,
}

#[derive(Debug, Clone)]
pub struct LineState {
    pub y: f64,
    pub height: f64,
    pub spans: Vec<Span>,
}

fn fade_of(since: f64) -> f64 {
    round3(ease_out((since + 1.0) / FADE))
}

pub fn spans_of(pieces: &[Piece], chunks: &[Chunk], local: f64, revealed: usize) -> Vec<Span> {
    let mut spans = Vec::new();
    for piece in pieces {
        if piece.kind == PieceKind::Cite {
            if revealed < piece.end {
                continue;
            }
            let since = local - arrival(chunks, piece.end - 1);
            spans.push(Span {
                kind: PieceKind::Cite,
                fragments: vec![Fragment {
                    text: text_of(&piece.text),
                    opacity: 1.0,
                }],
                opacity: round3(ease_out((since + 1.0) / 4.0)),
                scale: round3(0.4f64.mul_add(back_out((since + 1.0) / POP, CITE_BACK), 0.6)),
            });
            continue;
        }
        let visible = revealed.saturating_sub(piece.raw).min(piece.text.len());
        if visible == 0 {
            continue;
        }
        let mut fragments: Vec<Fragment> = Vec::new();
        let mut offset = 0;
        while offset < visible {
            let chunk = chunks.get(chunk_index_at(chunks, piece.raw + offset));
            let stop = chunk.map_or(visible, |c| visible.min(c.end.saturating_sub(piece.raw)));
            let opacity = chunk.map_or(1.0, |c| fade_of(local - c.at));
            let text = text_of(&piece.text[offset..stop.max(offset)]);
            match fragments.last_mut() {
                Some(last) if last.opacity >= 1.0 && opacity >= 1.0 => last.text.push_str(&text),
                _ => fragments.push(Fragment { text, opacity }),
            }
            offset = stop.max(offset + 1);
        }
        spans.push(Span {
            kind: piece.kind,
            fragments,
            opacity: 1.0,
            scale: 1.0,
        });
    }
    spans
}

#[derive(Debug, Clone)]
pub struct BlockState {
    pub kind: BlockKind,
    pub y: f64,
    pub styled: bool,
    pub morph: f64,
    pub mark: String,
    pub mark_width: f64,
    pub dot: f64,
    pub dot_opacity: f64,
    pub height: f64,
    pub opacity: f64,
    pub lang: String,
    pub lines: Vec<LineState>,
}

fn line_states(
    block: &BlockLayout,
    from: usize,
    chunks: &[Chunk],
    local: f64,
    revealed: usize,
) -> Vec<LineState> {
    let mut lines = Vec::new();
    for (row, line) in block.lines.iter().enumerate() {
        if revealed <= if row == 0 { from } else { line.raw } {
            break;
        }
        lines.push(LineState {
            y: line.y,
            height: line.height,
            spans: spans_of(&line.pieces, chunks, local, revealed),
        });
    }
    lines
}

pub fn block_state(
    block: &BlockLayout,
    text_len: usize,
    chunks: &[Chunk],
    local: f64,
    revealed: usize,
) -> Option<BlockState> {
    let base = BlockState {
        kind: block.kind,
        y: block.y,
        styled: false,
        morph: 0.0,
        mark: String::new(),
        mark_width: block.mark_width,
        dot: 0.0,
        dot_opacity: 0.0,
        height: 0.0,
        opacity: 1.0,
        lang: String::new(),
        lines: Vec::new(),
    };
    if block.kind == BlockKind::Code {
        if revealed < block.open {
            return None;
        }
        let since = local - arrival(chunks, block.open - 1);
        let mut height = expo((since + 1.0) / CODE_OPEN) * 2.0f64.mul_add(CODE_PAD, CODE_HEAD);
        let mut lines = Vec::new();
        for line in &block.lines {
            if revealed <= line.raw {
                break;
            }
            let grown = local - arrival(chunks, line.raw);
            height = expo((grown + 1.0) / LINE_GROW).mul_add(CODE_LINE, height);
            lines.push(LineState {
                y: line.y,
                height: line.height,
                spans: spans_of(&line.pieces, chunks, local, revealed),
            });
        }
        let shown = revealed
            .saturating_sub(block.lang_raw)
            .min(block.lang.len());
        return Some(BlockState {
            height: round3(height),
            opacity: round3(ease_out((since + 1.0) / 4.0)),
            lang: text_of(&block.lang[..shown]),
            lines,
            ..base
        });
    }
    if revealed <= block.raw {
        return None;
    }
    let lines = line_states(block, block.raw, chunks, local, revealed);
    match block.kind {
        BlockKind::Heading(_) => {
            let complete = revealed > block.end || (block.end >= text_len && revealed >= block.end);
            let complete_at = arrival(chunks, block.end.min(text_len.saturating_sub(1)));
            let morph = if complete {
                round3(expo((local - complete_at) / MORPH))
            } else {
                0.0
            };
            let shown = revealed.saturating_sub(block.raw).min(block.mark.len());
            Some(BlockState {
                styled: complete,
                morph,
                mark: text_of(&block.mark[..shown]),
                lines,
                ..base
            })
        }
        BlockKind::Item => {
            let since = local - arrival(chunks, block.raw);
            Some(BlockState {
                dot: round3(back_out((since + 1.0) / POP, DOT_BACK)),
                dot_opacity: round3(ease_out((since + 1.0) / 3.0)),
                lines,
                ..base
            })
        }
        _ => Some(BlockState { lines, ..base }),
    }
}

#[derive(Debug, Clone)]
pub struct PlanItemState {
    pub index: usize,
    pub y: f64,
    pub number_opacity: f64,
    pub lines: Vec<LineState>,
}

#[derive(Debug, Clone)]
pub struct PlanState {
    pub y: f64,
    pub rule: f64,
    pub items: Vec<PlanItemState>,
}

pub fn plan_state(plan: &PlanLayout, chunks: &[Chunk], local: f64) -> PlanState {
    let revealed = reveal(chunks, local);
    let mut items = Vec::new();
    let mut rule = 0.0_f64;
    for (index, item) in plan.items.iter().enumerate() {
        if revealed <= item.raw {
            break;
        }
        let mut lines = Vec::new();
        for (row, line) in item.lines.iter().enumerate() {
            if revealed <= if row == 0 { item.raw } else { line.raw } {
                break;
            }
            let since = local - arrival(chunks, line.raw);
            let grown = line.height * expo((since + 1.0) / LINE_GROW);
            rule = rule.max(item.y + line.y + grown);
            lines.push(LineState {
                y: line.y,
                height: line.height,
                spans: spans_of(&line.pieces, chunks, local, revealed),
            });
        }
        items.push(PlanItemState {
            index,
            y: item.y,
            number_opacity: fade_of(local - arrival(chunks, item.raw)),
            lines,
        });
    }
    PlanState {
        y: plan.y,
        rule: round3(rule),
        items,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pop {
    pub visible: bool,
    pub opacity: f64,
    pub scale: f64,
}

fn chip_pops(count: usize, since: f64, span: f64) -> Vec<Pop> {
    let spread = CHIP_SPAN.mul_add(span, -CHIP_LEAD).max(0.0);
    let step = if count > 1 {
        spread / float(count - 1)
    } else {
        0.0
    };
    (0..count)
        .map(|index| {
            let age = float(index).mul_add(-step, since - CHIP_LEAD);
            Pop {
                visible: age >= 0.0,
                opacity: round3(ease_out((age + 1.0) / 4.0)),
                scale: round3(0.2f64.mul_add(back_out((age + 1.0) / POP, CHIP_BACK), 0.8)),
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct RowState {
    pub y: f64,
    pub height: f64,
    pub layout: RowLayout,
    pub content: f64,
    pub spinner: Spinner,
    pub label: f64,
    pub chips: Vec<(Chip, Pop)>,
    pub more: Option<(usize, Pop)>,
    pub sources: bool,
}

#[derive(Debug, Clone)]
pub struct ToolsState {
    pub y: f64,
    pub height: f64,
    pub rows: Vec<RowState>,
}

pub fn tools_state(tools: &ToolsLayout, timeline: &Timeline, time: f64) -> ToolsState {
    let mut height = 2.0;
    let mut rows = Vec::new();
    for (index, row) in tools.rows.iter().enumerate() {
        let span = timeline.tools[index];
        if time < span.start {
            continue;
        }
        let since = time - span.start;
        let open = expo(since / ROW_IN);
        height = f64::mul_add(row.height, open, height);
        let pops = chip_pops(
            row.chips.len() + usize::from(row.more > 0),
            since,
            span.span,
        );
        let more = (row.more > 0)
            .then(|| pops.get(row.chips.len()).map(|p| (row.more, *p)))
            .flatten();
        rows.push(RowState {
            y: row.y,
            height: round3(row.height * open),
            content: round3(ease_out((since - 1.0) / ROW_FADE)),
            spinner: spinner_at(time, span.start, span.end),
            label: round3(ease_out((time - span.end - LABEL_DELAY) / LABEL_FADE)),
            chips: row
                .chips
                .iter()
                .cloned()
                .zip(pops.iter().copied())
                .collect(),
            more,
            sources: row.chips.len() + row.more > 0,
            layout: row.clone(),
        });
    }
    ToolsState {
        y: tools.y,
        height: round3(height),
        rows,
    }
}

pub fn think_label_shown(think_seconds: f64) -> f64 {
    (think_seconds + 0.5).floor().max(1.0)
}

pub fn panel_lift(time: f64) -> f64 {
    round3(PANEL_RISE * (1.0 - expo(time / PANEL_IN)))
}

pub fn truncate_len(value: f64) -> usize {
    truncate(value)
}
