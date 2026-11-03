use opsis_compile::nodes::browser::chord_keys;
use opsis_types::terminal::{CellTarget, PointerButton};
use opsis_vt::{CellPoint, CellWidth, Frame, KeyMods, MouseButton};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InputError {
    #[error("A terminal can't receive the Meta key")]
    Meta,
    #[error("Pick the keys to press")]
    Empty,
    #[error("“{0}” isn't on the screen")]
    NotOnScreen(String),
    #[error("Column {col}, row {row} is outside the {cols} × {rows} terminal")]
    Outside {
        col: u16,
        row: u16,
        cols: u16,
        rows: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord {
    pub key: String,
    pub mods: KeyMods,
}

pub fn parse_chord(text: &str) -> Result<Chord, InputError> {
    let keys = chord_keys(text);
    let (key, held) = keys.split_last().ok_or(InputError::Empty)?;
    if key.is_empty() {
        return Err(InputError::Empty);
    }
    let mut mods = KeyMods::NONE;
    for held in held {
        match held.as_str() {
            "Shift" => mods.shift = true,
            "Control" | "Ctrl" | "ControlOrMeta" => mods.ctrl = true,
            "Alt" | "Option" => mods.alt = true,
            "Meta" | "Command" | "Cmd" | "Super" => return Err(InputError::Meta),
            _ => {}
        }
    }
    if matches!(key.as_str(), "Meta" | "Command" | "Cmd" | "Super") {
        return Err(InputError::Meta);
    }
    let key = if key == " " { "Space" } else { key.as_str() };
    Ok(Chord {
        key: key.to_owned(),
        mods,
    })
}

pub const fn mouse_button(button: PointerButton) -> MouseButton {
    match button {
        PointerButton::Left => MouseButton::Left,
        PointerButton::Right => MouseButton::Right,
        PointerButton::Middle => MouseButton::Middle,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Found {
    pub row: u16,
    pub from: u16,
    pub to: u16,
}

impl Found {
    pub const fn middle(self) -> CellPoint {
        CellPoint {
            col: self.from + (self.to - self.from) / 2,
            row: self.row,
        }
    }
}

fn row_cells(row: &opsis_vt::Row<'_>) -> (String, Vec<(usize, u16)>) {
    let mut text = String::new();
    let mut places = Vec::new();
    for (x, cell) in row.cells().enumerate() {
        if cell.width() == CellWidth::Spacer {
            continue;
        }
        let col = u16::try_from(x).unwrap_or(u16::MAX);
        places.push((text.len(), col));
        text.push_str(if cell.is_empty() { " " } else { cell.text() });
    }
    (text, places)
}

fn col_at(places: &[(usize, u16)], byte: usize) -> u16 {
    places
        .iter()
        .take_while(|(start, _)| *start <= byte)
        .last()
        .map_or(0, |(_, col)| *col)
}

pub fn find_on_screen(
    frame: Frame<'_>,
    matches: &dyn Fn(&str) -> Option<(usize, usize)>,
) -> Option<Found> {
    frame.row_iter().enumerate().find_map(|(y, row)| {
        let (text, places) = row_cells(&row);
        let (start, end) = matches(&text)?;
        let from = col_at(&places, start);
        let to = col_at(&places, end.saturating_sub(1).max(start));
        Some(Found {
            row: u16::try_from(y).unwrap_or(u16::MAX),
            from,
            to,
        })
    })
}

pub fn find_text(frame: Frame<'_>, needle: &str) -> Option<Found> {
    if needle.is_empty() {
        return None;
    }
    find_on_screen(frame, &|line| {
        line.find(needle).map(|at| (at, at + needle.len()))
    })
}

pub fn target_cell(frame: Frame<'_>, target: &CellTarget) -> Result<CellPoint, InputError> {
    match target {
        CellTarget::Cell { col, row } => {
            if *col >= frame.cols() || *row >= frame.rows() {
                return Err(InputError::Outside {
                    col: *col,
                    row: *row,
                    cols: frame.cols(),
                    rows: frame.rows(),
                });
            }
            Ok(CellPoint {
                col: *col,
                row: *row,
            })
        }
        CellTarget::Text { text } => find_text(frame, text)
            .map(Found::middle)
            .ok_or_else(|| InputError::NotOnScreen(text.clone())),
    }
}
