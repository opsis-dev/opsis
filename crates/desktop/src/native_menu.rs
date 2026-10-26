use opsis_types::shell::{MenuCommand, MenuEntry, WindowRole};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tauri::menu::{Menu, MenuBuilder, MenuEvent, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Manager, WebviewWindow, Wry};
use tauri_specta::Event;

use crate::events::MenuCommandFired;
use crate::menu::app_menu;
use crate::windows::{MAIN_WINDOW, run_role};

const COMMAND_PREFIX: &str = "command:";
const ROLE_PREFIX: &str = "role:";
const QUIT_ACCELERATOR: &str = "Cmd+Q";

#[derive(Debug, PartialEq, Eq)]
enum Picked {
    Command(MenuCommand),
    Role(WindowRole),
}

fn wire_name(value: &impl Serialize) -> String {
    match serde_json::to_value(value) {
        Ok(Value::String(name)) => name,
        _ => String::new(),
    }
}

fn from_wire<T: DeserializeOwned>(name: &str) -> Option<T> {
    serde_json::from_value(Value::String(name.to_owned())).ok()
}

fn entry_id(entry: &MenuEntry) -> Option<String> {
    match entry {
        MenuEntry::Command { command, .. } => {
            Some(format!("{COMMAND_PREFIX}{}", wire_name(command)))
        }
        MenuEntry::Role { role, .. } => Some(format!("{ROLE_PREFIX}{}", wire_name(role))),
        MenuEntry::Separator => None,
    }
}

fn picked(id: &str) -> Option<Picked> {
    if let Some(name) = id.strip_prefix(COMMAND_PREFIX) {
        return from_wire(name).map(Picked::Command);
    }
    id.strip_prefix(ROLE_PREFIX)
        .and_then(from_wire)
        .map(Picked::Role)
}

pub fn build(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let info = app.package_info();
    let name = info.name.clone();
    let quit = MenuItemBuilder::new(format!("Quit {name}"))
        .id(format!("{ROLE_PREFIX}{}", wire_name(&WindowRole::Quit)))
        .accelerator(QUIT_ACCELERATOR)
        .build(app)?;
    let app_submenu = SubmenuBuilder::new(app, &name)
        .about(None)
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .item(&quit)
        .build()?;
    let mut menu = MenuBuilder::new(app).item(&app_submenu);
    for section in app_menu(true) {
        let mut submenu = SubmenuBuilder::new(app, &section.label);
        for entry in &section.items {
            submenu = match entry {
                MenuEntry::Separator => submenu.separator(),
                MenuEntry::Role {
                    role: WindowRole::Cut,
                    ..
                } => submenu.cut(),
                MenuEntry::Role {
                    role: WindowRole::Copy,
                    ..
                } => submenu.copy(),
                MenuEntry::Role {
                    role: WindowRole::Paste,
                    ..
                } => submenu.paste(),
                MenuEntry::Role {
                    role: WindowRole::SelectAll,
                    ..
                } => submenu.select_all(),
                MenuEntry::Command {
                    label, accelerator, ..
                }
                | MenuEntry::Role {
                    label, accelerator, ..
                } => {
                    let mut item =
                        MenuItemBuilder::new(label).id(entry_id(entry).unwrap_or_default());
                    if let Some(accelerator) = accelerator {
                        item = item.accelerator(accelerator);
                    }
                    submenu.item(&item.build(app)?)
                }
            };
        }
        menu = menu.item(&submenu.build()?);
    }
    let window = SubmenuBuilder::new(app, "Window")
        .minimize()
        .maximize()
        .separator()
        .bring_all_to_front()
        .build()?;
    let version = MenuItemBuilder::new(format!("{name} {}", info.version))
        .enabled(false)
        .build(app)?;
    let help = SubmenuBuilder::new(app, "Help").item(&version).build()?;
    menu.item(&window).item(&help).build()
}

fn focused_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.webview_windows()
        .into_values()
        .find(|w| w.is_focused().unwrap_or(false))
        .or_else(|| app.get_webview_window(MAIN_WINDOW))
}

pub fn on_event(app: &AppHandle, event: MenuEvent) {
    match picked(event.id().as_ref()) {
        Some(Picked::Command(command)) => {
            if let Err(error) = MenuCommandFired(command).emit_to(app, MAIN_WINDOW) {
                log::warn!("sending a menu command: {error}");
            }
        }
        Some(Picked::Role(role)) => {
            if let Some(window) = focused_window(app)
                && let Err(error) = run_role(app, &window, role)
            {
                log::warn!("running a menu role: {}", error.0);
            }
        }
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_menu_entry_comes_back_from_its_id() {
        for section in app_menu(true) {
            for entry in &section.items {
                let expected = match entry {
                    MenuEntry::Command { command, .. } => Some(Picked::Command(*command)),
                    MenuEntry::Role { role, .. } => Some(Picked::Role(*role)),
                    MenuEntry::Separator => None,
                };
                assert_eq!(
                    entry_id(entry).and_then(|id| picked(&id)),
                    expected,
                    "{entry:?}"
                );
            }
        }
        assert_eq!(picked("role:quit"), Some(Picked::Role(WindowRole::Quit)));
        assert_eq!(picked("command:nothing"), None);
        assert_eq!(picked("about"), None);
    }
}
