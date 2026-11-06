use std::sync::Mutex;

use opsis_types::shell::{MenuCommand, PlaybackCall, Playhead, ReplayTab, Selection, SharedState};

use crate::Session;
use crate::error::CommandResult;
use crate::events::{
    Emit, MAIN_WINDOW, MenuCommandFired, PlaybackCalled, PlayheadShared, ReplayRequested,
    SelectionShared,
};

#[derive(Debug, Default)]
pub struct Shared(Mutex<SharedState>);

impl Shared {
    fn state(&self) -> std::sync::MutexGuard<'_, SharedState> {
        self.0.lock().expect("shared state")
    }
}

pub fn forget_project(app: &Session) {
    app.state::<Shared>().state().selection = Selection::default();
    if let Err(error) = (SelectionShared {
        from: String::new(),
        selection: Selection::default(),
    })
    .emit(app)
    {
        log::warn!("clearing the selection: {error}");
    }
}
pub fn shared_state(app: &Session) -> SharedState {
    let shared = app.state::<Shared>();
    shared.state().clone()
}
pub fn share_selection(app: &Session, window: &str, selection: Selection) -> CommandResult<()> {
    let shared = app.state::<Shared>();
    shared.state().selection = selection.clone();
    Ok(SelectionShared {
        from: window.to_owned(),
        selection,
    }
    .emit(app)?)
}
pub fn share_playhead(app: &Session, window: &str, playhead: Playhead) -> CommandResult<()> {
    let shared = app.state::<Shared>();
    shared.state().playhead = Some(playhead.clone());
    Ok(PlayheadShared {
        from: window.to_owned(),
        playhead,
    }
    .emit(app)?)
}
pub fn control_playback(app: &Session, call: PlaybackCall) -> CommandResult<()> {
    Ok(PlaybackCalled(call).emit_to(app, MAIN_WINDOW)?)
}

fn raise_editor(app: &Session) {
    app.platform().raise_editor();
}
pub fn show_replay(app: &Session, node_id: Option<String>, tab: ReplayTab) -> CommandResult<()> {
    raise_editor(app);
    Ok(ReplayRequested { node_id, tab }.emit_to(app, MAIN_WINDOW)?)
}
pub fn send_menu_to_editor(app: &Session, command: MenuCommand) -> CommandResult<()> {
    raise_editor(app);
    Ok(MenuCommandFired(command).emit_to(app, MAIN_WINDOW)?)
}
