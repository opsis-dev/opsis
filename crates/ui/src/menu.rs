use opsis_qt::{MenuItemKind, MenuItemView, MenuSectionView};
use opsis_session::menu::app_menu;
use opsis_types::shell::{MenuEntry, WindowRole};
use serde_json::Value;

const ROLE_PREFIX: &str = "role:";

const fn shown_role(role: WindowRole) -> bool {
    !matches!(
        role,
        WindowRole::Reload
            | WindowRole::ToggleDevTools
            | WindowRole::ResetZoom
            | WindowRole::ZoomIn
            | WindowRole::ZoomOut
    )
}

pub fn qt_shortcut(accelerator: &str, mac: bool) -> String {
    accelerator
        .split('+')
        .map(|key| match key {
            "CmdOrCtrl" | "Cmd" => "Ctrl",
            "Ctrl" if mac => "Meta",
            other => other,
        })
        .collect::<Vec<_>>()
        .join("+")
}

fn id_of(value: &impl serde::Serialize) -> String {
    match serde_json::to_value(value) {
        Ok(Value::String(id)) => id,
        _ => String::new(),
    }
}

fn item(entry: &MenuEntry, mac: bool) -> Option<MenuItemView> {
    let shortcut = |accelerator: &Option<String>| {
        accelerator
            .as_deref()
            .map_or_else(String::new, |a| qt_shortcut(a, mac))
    };
    match entry {
        MenuEntry::Command {
            label,
            command,
            accelerator,
        } => Some(MenuItemView {
            kind: MenuItemKind::Command,
            id: id_of(command),
            label: label.clone(),
            shortcut: shortcut(accelerator),
        }),
        MenuEntry::Role {
            label,
            role,
            accelerator,
        } => shown_role(*role).then(|| MenuItemView {
            kind: MenuItemKind::Role,
            id: format!("{ROLE_PREFIX}{}", id_of(role)),
            label: label.clone(),
            shortcut: shortcut(accelerator),
        }),
        MenuEntry::Separator => Some(MenuItemView {
            kind: MenuItemKind::Separator,
            id: String::new(),
            label: String::new(),
            shortcut: String::new(),
        }),
    }
}

fn tidy(items: Vec<MenuItemView>) -> Vec<MenuItemView> {
    let mut kept: Vec<MenuItemView> = Vec::new();
    for item in items {
        let separator = item.kind == MenuItemKind::Separator;
        let after_separator = kept
            .last()
            .is_none_or(|last| last.kind == MenuItemKind::Separator);
        if !(separator && after_separator) {
            kept.push(item);
        }
    }
    if kept
        .last()
        .is_some_and(|last| last.kind == MenuItemKind::Separator)
    {
        kept.pop();
    }
    kept
}

pub fn menu(mac: bool) -> Vec<MenuSectionView> {
    app_menu(mac)
        .into_iter()
        .map(|section| MenuSectionView {
            label: section.label,
            items: tidy(
                section
                    .items
                    .iter()
                    .filter_map(|entry| item(entry, mac))
                    .collect(),
            ),
        })
        .filter(|section| !section.items.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn shortcuts_read_as_qt_writes_them() {
        assert_eq!(qt_shortcut("CmdOrCtrl+Shift+P", false), "Ctrl+Shift+P");
        assert_eq!(qt_shortcut("CmdOrCtrl+Alt+B", true), "Ctrl+Alt+B");
        assert_eq!(qt_shortcut("Ctrl+Cmd+F", true), "Meta+Ctrl+F");
        assert_eq!(qt_shortcut("F11", false), "F11");
    }

    #[test]
    fn webview_roles_stay_out_of_the_native_menu() {
        let ids: Vec<String> = menu(false)
            .into_iter()
            .flat_map(|section| section.items.into_iter().map(|item| item.id))
            .collect();
        assert!(!ids.iter().any(|id| id == "role:reload"));
        assert!(!ids.iter().any(|id| id == "role:toggleDevTools"));
        assert!(ids.iter().any(|id| id == "project:save"));
    }

    fn separator() -> MenuItemView {
        MenuItemView {
            kind: MenuItemKind::Separator,
            id: String::new(),
            label: String::new(),
            shortcut: String::new(),
        }
    }

    fn entry(id: usize) -> MenuItemView {
        MenuItemView {
            kind: MenuItemKind::Command,
            id: id.to_string(),
            label: String::new(),
            shortcut: String::new(),
        }
    }

    proptest! {
        #[test]
        fn tidy_menus_never_lead_end_or_double_a_separator(
            layout in proptest::collection::vec(any::<bool>(), 0..24)
        ) {
            let items: Vec<MenuItemView> = layout
                .iter()
                .enumerate()
                .map(|(i, is_separator)| if *is_separator { separator() } else { entry(i) })
                .collect();
            let commands = items.iter().filter(|i| i.kind == MenuItemKind::Command).count();
            let kept = tidy(items);
            let is_separator = |item: &MenuItemView| item.kind == MenuItemKind::Separator;
            prop_assert!(kept.first().is_none_or(|first| !is_separator(first)));
            prop_assert!(kept.last().is_none_or(|last| !is_separator(last)));
            prop_assert!(kept.windows(2).all(|pair| !(is_separator(&pair[0]) && is_separator(&pair[1]))));
            prop_assert_eq!(kept.iter().filter(|i| i.kind == MenuItemKind::Command).count(), commands);
        }
    }
}
