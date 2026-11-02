use crate::ffi::EventOut;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    Prompt,
    Input,
    Command,
    Done { code: Option<i32> },
    HoldBegin,
    HoldEnd,
    Title,
    Pwd,
    Bell,
}

impl Event {
    pub(crate) const fn from_out(out: EventOut) -> Option<Self> {
        Some(match out.kind {
            1 => Self::Prompt,
            2 => Self::Input,
            3 => Self::Command,
            4 => Self::Done {
                code: if out.has_code != 0 {
                    Some(out.code)
                } else {
                    None
                },
            },
            5 => Self::HoldBegin,
            6 => Self::HoldEnd,
            7 => Self::Title,
            8 => Self::Pwd,
            9 => Self::Bell,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modes {
    pub cursor_keys: bool,
    pub keypad_keys: bool,
    pub bracketed_paste: bool,
}

impl Modes {
    pub(crate) const fn from_bits(bits: u32) -> Self {
        Self {
            cursor_keys: bits & 1 != 0,
            keypad_keys: bits & 2 != 0,
            bracketed_paste: bits & 4 != 0,
        }
    }
}
