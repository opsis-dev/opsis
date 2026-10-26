use std::sync::Mutex;
use std::time::Duration;

use opsis_store::windows::Bounds;
use opsis_types::shell::{DockPanelName, SettingsPage, WindowRole, WindowState};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use tauri::window::Color;
use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalSize, Position, Size, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder, Window, WindowEvent,
};
use tauri_specta::Event;

use crate::error::CommandResult;
use crate::events::{PanelWindowsChanged, SettingsPageShown, WindowStateChanged};
use crate::state::AppState;

pub const MAIN_WINDOW: &str = "main";
pub const SETTINGS_WINDOW: &str = "settings";
pub const PREVIEW_WINDOW: &str = "preview";
const PANEL_PREFIX: &str = "panel-";
const MAIN_WINDOW_KEY: &str = "mainWindow";
const BACKGROUND: Color = Color(0x10, 0x10, 0x13, 0xff);
const SAVE_BOUNDS_AFTER: Duration = Duration::from_millis(400);
const MIN_VISIBLE_PX: i32 = 40;
const ABOVE_WORK_AREA_PX: i32 = 20;
const ZOOM_STEP: f64 = 0.1;
const ZOOM_LIMITS: (f64, f64) = (0.3, 3.0);

struct WindowSpec {
    label: String,
    route: String,
    title: String,
    size: (f64, f64),
    min: (f64, f64),
}

#[derive(Default)]
pub struct WindowRuntime {
    zoom: Mutex<std::collections::HashMap<String, f64>>,
    saving: Mutex<std::collections::HashMap<String, tauri::async_runtime::JoinHandle<()>>>,
}

fn base_builder<'a>(
    app: &'a AppHandle,
    spec: &WindowSpec,
) -> WebviewWindowBuilder<'a, tauri::Wry, AppHandle> {
    let url = WebviewUrl::App(format!("index.html#{}", spec.route).into());
    let builder = WebviewWindowBuilder::new(app, &spec.label, url)
        .title(&spec.title)
        .inner_size(spec.size.0, spec.size.1)
        .min_inner_size(spec.min.0, spec.min.1)
        .background_color(BACKGROUND)
        .visible(false)
        .disable_drag_drop_handler();
    #[cfg(target_os = "macos")]
    let builder = builder
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true)
        .traffic_light_position(tauri::LogicalPosition::new(16.0, 16.0));
    #[cfg(not(target_os = "macos"))]
    let builder = builder.decorations(false);
    builder
}

fn visible_bounds(app: &AppHandle, b: Bounds) -> Option<Bounds> {
    let monitors = app.available_monitors().ok()?;
    let fits = monitors.iter().any(|m| {
        let area = m.work_area();
        let (x, y) = (area.position.x, area.position.y);
        let (w, h) = (area.size.width as i32, area.size.height as i32);
        let across = b.x < x + w - MIN_VISIBLE_PX && b.x + b.width > x + MIN_VISIBLE_PX;
        let reachable = b.y >= y - ABOVE_WORK_AREA_PX && b.y < y + h - MIN_VISIBLE_PX;
        across && reachable
    });
    fits.then_some(b)
}

fn place(window: &WebviewWindow, bounds: Bounds) {
    let moved = window.set_position(Position::Physical(PhysicalPosition::new(
        bounds.x, bounds.y,
    )));
    let sized = window.set_size(Size::Physical(PhysicalSize::new(
        bounds.width as u32,
        bounds.height as u32,
    )));
    if let Err(error) = moved.and(sized) {
        log::warn!("placing a window: {error}");
    }
}

fn bounds_of(window: &Window) -> Option<Bounds> {
    let position = window.outer_position().ok()?;
    let size = window.inner_size().ok()?;
    Some(Bounds {
        x: position.x,
        y: position.y,
        width: size.width as i32,
        height: size.height as i32,
    })
}

fn bounds_key(label: &str) -> String {
    match label.strip_prefix(PANEL_PREFIX) {
        Some(panel) => format!("panel:{panel}"),
        None => label.to_owned(),
    }
}

fn show(window: &WebviewWindow) {
    if let Err(error) = window.show().and(window.set_focus()) {
        log::warn!("showing a window: {error}");
    }
}

fn open_remembered(app: &AppHandle, spec: &WindowSpec) -> CommandResult<Option<WebviewWindow>> {
    if let Some(existing) = app.get_webview_window(&spec.label) {
        if existing.is_minimized().unwrap_or(false)
            && let Err(error) = existing.unminimize()
        {
            log::warn!("restoring a window: {error}");
        }
        show(&existing);
        return Ok(None);
    }
    let window = base_builder(app, spec).build()?;
    let saved = app
        .state::<AppState>()
        .db()
        .saved_bounds(&bounds_key(&spec.label))?;
    if let Some(bounds) = saved.and_then(|b| visible_bounds(app, b)) {
        place(&window, bounds);
    }
    show(&window);
    Ok(Some(window))
}

