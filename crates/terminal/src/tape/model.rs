use opsis_types::terminal::LookIn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Enter,
    Space,
    Backspace,
    Delete,
    Insert,
    Escape,
    Tab,
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
    ScrollUp,
    ScrollDown,
}

impl Key {
    pub const ALL: [Self; 17] = [
        Self::Enter,
        Self::Space,
        Self::Backspace,
        Self::Delete,
        Self::Insert,
        Self::Escape,
        Self::Tab,
        Self::Up,
        Self::Down,
        Self::Left,
        Self::Right,
        Self::PageUp,
        Self::PageDown,
        Self::Home,
        Self::End,
        Self::ScrollUp,
        Self::ScrollDown,
    ];

    pub const fn word(self) -> &'static str {
        match self {
            Self::Enter => "Enter",
            Self::Space => "Space",
            Self::Backspace => "Backspace",
            Self::Delete => "Delete",
            Self::Insert => "Insert",
            Self::Escape => "Escape",
            Self::Tab => "Tab",
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::PageUp => "PageUp",
            Self::PageDown => "PageDown",
            Self::Home => "Home",
            Self::End => "End",
            Self::ScrollUp => "ScrollUp",
            Self::ScrollDown => "ScrollDown",
        }
    }

    pub fn from_word(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.word() == word)
    }

    pub const fn chord(self) -> &'static str {
        match self {
            Self::Up => "ArrowUp",
            Self::Down => "ArrowDown",
            Self::Left => "ArrowLeft",
            Self::Right => "ArrowRight",
            other => other.word(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Type {
        speed_ms: Option<u32>,
        text: String,
    },
    Key {
        key: Key,
        speed_ms: Option<u32>,
        repeat: u32,
    },
    Ctrl {
        keys: Vec<String>,
    },
    Alt {
        key: String,
    },
    Shift {
        key: String,
    },
    Sleep {
        ms: u32,
    },
    Hide,
    Show,
    Wait {
        scope: LookIn,
        timeout_ms: Option<u32>,
        pattern: Option<String>,
    },
    Require {
        program: String,
    },
    Screenshot {
        path: String,
    },
    Copy {
        text: String,
    },
    Paste,
    Env {
        name: String,
        value: String,
    },
    Source {
        path: String,
    },
    Output {
        path: String,
    },
    Set {
        name: String,
        value: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tape {
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Line {line}, column {column}: {message}")]
pub struct TapeError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}
