use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, neutral_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum MenuState {
    Opened,
    Closed,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum MenuIcon {
    Search,
    Settings,
    User,
    File,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct CommandMenuItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Icon",
        extend("type" = "string", "enum" = ["search", "settings", "user", "file"])
    )]
    pub icon: Option<MenuIcon>,
    #[default = "New item"]
    #[schemars(description = "Label")]
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Shortcut", extend("type" = "string"))]
    pub shortcut: Option<String>,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct CommandMenu {
    #[default(MenuState::Opened)]
    #[schemars(
        description = "Menu",
        extend(
            "x-show-if" = { "autoplay": ["false"] },
            "x-labels" = { "opened": "Open", "closed": "Closed" }
        )
    )]
    pub state: MenuState,
    #[default = "new"]
    #[schemars(description = "Search")]
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Letters typed",
        range(min = 0, max = 200),
        extend(
            "x-zero-label" = "All",
            "x-step" = 1,
            "x-show-if" = { "autoplay": ["false"] },
            "type" = "integer"
        )
    )]
    pub reveal_count: Option<u32>,
    #[default(-1)]
    #[schemars(
        description = "Selected item",
        range(min = -1, max = 20),
        extend(
            "x-step" = 1,
            "x-help" = "Counting from 0. -1 selects none.",
            "x-show-if" = { "autoplay": ["false"] }
        )
    )]
    pub selected_index: i32,
    #[default(-1)]
    #[schemars(
        description = "Hovered item",
        range(min = -1, max = 20),
        extend(
            "x-step" = 1,
            "x-help" = "Counting from 0. -1 hovers none.",
            "x-show-if" = { "autoplay": ["false"] }
        )
    )]
    pub highlighted_index: i32,
    #[schemars(transform = neutral_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[default(vec![
        CommandMenuItem {
            icon: Some(MenuIcon::User),
            label: "Profile".into(),
            shortcut: Some("⌘ P".into()),
        },
        CommandMenuItem {
            icon: Some(MenuIcon::Settings),
            label: "Settings".into(),
            shortcut: Some("⌘ S".into()),
        },
        CommandMenuItem {
            icon: Some(MenuIcon::File),
            label: "New File".into(),
            shortcut: Some("⌘ N".into()),
        },
        CommandMenuItem {
            icon: Some(MenuIcon::Search),
            label: "Search docs".into(),
            ..Default::default()
        },
    ])]
    #[schemars(description = "Items")]
    pub items: Vec<CommandMenuItem>,
    #[default = true]
    #[schemars(
        description = "Play it out",
        extend(
            "x-help" = "Opens the menu, types the search, then picks the first match. Off shows one moment you set below."
        )
    )]
    pub autoplay: bool,
    #[default(-1)]
    #[schemars(
        description = "Press item",
        range(min = -1, max = 20),
        extend(
            "x-step" = 1,
            "x-help" = "The item that gets pressed, counting from 0. -1 presses none.",
            "x-show-if" = { "autoplay": ["false"] }
        )
    )]
    pub pressed_index: i32,
}

pub fn definition() -> ElementMeta {
    meta::<CommandMenu>(
        element(
            ElementKind::Component,
            "command-menu",
            "Command menu",
            "A command palette that opens, filters and picks an item.",
            ElementCategory::Scene,
            240,
        )
        .authored_clock()
        .exact_design(1280, 720)
        .look(NodeLook::of("lucide:search-code")),
    )
}
