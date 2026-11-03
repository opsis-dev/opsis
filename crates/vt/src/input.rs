#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct KeyMods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl KeyMods {
    pub const NONE: Self = Self {
        shift: false,
        ctrl: false,
        alt: false,
        super_key: false,
    };

    pub(crate) fn bits(self) -> u16 {
        u16::from(self.shift)
            | u16::from(self.ctrl) << 1
            | u16::from(self.alt) << 2
            | u16::from(self.super_key) << 3
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum KeyAction {
    #[default]
    Press,
    Release,
}

impl KeyAction {
    pub(crate) const fn code(self) -> u8 {
        match self {
            Self::Press => 0,
            Self::Release => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseAction {
    Press,
    Release,
    Motion,
}

impl MouseAction {
    pub(crate) const fn code(self) -> u8 {
        match self {
            Self::Press => 0,
            Self::Release => 1,
            Self::Motion => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    None,
    Left,
    Right,
    Middle,
    WheelUp,
    WheelDown,
}

impl MouseButton {
    pub(crate) const fn code(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Left => 1,
            Self::Right => 2,
            Self::Middle => 3,
            Self::WheelUp => 4,
            Self::WheelDown => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct CellPoint {
    pub col: u16,
    pub row: u16,
}