pub fn open_main(app: &AppHandle) -> CommandResult<()> {
    let spec = WindowSpec {
        label: MAIN_WINDOW.to_owned(),
        route: "/".to_owned(),
        title: "Opsis".to_owned(),
        size: (1600.0, 1000.0),
        min: (1100.0, 700.0),
    };
    let window = base_builder(app, &spec)
        .background_throttling(tauri::utils::config::BackgroundThrottlingPolicy::Disabled)
        .build()?;
    show(&window);
    Ok(())
}

pub fn panel_label(panel: DockPanelName) -> String {
    format!("{PANEL_PREFIX}{}", panel.id())
}

fn panel_size(panel: DockPanelName) -> (f64, f64) {
    match panel {
        DockPanelName::Timeline => (1400.0, 460.0),
        DockPanelName::Mixer => (1100.0, 560.0),
        DockPanelName::Run => (760.0, 560.0),
    }
}

pub fn open_panels(app: &AppHandle) -> Vec<DockPanelName> {
    DockPanelName::ALL
        .into_iter()
        .filter(|p| app.get_webview_window(&panel_label(*p)).is_some())
        .collect()
}

fn announce_panels(app: &AppHandle) {
    if let Err(error) = PanelWindowsChanged(open_panels(app)).emit_to(app, MAIN_WINDOW) {
        log::warn!("telling the editor which panels are open: {error}");
    }
}

#[derive(Deserialize)]
struct SavedWindow {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    maximized: bool,
}

pub fn apply_project_window(window: &WebviewWindow, view: &Map<String, Value>) {
    if window.is_fullscreen().unwrap_or(false) {
        return;
    }
    let saved = view
        .get(MAIN_WINDOW_KEY)
        .and_then(|v| serde_json::from_value::<SavedWindow>(v.clone()).ok());
    let Some(saved) = saved else {
        if let Err(error) = window.maximize() {
            log::warn!("maximizing the window: {error}");
        }
        return;
    };
    if window.is_maximized().unwrap_or(false)
        && let Err(error) = window.unmaximize()
    {
        log::warn!("restoring the window: {error}");
    }
    if let Some(bounds) = visible_bounds(
        window.app_handle(),
        Bounds {
            x: saved.x,
            y: saved.y,
            width: saved.width,
            height: saved.height,
        },
    ) {
        place(window, bounds);
    }
    if saved.maximized
        && let Err(error) = window.maximize()
    {
        log::warn!("maximizing the window: {error}");
    }
}

pub fn window_state(window: &Window) -> WindowState {
    WindowState {
        maximized: window.is_maximized().unwrap_or(false),
        fullscreen: window.is_fullscreen().unwrap_or(false),
        focused: window.is_focused().unwrap_or(false),
    }
}

fn send_state(window: &Window) {
    if let Err(error) =
        WindowStateChanged(window_state(window)).emit_to(window.app_handle(), window.label())
    {
        log::warn!("sending the window's state: {error}");
    }
}

fn save_bounds_now(window: &Window) {
    let app = window.app_handle();
    let state = app.state::<AppState>();
    if window.is_minimized().unwrap_or(false) || window.is_fullscreen().unwrap_or(false) {
        return;
    }
    let maximized = window.is_maximized().unwrap_or(false);
    let Some(bounds) = bounds_of(window) else {
        return;
    };
    if window.label() == MAIN_WINDOW {
        let dir = state.editor().dir.clone();
        if let Some(dir) = dir {
            let value = json!({ "x": bounds.x, "y": bounds.y, "width": bounds.width, "height": bounds.height, "maximized": maximized });
            if let Err(error) =
                state
                    .db()
                    .write_view_state(&dir.to_string_lossy(), MAIN_WINDOW_KEY, &value)
            {
                log::warn!("saving the window's size: {error}");
            }
        }
        return;
    }
    if maximized {
        return;
    }
    if let Err(error) = state.db().save_bounds(&bounds_key(window.label()), bounds) {
        log::warn!("saving the window's size: {error}");
    }
}

fn save_bounds_soon(window: &Window) {
    let runtime = window.app_handle().state::<WindowRuntime>();
    let label = window.label().to_owned();
    let delayed = window.clone();
    let task = tauri::async_runtime::spawn(async move {
        tokio::time::sleep(SAVE_BOUNDS_AFTER).await;
        save_bounds_now(&delayed);
    });
    let mut saving = runtime.saving.lock().expect("bounds savers");
    if let Some(previous) = saving.insert(label, task) {
        previous.abort();
    }
}

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::Resized(_) | WindowEvent::Moved(_) => {
            send_state(window);
            save_bounds_soon(window);
        }
        WindowEvent::Focused(_) => send_state(window),
        WindowEvent::CloseRequested { api, .. } if window.label() == MAIN_WINDOW => {
            crate::exit::close_requested(window, api)
        }
        WindowEvent::Destroyed => {
            save_bounds_now(window);
            let app = window.app_handle();
            if window.label() == MAIN_WINDOW {
                app.state::<crate::api::Api>().editor_closed();
                for (label, other) in app.webview_windows() {
                    if label != MAIN_WINDOW
                        && let Err(error) = other.close()
                    {
                        log::warn!("closing {label}: {error}");
                    }
                }
            } else if window.label().starts_with(PANEL_PREFIX) {
                announce_panels(app);
            } else if window.label() == PREVIEW_WINDOW {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    app.state::<crate::views::Views>().close_preview().await;
                });
            }
        }
        _ => {}
    }
}

