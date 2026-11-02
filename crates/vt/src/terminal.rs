use std::ptr::NonNull;

use crate::error::VtError;
use crate::event::{Event, Modes};
use crate::ffi::{self, CellOut, EventOut, PALETTE_ENTRIES, RawVt, RowOut, Sizes};
use crate::frame::{Buffers, Frame, Palette, index};

const FRAME_ATTEMPTS: usize = 4;

#[derive(Debug)]
pub struct Terminal {
    handle: NonNull<RawVt>,
    buffers: Buffers,
}

unsafe impl Send for Terminal {}

pub fn check_abi() -> Result<(), VtError> {
    let found = ffi::opsis_vt_abi();
    if found != ffi::ABI_VERSION {
        return Err(VtError::Abi {
            part: "version",
            expected: index(ffi::ABI_VERSION),
            found: index(found),
        });
    }
    let mut sizes = Sizes::default();
    ffi::opsis_vt_sizes(&mut sizes);
    let parts = [
        ("cell", size_of::<CellOut>(), sizes.cell),
        ("row", size_of::<RowOut>(), sizes.row),
        ("frame", size_of::<ffi::FrameOut>(), sizes.frame),
        ("event", size_of::<EventOut>(), sizes.event),
    ];
    for (part, expected, found) in parts {
        if expected != index(found) {
            return Err(VtError::Abi {
                part,
                expected,
                found: index(found),
            });
        }
    }
    Ok(())
}

impl Terminal {
    pub fn new(cols: u16, rows: u16, scrollback: u32) -> Result<Self, VtError> {
        check_abi()?;
        if cols == 0 || rows == 0 {
            return Err(VtError::Size { cols, rows });
        }
        let handle =
            NonNull::new(ffi::opsis_vt_new(cols, rows, scrollback)).ok_or(VtError::OutOfMemory)?;
        Ok(Self {
            handle,
            buffers: Buffers::new(),
        })
    }

    const fn raw(&self) -> *mut RawVt {
        self.handle.as_ptr()
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        unsafe { ffi::opsis_vt_feed(self.raw(), bytes.as_ptr(), bytes.len()) }
    }

    pub fn set_cell_size(&mut self, width: u32, height: u32) {
        unsafe { ffi::opsis_vt_set_cell_size(self.raw(), width, height) }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), VtError> {
        if cols == 0 || rows == 0 {
            return Err(VtError::Size { cols, rows });
        }
        match unsafe { ffi::opsis_vt_resize(self.raw(), cols, rows) } {
            ffi::STATUS_OK => Ok(()),
            _ => Err(VtError::OutOfMemory),
        }
    }

    pub fn scroll(&mut self, lines: i32) {
        unsafe { ffi::opsis_vt_scroll(self.raw(), lines) }
    }

    pub fn modes(&mut self) -> Modes {
        Modes::from_bits(unsafe { ffi::opsis_vt_modes(self.raw()) })
    }

    pub fn take_replies(&mut self) -> Vec<u8> {
        let raw = self.raw();
        let pending = unsafe { ffi::opsis_vt_pending_replies(raw) };
        let mut replies = vec![0; pending];
        let taken = unsafe { ffi::opsis_vt_take_replies(raw, replies.as_mut_ptr(), replies.len()) };
        replies.truncate(taken);
        replies
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        let raw = self.raw();
        let pending = unsafe { ffi::opsis_vt_pending_events(raw) };
        let mut events = vec![EventOut::default(); pending];
        let taken = unsafe { ffi::opsis_vt_take_events(raw, events.as_mut_ptr(), events.len()) };
        events.truncate(taken);
        events.into_iter().filter_map(Event::from_out).collect()
    }

    pub fn title(&mut self) -> String {
        self.text_of(ffi::opsis_vt_title)
    }

    pub fn pwd(&mut self) -> String {
        self.text_of(ffi::opsis_vt_pwd)
    }

    fn text_of(&self, read: unsafe extern "C" fn(*mut RawVt, *mut u8, usize) -> usize) -> String {
        let raw = self.raw();
        let length = unsafe { read(raw, std::ptr::null_mut(), 0) };
        let mut bytes = vec![0; length];
        let written = unsafe { read(raw, bytes.as_mut_ptr(), bytes.len()) };
        bytes.truncate(written.min(length));
        String::from_utf8_lossy(&bytes).into_owned()
    }

    pub fn release_hold(&mut self) {
        unsafe { ffi::opsis_vt_release_hold(self.raw()) }
    }

    pub fn palette(&mut self) -> Option<Palette> {
        let mut values = [0; PALETTE_ENTRIES];
        unsafe { ffi::opsis_vt_palette(self.raw(), values.as_mut_ptr()) };
        Palette::from_tagged(&values)
    }

    pub fn frame(&mut self) -> Result<Frame<'_>, VtError> {
        let raw = self.raw();
        let buffers = &mut self.buffers;
        for _ in 0..FRAME_ATTEMPTS {
            let status = unsafe {
                ffi::opsis_vt_frame(
                    raw,
                    &raw mut buffers.frame,
                    buffers.cells.as_mut_ptr(),
                    buffers.cells.len(),
                    buffers.rows.as_mut_ptr(),
                    buffers.rows.len(),
                    buffers.graphemes.as_mut_ptr(),
                    buffers.graphemes.len(),
                )
            };
            let cols = usize::from(buffers.frame.cols);
            let rows = usize::from(buffers.frame.rows);
            match status {
                ffi::STATUS_OK => {
                    buffers.build_text();
                    return Ok(Frame { buffers });
                }
                ffi::STATUS_ROWS_TOO_SMALL => buffers.rows.resize(rows, RowOut::default()),
                ffi::STATUS_CELLS_TOO_SMALL => {
                    buffers.cells.resize(cols * rows, CellOut::default());
                }
                ffi::STATUS_GRAPHEMES_TOO_SMALL => {
                    buffers
                        .graphemes
                        .resize(index(buffers.frame.graphemes_written), 0);
                }
                _ => return Err(VtError::OutOfMemory),
            }
        }
        Err(VtError::OutOfMemory)
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        unsafe { ffi::opsis_vt_free(self.raw()) }
    }
}
