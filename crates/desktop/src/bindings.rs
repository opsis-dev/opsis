use opsis_compile::ts_export::frontend_types;
use opsis_types::ts_export::tidy_export;
use tauri_specta::{Builder, ErrorHandlingMode, collect_commands, collect_events};

use crate::{
    analysis, api, compile, document, engine, events, exit, file_server, fonts, gradients,
    page_log, projects, run, settings, shared, terminal_live, views, windows,
};

pub const BINDINGS_FILE: &str = "apps/frontend/src/generated/bindings.ts";

pub fn builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            projects::list_recent_projects,
            projects::project_locations,
            projects::choose_folder,
            projects::pick_project_dir,
            projects::open_project,
            document::document_state,
            document::edit_workflow,
            document::undo,
            document::redo,
            document::begin_gesture,
            document::end_gesture,
            document::edit_timeline,
            document::timeline_removal,
            document::save_project,
            document::resolve_disk_change,
            projects::project_folder,
            projects::create_project,
            projects::read_last_run,
            projects::import_assets,
            projects::clear_cache,
            projects::list_templates,
            settings::get_settings,
            settings::patch_settings,
            settings::view_state,
            settings::set_view_pref,
            settings::app_menu_sections,
            settings::get_environment,
            engine::install_engine,
            engine::choose_browser,
            engine::get_engine_status,
            settings::list_licences,
            settings::open_licence_file,
            settings::reveal_in_folder,
            settings::read_trace,
            windows::open_settings,
            windows::open_panel_window,
            windows::open_live_window,
            windows::focus_panel_window,
            windows::close_panel_window,
            windows::close_panel_windows,
            windows::list_panel_windows,
            windows::get_window_state,
            windows::set_title,
            windows::window_role,
            exit::report_drafts,
            exit::close_shown,
            exit::decide_close,
            exit::save_flushed,
            compile::set_compile_focus,
            compile::latest_compile,
            compile::editor_catalog,
            file_server::file_root,
            run::run_workflow,
            run::cancel_run,
            run::run_state,
            run::clear_run_log,
            run::watch_live,
            run::unwatch_live,
            run::live_window_watching,
            gradients::gradient_view,
            gradients::edit_gradient,
            fonts::list_fonts,
            fonts::font_choices,
            fonts::terminal_fonts,
            fonts::inspect_font_files,
            fonts::installed_font_families,
            fonts::inspect_installed_fonts,
            fonts::add_inspected_fonts,
            fonts::release_inspected_fonts,
            fonts::remove_font,
            fonts::font_catalog,
            fonts::download_font,
            fonts::use_font_in_project,
            analysis::analyze,
            views::pick_element,
            views::attach_view,
            views::view_input,
            views::view_resize,
            views::view_navigate,
            views::view_go,
            views::view_reload,
            views::view_switch_tab,
            views::view_answer_dialog,
            views::view_choose_files,
            views::view_close,
            terminal_live::attach_terminal,
            terminal_live::terminal_input,
            terminal_live::finish_terminal,
            page_log::page_log,
            shared::send_menu_to_editor,
            shared::shared_state,
            shared::share_selection,
            shared::share_playhead,
            shared::control_playback,
            shared::show_replay,
            api::api_status,
        ])
        .events(collect_events![
            events::DocumentChanged,
            events::DiskChangeAsked,
            events::ProjectReloaded,
            events::OpenRequested,
            events::ProjectFilesChanged,
            events::RunEventFired,
            events::EngineInstallProgress,
            events::EngineChanged,
            events::SettingsChanged,
            events::ViewPrefChanged,
            events::ApiStatusChanged,
            events::MenuCommandFired,
            events::PanelWindowsChanged,
            events::ConfirmClose,
            events::FlushSave,
            events::WindowStateChanged,
            events::SettingsPageShown,
            events::LiveNoted,
            events::LiveWindowWatching,
            events::Compiled,
            events::SelectionShared,
            events::PlayheadShared,
            events::PlaybackCalled,
            events::ReplayRequested,
            events::RunViewChanged,
            events::RunLogged,
            events::RunStateReset,
            views::ViewEventFired,
        ])
        .types(&frontend_types())
        .dangerously_cast_bigints_to_number()
        .error_handling(ErrorHandlingMode::Throw)
}

#[derive(Debug, thiserror::Error)]
pub enum BindingsError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Export(#[from] specta_typescript::Error),
}

pub fn bindings() -> Result<String, BindingsError> {
    let file = tempfile::NamedTempFile::new()?;
    builder().export(specta_typescript::Typescript::default(), file.path())?;
    let text = std::fs::read_to_string(file.path())?;
    Ok(tidy_export(&text))
}
