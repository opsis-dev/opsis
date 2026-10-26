use opsis_compile::ts_export::ui_types;
use opsis_types::ts_export::tidy_export;
use tauri_specta::{Builder, collect_commands, collect_events};

use crate::{
    analysis, api, compile, events, exit, file_server, fonts, page_log, projects, relay, run,
    settings, views, windows,
};

pub const BINDINGS_FILE: &str = "apps/ui/src/generated/bindings.ts";

pub fn builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            projects::list_recent_projects,
            projects::project_locations,
            projects::choose_folder,
            projects::pick_project_dir,
            projects::open_project,
            projects::project_folder,
            projects::create_project,
            projects::save_workflow,
            projects::watch_project,
            projects::watch_project_files,
            projects::read_last_run,
            projects::import_assets,
            projects::clear_cache,
            projects::list_templates,
            settings::get_settings,
            settings::patch_settings,
            settings::set_view_pref,
            settings::app_menu_sections,
            settings::get_environment,
            settings::install_engine,
            settings::list_licences,
            settings::open_licence_file,
            settings::reveal_in_folder,
            settings::read_trace,
            settings::list_secrets,
            windows::open_settings,
            windows::open_panel_window,
            windows::open_preview_window,
            windows::focus_panel_window,
            windows::close_panel_window,
            windows::close_panel_windows,
            windows::list_panel_windows,
            windows::get_window_state,
            windows::set_title,
            windows::window_role,
            exit::report_unsaved,
            exit::close_shown,
            exit::decide_close,
            exit::save_flushed,
            compile::compile,
            compile::latest_compile,
            compile::editor_catalog,
            file_server::file_root,
            run::run_workflow,
            run::cancel_run,
            run::run_status,
            run::watch_live,
            fonts::list_fonts,
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
            views::preview_open,
            views::preview_load,
            views::preview_play,
            views::preview_pause,
            views::preview_seek,
            views::preview_resize,
            views::preview_close,
            relay::relay,
            page_log::page_log,
            relay::send_menu_to_editor,
            api::api_status,
            api::api_listening,
            api::api_reply,
        ])
        .events(collect_events![
            events::ProjectFileChanged,
            events::ProjectFilesChanged,
            events::RunEventFired,
            events::EngineInstallProgress,
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
            events::Compiled,
            events::Relayed,
            events::ApiRequestReceived,
            views::ViewEventFired,
            views::PreviewReported,
        ])
        .types(&ui_types().register::<opsis_api::bodies::EditorRequest>())
        .dangerously_cast_bigints_to_number()
}

pub fn bindings() -> Result<String, String> {
    let file = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
    builder()
        .export(specta_typescript::Typescript::default(), file.path())
        .map_err(|e| e.to_string())?;
    let text = std::fs::read_to_string(file.path()).map_err(|e| e.to_string())?;
    Ok(tidy_export(&text))
}
