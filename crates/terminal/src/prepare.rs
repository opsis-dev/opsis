use std::time::Duration;

use opsis_types::numbers::truncate;
use opsis_types::terminal::TerminalTheme;
use opsis_vt::{CellWidth, CursorShape, Event as VtEvent, Frame, Terminal, Underline, VtError};
use regex::Regex;

use crate::cast::{Cast, EventData};
use crate::colours::{ThemeColours, mix};
use crate::frames::{FrameCursor, FrameRecord, Frames, Redaction, RowTable, Run, flag};
use crate::timeline::{SCROLL_PREFIX, Timeline};

const NANOS_PER_SECOND: u128 = 1_000_000_000;
const HOLD_LIMIT: Duration = Duration::from_secs(1);
const SCROLLBACK: u32 = 10_000;
const FAINT_SHARE: u16 = 128;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaskArea {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskLook {
    Solid(u32),
    Blur,
    Pixels,
}

impl MaskLook {
    const fn redaction(self) -> Redaction {
        match self {
            Self::Solid(_) => Redaction::Solid,
            Self::Blur => Redaction::Blur,
            Self::Pixels => Redaction::Pixels,
        }
    }

    const fn colour(self) -> u32 {
        match self {
            Self::Solid(colour) => colour,
            Self::Blur | Self::Pixels => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mask {
    pub areas: Vec<MaskArea>,
    pub patterns: Vec<Regex>,
    pub from: Duration,
    pub until: Option<Duration>,
    pub look: MaskLook,
}

impl Mask {
    fn active(&self, at: Duration) -> bool {
        at >= self.from && self.until.is_none_or(|until| at < until)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PrepareInput<'a> {
    pub cast: &'a Cast,
    pub fps: u32,
    pub theme: &'a TerminalTheme,
    pub masks: &'a [Mask],
}

#[derive(Debug, thiserror::Error)]
pub enum PrepareError {
    #[error(transparent)]
    Terminal(#[from] VtError),
    #[error("A film needs at least one frame a second")]
    Fps,
}

pub fn frame_count(length: Duration, fps: u32) -> usize {
    let frames = (length.as_nanos() * u128::from(fps)).div_ceil(NANOS_PER_SECOND) + 1;
    usize::try_from(frames).unwrap_or(usize::MAX)
}

pub fn frame_time(index: usize, fps: u32) -> Duration {
    let nanos =
        u128::try_from(index).unwrap_or(u128::MAX) * NANOS_PER_SECOND / u128::from(fps.max(1));
    Duration::from_nanos(u64::try_from(nanos).unwrap_or(u64::MAX))
}

struct Replay<'a> {
    cast: &'a Cast,
    terminal: Terminal,
    next: usize,
    hold_since: Option<Duration>,
}

impl Replay<'_> {
    fn advance(&mut self, raw: Duration) -> Result<(), VtError> {
        while let Some(event) = self.cast.events.get(self.next).filter(|e| e.time <= raw) {
            self.next += 1;
            match &event.data {
                EventData::Output(text) => self.terminal.feed(text.as_bytes()),
                EventData::Resize(cols, rows) => self.terminal.resize(*cols, *rows)?,
                EventData::Marker(label) => {
                    if let Some(lines) = label
                        .strip_prefix(SCROLL_PREFIX)
                        .and_then(|l| l.parse().ok())
                    {
                        self.terminal.scroll(lines);
                    }
                }
                EventData::Input(_) | EventData::Exit(_) | EventData::Other(..) => {}
            }
            self.terminal.take_replies();
            for vt_event in self.terminal.take_events() {
                match vt_event {
                    VtEvent::HoldBegin => self.hold_since = Some(event.time),
                    VtEvent::HoldEnd => self.hold_since = None,
                    _ => {}
                }
            }
        }
        if let Some(since) = self.hold_since
            && raw.saturating_sub(since) >= HOLD_LIMIT
        {
            self.terminal.release_hold();
            self.hold_since = None;
        }
        Ok(())
    }
}

pub fn prepare(input: &PrepareInput<'_>) -> Result<Frames, PrepareError> {
    if input.fps == 0 {
        return Err(PrepareError::Fps);
    }
    let header = &input.cast.header;
    let timeline = Timeline::from_events(&input.cast.events);
    let theme = ThemeColours::of(input.theme);
    let mut replay = Replay {
        cast: input.cast,
        terminal: Terminal::new(header.term_cols, header.term_rows, SCROLLBACK)?,
        next: 0,
        hold_since: None,
    };
    let mut table = RowTable::default();
    let mut frames = Vec::new();
    for index in 0..frame_count(timeline.length(), input.fps) {
        let visible = frame_time(index, input.fps);
        replay.advance(timeline.raw(visible))?;
        let program = replay.terminal.palette();
        let colours = theme.with_program(program.as_ref());
        let frame = replay.terminal.frame()?;
        let masks: Vec<&Mask> = input.masks.iter().filter(|m| m.active(visible)).collect();
        frames.push(record(frame, &colours, &masks, &mut table));
    }
    Ok(Frames {
        cols: header.term_cols,
        rows: header.term_rows,
        fps: input.fps,
        row_table: table.into_rows(),
        frames,
    })
}

pub fn snapshot(terminal: &mut Terminal, theme: &TerminalTheme) -> Result<Frames, VtError> {
    let program = terminal.palette();
    let colours = ThemeColours::of(theme).with_program(program.as_ref());
    let frame = terminal.frame()?;
    let mut table = RowTable::default();
    let record = record(frame, &colours, &[], &mut table);
    Ok(Frames {
        cols: record.cols,
        rows: record.rows,
        fps: 1,
        row_table: table.into_rows(),
        frames: vec![record],
    })
}

const fn shape_code(shape: CursorShape) -> u8 {
    match shape {
        CursorShape::Bar => 0,
        CursorShape::Block => 1,
        CursorShape::Underline => 2,
        CursorShape::BlockHollow => 3,
    }
}

const fn underline_code(underline: Underline) -> u8 {
    match underline {
        Underline::None => 0,
        Underline::Single => 1,
        Underline::Double => 2,
        Underline::Curly => 3,
        Underline::Dotted => 4,
        Underline::Dashed => 5,
    }
}

fn record(
    frame: Frame<'_>,
    colours: &ThemeColours,
    masks: &[&Mask],
    table: &mut RowTable,
) -> FrameRecord {
    let grid = mask_grid(frame, masks);
    let cols = frame.cols();
    let row_indices = frame
        .row_iter()
        .enumerate()
        .map(|(y, row)| {
            let start = y * usize::from(cols);
            let masked = &grid[start..start + usize::from(cols)];
            table.intern(row_runs(&row, colours, frame.reverse(), masked))
        })
        .collect();
    let cursor = frame.cursor();
    FrameRecord {
        cols,
        rows: frame.rows(),
        row_indices,
        cursor: FrameCursor {
            x: cursor.x,
            y: cursor.y,
            visible: cursor.visible,
            shape: shape_code(cursor.shape),
            blink: cursor.blink,
            colour: colours.cursor,
        },
        bg: colours.bg,
    }
}

struct Styled {
    fg: u32,
    bg: u32,
    underline_colour: u32,
    flags: u16,
    underline: u8,
}

fn styled(cell: &opsis_vt::Cell<'_>, colours: &ThemeColours, reverse: bool) -> Styled {
    let f = cell.flags();
    let mut fg = colours.resolve(cell.fg(), colours.fg);
    let mut bg = colours.resolve(cell.bg(), colours.bg);
    if f.inverse != reverse {
        std::mem::swap(&mut fg, &mut bg);
    }
    if f.faint {
        fg = mix(fg, bg, FAINT_SHARE);
    }
    if f.invisible {
        fg = bg;
    }
    let mut flags = 0;
    for (on, bit) in [
        (f.bold, flag::BOLD),
        (f.italic, flag::ITALIC),
        (f.strikethrough, flag::STRIKETHROUGH),
        (f.overline, flag::OVERLINE),
        (f.blink, flag::BLINK),
    ] {
        if on {
            flags |= bit;
        }
    }
    Styled {
        fg,
        bg,
        underline_colour: colours.resolve(cell.underline_colour(), fg),
        flags,
        underline: underline_code(cell.underline()),
    }
}

fn simple(text: &str, width: CellWidth) -> bool {
    width == CellWidth::Narrow && text.len() == 1 && text.bytes().all(|b| (0x20..0x7f).contains(&b))
}

fn plain_blank(run: &Run, screen_bg: u32) -> bool {
    run.redaction == Redaction::None
        && run.bg == screen_bg
        && run.underline == 0
        && run.flags & (flag::STRIKETHROUGH | flag::OVERLINE) == 0
        && run.text.chars().all(|c| c == ' ')
}

fn row_runs(
    row: &opsis_vt::Row<'_>,
    colours: &ThemeColours,
    reverse: bool,
    masked: &[Option<MaskLook>],
) -> Vec<Run> {
    let mut runs: Vec<Run> = Vec::new();
    let mut open = false;
    for (x, cell) in row.cells().enumerate() {
        let width = cell.width();
        if width == CellWidth::Spacer {
            continue;
        }
        let col = u16::try_from(x).unwrap_or(u16::MAX);
        let cells = if width == CellWidth::Wide { 2 } else { 1 };
        let text = if cell.is_empty() { " " } else { cell.text() };
        let style = styled(&cell, colours, reverse);
        if let Some(look) = masked.get(x).copied().flatten() {
            let redaction = look.redaction();
            if let Some(last) = runs.last_mut().filter(|r| open && r.redaction == redaction) {
                last.cells += cells;
                continue;
            }
            runs.push(Run {
                col,
                cells,
                text: String::new(),
                fg: look.colour(),
                bg: style.bg,
                underline_colour: 0,
                flags: 0,
                underline: 0,
                redaction,
            });
            open = true;
            continue;
        }
        let run = Run {
            col,
            cells,
            text: text.to_owned(),
            fg: style.fg,
            bg: style.bg,
            underline_colour: style.underline_colour,
            flags: style.flags,
            underline: style.underline,
            redaction: Redaction::None,
        };
        let mergeable = simple(text, width);
        if let Some(last) = runs.last_mut().filter(|r| {
            open && mergeable
                && r.redaction == Redaction::None
                && r.fg == run.fg
                && r.bg == run.bg
                && r.underline_colour == run.underline_colour
                && r.flags == run.flags
                && r.underline == run.underline
        }) {
            last.text.push_str(text);
            last.cells += 1;
            continue;
        }
        runs.push(run);
        open = mergeable;
    }
    runs.retain(|r| !plain_blank(r, colours.bg));
    runs
}

fn mask_grid(frame: Frame<'_>, masks: &[&Mask]) -> Vec<Option<MaskLook>> {
    let cols = usize::from(frame.cols());
    let rows = usize::from(frame.rows());
    let mut grid = vec![None; cols * rows];
    if masks.is_empty() {
        return grid;
    }
    for mask in masks {
        for area in &mask.areas {
            let span = |start: f64, length: f64, cells: usize| {
                let n = cells.max(1);
                let size = f64::from(u32::try_from(n).unwrap_or(u32::MAX));
                let from: usize = truncate((start * size).floor().clamp(0.0, size));
                let to: usize = truncate(((start + length) * size).ceil().clamp(0.0, size));
                from..to.min(n)
            };
            for y in span(area.y, area.height, rows) {
                for x in span(area.x, area.width, cols) {
                    grid[y * cols + x] = Some(mask.look);
                }
            }
        }
        if !mask.patterns.is_empty() {
            mask_text(frame, &mask.patterns, mask.look, &mut grid);
        }
    }
    grid
}

fn mask_text(frame: Frame<'_>, patterns: &[Regex], look: MaskLook, grid: &mut [Option<MaskLook>]) {
    let cols = usize::from(frame.cols());
    let mut line = String::new();
    let mut places: Vec<(std::ops::Range<usize>, usize)> = Vec::new();
    let mut flush = |line: &mut String, places: &mut Vec<(std::ops::Range<usize>, usize)>| {
        for pattern in patterns {
            for found in pattern.find_iter(line) {
                for (bytes, cell) in places.iter() {
                    if bytes.start < found.end() && found.start() < bytes.end {
                        grid[*cell] = Some(look);
                    }
                }
            }
        }
        line.clear();
        places.clear();
    };
    for (y, row) in frame.row_iter().enumerate() {
        for (x, cell) in row.cells().enumerate() {
            if cell.width() == CellWidth::Spacer {
                continue;
            }
            let text = if cell.is_empty() { " " } else { cell.text() };
            let start = line.len();
            line.push_str(text);
            places.push((start..line.len(), y * cols + x));
            if cell.width() == CellWidth::Wide {
                places.push((start..line.len(), y * cols + x + 1));
            }
        }
        if !row.wrapped() {
            flush(&mut line, &mut places);
        }
    }
    flush(&mut line, &mut places);
}