#[tauri::command]
#[specta::specta]
pub fn open_settings(app: AppHandle, page: Option<SettingsPage>) -> CommandResult<()> {
    let route = match page {
        Some(page) => format!(
            "/settings?page={}",
            serde_json::to_value(page)?.as_str().unwrap_or_default()
        ),
        None => "/settings".to_owned(),
    };
    let spec = WindowSpec {
        label: SETTINGS_WINDOW.to_owned(),
        route,
        title: "Settings — Opsis".to_owned(),
        size: (980.0, 720.0),
        min: (720.0, 480.0),
    };
    if open_remembered(&app, &spec)?.is_none()
        && let Some(page) = page
        && let Err(error) = SettingsPageShown(page).emit_to(&app, SETTINGS_WINDOW)
    {
        log::warn!("showing a settings page: {error}");
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn open_panel_window(app: AppHandle, panel: DockPanelName) -> CommandResult<()> {
    let spec = WindowSpec {
        label: panel_label(panel),
        route: format!("/panel/{}", panel.id()),
        title: format!("{} — Opsis", panel.title()),
        size: panel_size(panel),
        min: (420.0, 260.0),
    };
    if open_remembered(&app, &spec)?.is_some() {
        announce_panels(&app);
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn open_preview_window(app: AppHandle) -> CommandResult<()> {
    let spec = WindowSpec {
        label: PREVIEW_WINDOW.to_owned(),
        route: "/preview".to_owned(),
        title: "Preview — Opsis".to_owned(),
        size: (960.0, 640.0),
        min: (420.0, 260.0),
    };
    open_remembered(&app, &spec)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn focus_panel_window(app: AppHandle, panel: DockPanelName) {
    if let Some(window) = app.get_webview_window(&panel_label(panel)) {
        show(&window);
    }
}

#[tauri::command]
#[specta::specta]
pub fn close_panel_window(app: AppHandle, panel: DockPanelName) -> CommandResult<()> {
    if let Some(window) = app.get_webview_window(&panel_label(panel)) {
        window.close()?;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn close_panel_windows(app: AppHandle) -> CommandResult<()> {
    for panel in open_panels(&app) {
        if let Some(window) = app.get_webview_window(&panel_label(panel)) {
            window.close()?;
        }
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn list_panel_windows(app: AppHandle) -> Vec<DockPanelName> {
    open_panels(&app)
}

#[tauri::command]
#[specta::specta]
pub fn get_window_state(window: Window) -> WindowState {
    window_state(&window)
}

#[tauri::command]
#[specta::specta]
pub fn set_title(window: Window, title: String) -> CommandResult<()> {
    Ok(window.set_title(&title)?)
}

fn edit_command(webview: &WebviewWindow, command: &str) -> CommandResult<()> {
    Ok(webview.eval(format!("document.execCommand({})", json!(command)))?)
}

#[tauri::command]
#[specta::specta]
pub fn window_role(app: AppHandle, webview: WebviewWindow, role: WindowRole) -> CommandResult<()> {
    run_role(&app, &webview, role)
}

pub fn run_role(app: &AppHandle, webview: &WebviewWindow, role: WindowRole) -> CommandResult<()> {
    let runtime = app.state::<WindowRuntime>();
    let zoom_by = |step: Option<f64>| -> CommandResult<()> {
        let mut zooms = runtime.zoom.lock().expect("zoom levels");
        let level = zooms.entry(webview.label().to_owned()).or_insert(1.0);
        *level = step.map_or(1.0, |s| (*level + s).clamp(ZOOM_LIMITS.0, ZOOM_LIMITS.1));
        Ok(webview.set_zoom(*level)?)
    };
    match role {
        WindowRole::Cut => edit_command(webview, "cut"),
        WindowRole::Copy => edit_command(webview, "copy"),
        WindowRole::Paste => edit_command(webview, "paste"),
        WindowRole::SelectAll => edit_command(webview, "selectAll"),
        WindowRole::Reload => Ok(webview.reload()?),
        WindowRole::ToggleDevTools => {
            #[cfg(debug_assertions)]
            if webview.is_devtools_open() {
                webview.close_devtools();
            } else {
                webview.open_devtools();
            }
            Ok(())
        }
        WindowRole::ResetZoom => zoom_by(None),
        WindowRole::ZoomIn => zoom_by(Some(ZOOM_STEP)),
        WindowRole::ZoomOut => zoom_by(Some(-ZOOM_STEP)),
        WindowRole::ToggleFullscreen => Ok(webview.set_fullscreen(!webview.is_fullscreen()?)?),
        WindowRole::Minimize => Ok(webview.minimize()?),
        WindowRole::ToggleMaximize => {
            if webview.is_maximized()? {
                webview.unmaximize()?;
            } else {
                webview.maximize()?;
            }
            Ok(())
        }
        WindowRole::Close => Ok(webview.close()?),
        WindowRole::Quit => {
            crate::exit::quit(app);
            Ok(())
        }
    }
}
