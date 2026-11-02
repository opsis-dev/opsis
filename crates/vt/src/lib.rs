#![allow(
    unsafe_code,
    reason = "opsis-vt is the one crate that calls the terminal engine through its C interface"
)]
#![deny(unsafe_op_in_unsafe_fn)]

mod cell;
mod error;
mod event;
mod ffi;
mod frame;
mod terminal;

pub use cell::{Cell, CellWidth, Colour, Flags, Rgb, Underline};
pub use error::VtError;
pub use event::{Event, Modes};
pub use frame::{Cursor, CursorShape, Frame, Palette, Row, RowId};
pub use terminal::{Terminal, check_abi};
