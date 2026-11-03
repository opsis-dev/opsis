use std::path::Path;

use opsis_compile::terminal::default_theme;
use opsis_project::paths::ref_to_path;
use opsis_terminal::cast::read_path;
use opsis_terminal::frames::write;
use opsis_terminal::prepare::{PrepareInput, prepare};
use opsis_types::terminal::TerminalTheme;
use tauri::State;

use crate::error::{CommandError, CommandResult};
use crate::frames::Bytes;
use crate::state::AppState;

const CAST_THEME: &str = "Cast file";

#[tauri::command]
#[specta::specta]
pub async fn terminal_frames(
    state: State<'_, AppState>,
    project_dir: String,
    cast: String,
    fps: u32,
    theme: Option<TerminalTheme>,
) -> CommandResult<Bytes> {
    let file = ref_to_path(Path::new(&project_dir), &state.paths.public_dir, &cast)
        .map_err(|_| CommandError("A take can't come from the internet.".into()))?;
    let bytes = tokio::task::spawn_blocking(move || {
        let cast = read_path(&file).map_err(|e| e.to_string())?;
        let theme = theme
            .or_else(|| {
                cast.header
                    .term_theme
                    .as_ref()
                    .map(|t| t.to_theme(CAST_THEME))
            })
            .unwrap_or_else(default_theme);
        let frames = prepare(&PrepareInput {
            cast: &cast,
            fps,
            theme: &theme,
            masks: &[],
        })
        .map_err(|e| e.to_string())?;
        write(&frames).map_err(|e| e.to_string())
    })
    .await?
    .map_err(CommandError)?;
    Ok(Bytes(bytes))
}
