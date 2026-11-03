use opsis_types::terminal::TerminalTheme;
use opsis_vt::{Colour, Palette, Rgb};

const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];
const GREY_START: u8 = 8;
const GREY_STEP: u8 = 10;

pub const fn pack(rgb: Rgb) -> u32 {
    u32::from_be_bytes([0, rgb.r, rgb.g, rgb.b])
}

pub const fn unpack(colour: u32) -> Rgb {
    let [_, r, g, b] = colour.to_be_bytes();
    Rgb { r, g, b }
}

pub fn parse_hex(text: &str) -> Option<u32> {
    let hex = text.trim().strip_prefix('#')?;
    let value = match hex.len() {
        6 => u32::from_str_radix(hex, 16).ok()?,
        3 => {
            let short = u32::from_str_radix(hex, 16).ok()?;
            let (r, g, b) = ((short >> 8) & 0xf, (short >> 4) & 0xf, short & 0xf);
            (r * 17) << 16 | (g * 17) << 8 | (b * 17)
        }
        _ => return None,
    };
    Some(value)
}

pub fn mix(a: u32, b: u32, share_of_a: u16) -> u32 {
    let (a, b) = (unpack(a), unpack(b));
    let blend = |x: u8, y: u8| {
        let total = u16::from(x) * share_of_a + u16::from(y) * (256 - share_of_a);
        u8::try_from(total >> 8).unwrap_or(u8::MAX)
    };
    pack(Rgb {
        r: blend(a.r, b.r),
        g: blend(a.g, b.g),
        b: blend(a.b, b.b),
    })
}

fn xterm(index: u8) -> u32 {
    match index {
        16..=231 => {
            let i = usize::from(index - 16);
            pack(Rgb {
                r: CUBE[i / 36],
                g: CUBE[(i / 6) % 6],
                b: CUBE[i % 6],
            })
        }
        232..=255 => {
            let level = GREY_START + (index - 232) * GREY_STEP;
            pack(Rgb {
                r: level,
                g: level,
                b: level,
            })
        }
        _ => 0,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeColours {
    pub fg: u32,
    pub bg: u32,
    pub cursor: u32,
    pub palette: [u32; 256],
}

impl ThemeColours {
    pub fn of(theme: &TerminalTheme) -> Self {
        let fg = parse_hex(&theme.fg).unwrap_or(0x00ff_ffff);
        let bg = parse_hex(&theme.bg).unwrap_or(0);
        let mut palette = [0; 256];
        for (i, slot) in palette.iter_mut().enumerate() {
            let index = u8::try_from(i).unwrap_or(u8::MAX);
            *slot = theme
                .palette
                .get(i)
                .and_then(|c| parse_hex(c))
                .unwrap_or_else(|| xterm(index));
        }
        Self {
            fg,
            bg,
            cursor: parse_hex(&theme.cursor).unwrap_or(fg),
            palette,
        }
    }

    #[must_use]
    pub fn with_program(&self, changed: Option<&Palette>) -> Self {
        let Some(changed) = changed else {
            return self.clone();
        };
        let mut out = self.clone();
        for (slot, colour) in out.palette.iter_mut().zip(changed.colours.iter()) {
            if let Some(rgb) = colour {
                *slot = pack(*rgb);
            }
        }
        if let Some(rgb) = changed.foreground {
            out.fg = pack(rgb);
        }
        if let Some(rgb) = changed.background {
            out.bg = pack(rgb);
        }
        if let Some(rgb) = changed.cursor {
            out.cursor = pack(rgb);
        }
        out
    }

    pub fn resolve(&self, colour: Colour, default: u32) -> u32 {
        match colour {
            Colour::Default => default,
            Colour::Palette(i) => self.palette[usize::from(i)],
            Colour::Rgb(rgb) => pack(rgb),
        }
    }
}
