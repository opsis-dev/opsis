use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SettingsError {
    #[error("{label} {problem}")]
    Field { label: String, problem: String },
    #[error("{0}")]
    Rule(Cow<'static, str>),
    #[error("{0}")]
    Unreadable(String),
}

impl SettingsError {
    pub const fn rule(message: &'static str) -> Self {
        Self::Rule(Cow::Borrowed(message))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CompileError {
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error("Start is at or after the end of the clip, which is {clip} long.")]
    StartAfterEnd { clip: String },
    #[error(
        "Repeated {times} times the clip would run {length}, longer than the {limit} a film can be."
    )]
    TooLong {
        times: u32,
        length: String,
        limit: String,
    },
    #[error("Connect a clip to “{0}”.")]
    MissingClip(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PathError {
    #[error("Line {0} of the path isn't two numbers")]
    Unreadable(usize),
    #[error("Add points to the path")]
    Empty,
    #[error("Add a point away from where the drag starts")]
    Still,
}

#[derive(Debug, thiserror::Error)]
pub enum WallTimeError {
    #[error("“{0}” isn't a date and time.")]
    NotATime(String),
    #[error("“{0}” isn't a time zone.")]
    NotAZone(String),
    #[error(transparent)]
    Placing(#[from] jiff::Error),
}
