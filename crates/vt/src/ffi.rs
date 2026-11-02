pub const ABI_VERSION: u32 = 1;
pub const PALETTE_ENTRIES: usize = 259;

pub const STATUS_OK: u32 = 0;
pub const STATUS_CELLS_TOO_SMALL: u32 = 2;
pub const STATUS_ROWS_TOO_SMALL: u32 = 3;
pub const STATUS_GRAPHEMES_TOO_SMALL: u32 = 4;

#[repr(C)]
#[derive(Debug)]
pub struct RawVt {
    opaque: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CellOut {
    pub cp: u32,
    pub extra_start: u32,
    pub extra_len: u16,
    pub width: u8,
    pub underline: u8,
    pub fg: u32,
    pub bg: u32,
    pub underline_colour: u32,
    pub flags: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RowOut {
    pub serial: u64,
    pub y: u16,
    pub dirty: u8,
    pub wrapped: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FrameOut {
    pub default_fg: u32,
    pub default_bg: u32,
    pub cells_written: u32,
    pub graphemes_written: u32,
    pub cols: u16,
    pub rows: u16,
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub cursor_visible: u8,
    pub cursor_shape: u8,
    pub cursor_blink: u8,
    pub held: u8,
    pub palette_changed: u8,
    pub reverse: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EventOut {
    pub code: i32,
    pub kind: u8,
    pub has_code: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Sizes {
    pub cell: u32,
    pub row: u32,
    pub frame: u32,
    pub event: u32,
}

unsafe extern "C" {
    pub safe fn opsis_vt_abi() -> u32;
    pub safe fn opsis_vt_sizes(out: &mut Sizes);
    pub safe fn opsis_vt_new(cols: u16, rows: u16, scrollback_rows: u32) -> *mut RawVt;
    pub fn opsis_vt_free(vt: *mut RawVt);
    pub fn opsis_vt_feed(vt: *mut RawVt, ptr: *const u8, len: usize);
    pub fn opsis_vt_set_cell_size(vt: *mut RawVt, width: u32, height: u32);
    pub fn opsis_vt_resize(vt: *mut RawVt, cols: u16, rows: u16) -> u32;
    pub fn opsis_vt_scroll(vt: *mut RawVt, delta: i32);
    pub fn opsis_vt_modes(vt: *mut RawVt) -> u32;
    pub fn opsis_vt_pending_replies(vt: *mut RawVt) -> usize;
    pub fn opsis_vt_take_replies(vt: *mut RawVt, out: *mut u8, cap: usize) -> usize;
    pub fn opsis_vt_pending_events(vt: *mut RawVt) -> usize;
    pub fn opsis_vt_take_events(vt: *mut RawVt, out: *mut EventOut, cap: usize) -> usize;
    pub fn opsis_vt_title(vt: *mut RawVt, out: *mut u8, cap: usize) -> usize;
    pub fn opsis_vt_pwd(vt: *mut RawVt, out: *mut u8, cap: usize) -> usize;
    pub fn opsis_vt_release_hold(vt: *mut RawVt);
    pub fn opsis_vt_palette(vt: *mut RawVt, out: *mut u32);
    pub fn opsis_vt_frame(
        vt: *mut RawVt,
        frame: *mut FrameOut,
        cells: *mut CellOut,
        cells_cap: usize,
        rows: *mut RowOut,
        rows_cap: usize,
        graphemes: *mut u32,
        graphemes_cap: usize,
    ) -> u32;
}
