use serde::Serialize;

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(transparent)]
pub struct CommandError(pub String);

impl<E: std::fmt::Display> From<E> for CommandError {
    fn from(error: E) -> Self {
        Self(error.to_string())
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
