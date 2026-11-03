use std::collections::HashMap;

pub const MAGIC: &[u8; 8] = b"OPSTERM2";

pub mod flag {
    pub const BOLD: u16 = 1;
    pub const ITALIC: u16 = 1 << 1;
    pub const STRIKETHROUGH: u16 = 1 << 2;
    pub const OVERLINE: u16 = 1 << 3;
    pub const BLINK: u16 = 1 << 4;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Redaction {
    #[default]
    None,
    Solid,
    Blur,
    Pixels,
}

impl Redaction {
    pub const fn code(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Solid => 1,
            Self::Blur => 2,
            Self::Pixels => 3,
        }
    }

    pub const fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            0 => Self::None,
            1 => Self::Solid,
            2 => Self::Blur,
            3 => Self::Pixels,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Run {
    pub col: u16,
    pub cells: u16,
    pub text: String,
    pub fg: u32,
    pub bg: u32,
    pub underline_colour: u32,
    pub flags: u16,
    pub underline: u8,
    pub redaction: Redaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FrameCursor {
    pub x: u16,
    pub y: u16,
    pub visible: bool,
    pub shape: u8,
    pub blink: bool,
    pub colour: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameRecord {
    pub cols: u16,
    pub rows: u16,
    pub row_indices: Vec<u32>,
    pub cursor: FrameCursor,
    pub bg: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Frames {
    pub cols: u16,
    pub rows: u16,
    pub fps: u32,
    pub row_table: Vec<Vec<Run>>,
    pub frames: Vec<FrameRecord>,
}

#[derive(Debug, Default)]
pub struct RowTable {
    rows: Vec<Vec<Run>>,
    index: HashMap<Vec<Run>, u32>,
}

impl RowTable {
    pub fn intern(&mut self, runs: Vec<Run>) -> u32 {
        if let Some(&i) = self.index.get(&runs) {
            return i;
        }
        let i = u32::try_from(self.rows.len()).unwrap_or(u32::MAX);
        self.index.insert(runs.clone(), i);
        self.rows.push(runs);
        i
    }

    pub fn into_rows(self) -> Vec<Vec<Run>> {
        self.rows
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum FramesError {
    #[error("This isn't a terminal frames file")]
    Magic,
    #[error("The terminal frames file ends early")]
    Truncated,
    #[error("The terminal frames file has a run with text outside its text block")]
    Text,
    #[error("The terminal frames file names a row it doesn't have")]
    Row,
    #[error("The terminal frames file is too large")]
    TooLarge,
}

fn size(n: usize) -> Result<u32, FramesError> {
    u32::try_from(n).map_err(|_| FramesError::TooLarge)
}

struct Writer(Vec<u8>);

impl Writer {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
}

pub fn write(frames: &Frames) -> Result<Vec<u8>, FramesError> {
    let mut text = String::new();
    let mut offsets: HashMap<&str, u32> = HashMap::new();
    for run in frames.row_table.iter().flatten() {
        if !offsets.contains_key(run.text.as_str()) {
            offsets.insert(&run.text, size(text.len())?);
            text.push_str(&run.text);
        }
    }
    let mut w = Writer(Vec::with_capacity(text.len() + 64));
    w.0.extend_from_slice(MAGIC);
    w.u16(frames.cols);
    w.u16(frames.rows);
    w.u32(frames.fps);
    w.u32(size(frames.frames.len())?);
    w.u32(size(frames.row_table.len())?);
    w.u32(size(text.len())?);
    w.0.extend_from_slice(text.as_bytes());
    for row in &frames.row_table {
        w.u16(u16::try_from(row.len()).map_err(|_| FramesError::TooLarge)?);
        for run in row {
            w.u16(run.col);
            w.u16(run.cells);
            w.u32(offsets.get(run.text.as_str()).copied().unwrap_or(0));
            w.u16(u16::try_from(run.text.len()).map_err(|_| FramesError::TooLarge)?);
            w.u32(run.fg);
            w.u32(run.bg);
            w.u32(run.underline_colour);
            w.u16(run.flags);
            w.u8(run.underline);
            w.u8(run.redaction.code());
        }
    }
    for frame in &frames.frames {
        w.u16(frame.cols);
        w.u16(frame.rows);
        for &i in &frame.row_indices {
            w.u32(i);
        }
        let c = frame.cursor;
        w.u16(c.x);
        w.u16(c.y);
        w.u8(u8::from(c.visible));
        w.u8(c.shape);
        w.u8(u8::from(c.blink));
        w.u32(c.colour);
        w.u32(frame.bg);
    }
    Ok(w.0)
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], FramesError> {
        let end = self.at.checked_add(n).ok_or(FramesError::Truncated)?;
        let slice = self.bytes.get(self.at..end).ok_or(FramesError::Truncated)?;
        self.at = end;
        Ok(slice)
    }
    fn u8(&mut self) -> Result<u8, FramesError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, FramesError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    fn u32(&mut self) -> Result<u32, FramesError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn count(&mut self) -> Result<usize, FramesError> {
        usize::try_from(self.u32()?).map_err(|_| FramesError::TooLarge)
    }
}

pub fn read(bytes: &[u8]) -> Result<Frames, FramesError> {
    let mut r = Reader { bytes, at: 0 };
    if r.take(MAGIC.len())? != MAGIC {
        return Err(FramesError::Magic);
    }
    let cols = r.u16()?;
    let rows = r.u16()?;
    let fps = r.u32()?;
    let frame_count = r.count()?;
    let row_count = r.count()?;
    let text_len = r.count()?;
    let text = std::str::from_utf8(r.take(text_len)?).map_err(|_| FramesError::Text)?;
    let mut row_table = Vec::with_capacity(row_count.min(bytes.len()));
    for _ in 0..row_count {
        let runs = usize::from(r.u16()?);
        let mut row = Vec::with_capacity(runs);
        for _ in 0..runs {
            let col = r.u16()?;
            let cells = r.u16()?;
            let offset = r.count()?;
            let len = usize::from(r.u16()?);
            let run_text = text.get(offset..offset + len).ok_or(FramesError::Text)?;
            row.push(Run {
                col,
                cells,
                text: run_text.to_owned(),
                fg: r.u32()?,
                bg: r.u32()?,
                underline_colour: r.u32()?,
                flags: r.u16()?,
                underline: r.u8()?,
                redaction: Redaction::from_code(r.u8()?).ok_or(FramesError::Truncated)?,
            });
        }
        row_table.push(row);
    }
    let mut frames = Vec::with_capacity(frame_count.min(bytes.len()));
    for _ in 0..frame_count {
        let frame_cols = r.u16()?;
        let frame_rows = r.u16()?;
        let mut row_indices = Vec::with_capacity(usize::from(frame_rows));
        for _ in 0..frame_rows {
            let i = r.u32()?;
            if usize::try_from(i).map_err(|_| FramesError::Row)? >= row_table.len() {
                return Err(FramesError::Row);
            }
            row_indices.push(i);
        }
        let cursor = FrameCursor {
            x: r.u16()?,
            y: r.u16()?,
            visible: r.u8()? != 0,
            shape: r.u8()?,
            blink: r.u8()? != 0,
            colour: r.u32()?,
        };
        frames.push(FrameRecord {
            cols: frame_cols,
            rows: frame_rows,
            row_indices,
            cursor,
            bg: r.u32()?,
        });
    }
    if r.at != bytes.len() {
        return Err(FramesError::Truncated);
    }
    Ok(Frames {
        cols,
        rows,
        fps,
        row_table,
        frames,
    })
}
