pub mod editing;
pub mod folder;
pub mod host;
pub mod io;
pub mod module;
pub mod output;
pub mod paths;
pub mod runtime;
pub mod sandbox;
pub mod syntax;
pub mod text;
pub mod themes;
pub mod values;

pub use host::{HostResult, ProjectInfo, RunEvents, RunNext, ScriptHost};
pub use runtime::{Ending, Script, ScriptError, run};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Failed(pub String);

pub fn failed(message: impl Into<String>) -> mlua::Error {
    mlua::Error::external(Failed(message.into()))
}
