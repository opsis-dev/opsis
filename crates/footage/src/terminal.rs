use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use opsis_terminal::cast::read_path;
use opsis_terminal::frames::write;
use opsis_terminal::prepare::{Mask, PrepareInput, prepare};
use opsis_types::terminal::TerminalTheme;

use crate::FootageError;

#[derive(Debug, Clone)]
pub struct TerminalFootage {
    pub cast: PathBuf,
    pub theme: TerminalTheme,
    pub masks: Vec<Mask>,
    pub fps: u32,
}

#[derive(Debug)]
pub struct TerminalState {
    source: TerminalFootage,
    made: OnceLock<Result<Arc<Vec<u8>>, String>>,
}

impl TerminalState {
    pub const fn new(source: TerminalFootage) -> Self {
        Self {
            source,
            made: OnceLock::new(),
        }
    }

    fn make(source: &TerminalFootage) -> Result<Arc<Vec<u8>>, String> {
        let cast = read_path(&source.cast).map_err(|e| e.to_string())?;
        let frames = prepare(&PrepareInput {
            cast: &cast,
            fps: source.fps,
            theme: &source.theme,
            masks: &source.masks,
        })
        .map_err(|e| e.to_string())?;
        write(&frames).map(Arc::new).map_err(|e| e.to_string())
    }

    pub fn frames(&self) -> Result<Arc<Vec<u8>>, FootageError> {
        self.made
            .get_or_init(|| Self::make(&self.source))
            .clone()
            .map_err(FootageError::Terminal)
    }
}
