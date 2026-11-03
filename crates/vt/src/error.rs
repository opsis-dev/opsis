#[derive(Debug, thiserror::Error)]
pub enum VtError {
    #[error("A terminal needs at least one column and one row, not {cols} × {rows}")]
    Size { cols: u16, rows: u16 },
    #[error("The terminal doesn't know the key “{0}”")]
    UnknownKey(String),
    #[error("The terminal engine ran out of memory")]
    OutOfMemory,
    #[error("The terminal engine's {part} layout is {found}, but Opsis expects {expected}")]
    Abi {
        part: &'static str,
        expected: usize,
        found: usize,
    },
}
