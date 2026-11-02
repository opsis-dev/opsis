use crate::ffi::CellOut;

const TAG_PALETTE: u32 = 1;
const TAG_RGB: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Colour {
    #[default]
    Default,
    Palette(u8),
    Rgb(Rgb),
}

impl Colour {
    pub(crate) fn from_tagged(value: u32) -> Self {
        let [tag, r, g, b] = value.to_be_bytes();
        match u32::from(tag) {
            TAG_PALETTE => Self::Palette(b),
            TAG_RGB => Self::Rgb(Rgb { r, g, b }),
            _ => Self::Default,
        }
    }

    pub(crate) fn rgb_from_tagged(value: u32) -> Option<Rgb> {
        match Self::from_tagged(value) {
            Self::Rgb(rgb) => Some(rgb),
            Self::Default | Self::Palette(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Flags {
    pub bold: bool,
    pub italic: bool,
    pub faint: bool,
    pub blink: bool,
    pub inverse: bool,
    pub invisible: bool,
    pub strikethrough: bool,
    pub overline: bool,
    pub hyperlink: bool,
}

impl Flags {
    fn from_bits(bits: u16) -> Self {
        let on = |bit: u16| bits & (1 << bit) != 0;
        Self {
            bold: on(0),
            italic: on(1),
            faint: on(2),
            blink: on(3),
            inverse: on(4),
            invisible: on(5),
            strikethrough: on(6),
            overline: on(7),
            hyperlink: on(8),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Underline {
    #[default]
    None,
    Single,
    Double,
    Curly,
    Dotted,
    Dashed,
}

impl Underline {
    const fn from_code(code: u8) -> Self {
        match code {
            1 => Self::Single,
            2 => Self::Double,
            3 => Self::Curly,
            4 => Self::Dotted,
            5 => Self::Dashed,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellWidth {
    Spacer,
    Narrow,
    Wide,
}

#[derive(Debug, Clone, Copy)]
pub struct Cell<'a> {
    pub(crate) out: &'a CellOut,
    pub(crate) text: &'a str,
}

impl<'a> Cell<'a> {
    pub const fn text(&self) -> &'a str {
        self.text
    }

    pub const fn is_empty(&self) -> bool {
        self.out.cp == 0
    }

    pub const fn width(&self) -> CellWidth {
        match self.out.width {
            0 => CellWidth::Spacer,
            2 => CellWidth::Wide,
            _ => CellWidth::Narrow,
        }
    }

    pub fn fg(&self) -> Colour {
        Colour::from_tagged(self.out.fg)
    }

    pub fn bg(&self) -> Colour {
        Colour::from_tagged(self.out.bg)
    }

    pub const fn underline(&self) -> Underline {
        Underline::from_code(self.out.underline)
    }

    pub fn underline_colour(&self) -> Colour {
        Colour::from_tagged(self.out.underline_colour)
    }

    pub fn flags(&self) -> Flags {
        Flags::from_bits(self.out.flags)
    }
}
