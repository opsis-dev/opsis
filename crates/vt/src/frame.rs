use std::ops::Range;

use crate::cell::{Cell, CellWidth, Colour, Rgb};
use crate::ffi::{CellOut, FrameOut, RowOut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorShape {
    Bar,
    Block,
    Underline,
    BlockHollow,
}

impl CursorShape {
    const fn from_code(code: u8) -> Self {
        match code {
            0 => Self::Bar,
            2 => Self::Underline,
            3 => Self::BlockHollow,
            _ => Self::Block,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cursor {
    pub x: u16,
    pub y: u16,
    pub visible: bool,
    pub shape: CursorShape,
    pub blink: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RowId {
    pub serial: u64,
    pub y: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    pub colours: [Option<Rgb>; 256],
    pub foreground: Option<Rgb>,
    pub background: Option<Rgb>,
    pub cursor: Option<Rgb>,
}

impl Palette {
    pub(crate) fn from_tagged(values: &[u32]) -> Option<Self> {
        let mut colours = [None; 256];
        for (colour, value) in colours.iter_mut().zip(values) {
            *colour = Colour::rgb_from_tagged(*value);
        }
        let at = |index: usize| values.get(index).copied().and_then(Colour::rgb_from_tagged);
        let palette = Self {
            colours,
            foreground: at(256),
            background: at(257),
            cursor: at(258),
        };
        let changed = palette.colours.iter().any(Option::is_some)
            || palette.foreground.is_some()
            || palette.background.is_some()
            || palette.cursor.is_some();
        changed.then_some(palette)
    }
}

#[derive(Debug)]
pub struct Buffers {
    pub frame: FrameOut,
    pub cells: Vec<CellOut>,
    pub rows: Vec<RowOut>,
    pub graphemes: Vec<u32>,
    pub text: String,
    pub spans: Vec<Range<usize>>,
}

impl Buffers {
    pub fn new() -> Self {
        Self {
            frame: FrameOut::default(),
            cells: Vec::new(),
            rows: Vec::new(),
            graphemes: Vec::new(),
            text: String::new(),
            spans: Vec::new(),
        }
    }

    pub fn build_text(&mut self) {
        self.text.clear();
        self.spans.clear();
        let written = usize::from(self.frame.cols) * usize::from(self.frame.rows);
        for cell in &self.cells[..written] {
            let start = self.text.len();
            if cell.cp != 0 {
                self.text
                    .push(char::from_u32(cell.cp).unwrap_or(char::REPLACEMENT_CHARACTER));
                let first = index(cell.extra_start);
                let extras = self
                    .graphemes
                    .get(first..first + usize::from(cell.extra_len))
                    .unwrap_or_default();
                for &codepoint in extras {
                    self.text
                        .push(char::from_u32(codepoint).unwrap_or(char::REPLACEMENT_CHARACTER));
                }
            }
            self.spans.push(start..self.text.len());
        }
    }
}

pub fn index(value: u32) -> usize {
    usize::try_from(value).expect("opsis-vt builds for 64-bit targets only")
}

#[derive(Debug, Clone, Copy)]
pub struct Frame<'a> {
    pub(crate) buffers: &'a Buffers,
}

impl<'a> Frame<'a> {
    pub const fn cols(&self) -> u16 {
        self.buffers.frame.cols
    }

    pub const fn rows(&self) -> u16 {
        self.buffers.frame.rows
    }

    pub const fn cursor(&self) -> Cursor {
        let frame = &self.buffers.frame;
        Cursor {
            x: frame.cursor_x,
            y: frame.cursor_y,
            visible: frame.cursor_visible != 0,
            shape: CursorShape::from_code(frame.cursor_shape),
            blink: frame.cursor_blink != 0,
        }
    }

    pub const fn held(&self) -> bool {
        self.buffers.frame.held != 0
    }

    pub const fn palette_changed(&self) -> bool {
        self.buffers.frame.palette_changed != 0
    }

    pub const fn reverse(&self) -> bool {
        self.buffers.frame.reverse != 0
    }

    pub fn default_fg(&self) -> Option<Rgb> {
        Colour::rgb_from_tagged(self.buffers.frame.default_fg)
    }

    pub fn default_bg(&self) -> Option<Rgb> {
        Colour::rgb_from_tagged(self.buffers.frame.default_bg)
    }

    pub fn row(&self, y: u16) -> Option<Row<'a>> {
        if y >= self.rows() {
            return None;
        }
        let cols = usize::from(self.cols());
        let start = usize::from(y) * cols;
        Some(Row {
            out: &self.buffers.rows[usize::from(y)],
            cells: &self.buffers.cells[start..start + cols],
            spans: &self.buffers.spans[start..start + cols],
            text: &self.buffers.text,
        })
    }

    pub fn row_iter(&self) -> impl Iterator<Item = Row<'a>> + use<'a> {
        let frame = *self;
        (0..frame.rows()).filter_map(move |y| frame.row(y))
    }

    pub fn text(&self) -> String {
        self.row_iter()
            .map(|row| row.text())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Row<'a> {
    out: &'a RowOut,
    cells: &'a [CellOut],
    spans: &'a [Range<usize>],
    text: &'a str,
}

impl<'a> Row<'a> {
    pub const fn id(&self) -> RowId {
        RowId {
            serial: self.out.serial,
            y: self.out.y,
        }
    }

    pub const fn dirty(&self) -> bool {
        self.out.dirty != 0
    }

    pub const fn wrapped(&self) -> bool {
        self.out.wrapped != 0
    }

    pub const fn len(&self) -> usize {
        self.cells.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn cell(&self, x: u16) -> Option<Cell<'a>> {
        let x = usize::from(x);
        let out = self.cells.get(x)?;
        let span = self.spans.get(x)?.clone();
        Some(Cell {
            out,
            text: &self.text[span],
        })
    }

    pub fn cells(&self) -> impl ExactSizeIterator<Item = Cell<'a>> + use<'a> {
        let text = self.text;
        self.cells
            .iter()
            .zip(self.spans)
            .map(move |(out, span)| Cell {
                out,
                text: &text[span.clone()],
            })
    }

    pub fn text(&self) -> String {
        let mut line = String::new();
        for cell in self.cells() {
            match cell.width() {
                CellWidth::Spacer => {}
                _ if cell.is_empty() => line.push(' '),
                _ => line.push_str(cell.text()),
            }
        }
        line.truncate(line.trim_end().len());
        line
    }
}
