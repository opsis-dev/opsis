use opsis_macros::{choice, settings};
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, neutral_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum MenuState {
    #[label("Open")]
    Opened,
    #[label("Closed")]
    Closed,
}

#[choice(export)]
pub enum MenuIcon {
    Search,
    Settings,
    User,
    File,
}

#[settings(export, eq)]
pub struct CommandMenuItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Icon", "type" = "string", "enum" = ["search", "settings", "user", "file"])]
    pub icon: Option<MenuIcon>,
    #[setting("Label", default = "New item")]
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Shortcut", "type" = "string")]
    pub shortcut: Option<String>,
}

#[settings(export)]
pub struct CommandMenu {
    #[setting(
        "Menu",
        default = MenuState::Opened,
        show_if = { "autoplay": ["false"] }
    )]
    pub state: MenuState,
    #[setting("Search", default = "new")]
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Letters typed",
        range(min = 0, max = 200),
        zero_label = "All",
        step = 1,
        show_if = { "autoplay": ["false"] },
        "type" = "integer",
        advanced
    )]
    pub reveal_count: Option<u32>,
    #[setting(
        "Selected item",
        default = -1,
        range(min = -1, max = 20),
        step = 1,
        help = "Counting from 0. -1 selects none.",
        show_if = { "autoplay": ["false"] }
    )]
    pub selected_index: i32,
    #[setting(
        "Hovered item",
        default = -1,
        range(min = -1, max = 20),
        step = 1,
        help = "Counting from 0. -1 hovers none.",
        show_if = { "autoplay": ["false"] },
        advanced
    )]
    pub highlighted_index: i32,
    #[setting("Theme", transform = neutral_themes, widget = "theme")]
    pub theme: Theme,
    #[setting(
        "Items",
        default = vec![
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
        ]
    )]
    pub items: Vec<CommandMenuItem>,
    #[setting(
        "Play it out",
        default = true,
        help = "Opens the menu, types the search, then picks the first match. Off shows one moment you set below."
    )]
    pub autoplay: bool,
    #[setting(
        "Press item",
        default = -1,
        range(min = -1, max = 20),
        step = 1,
        help = "The item that gets pressed, counting from 0. -1 presses none.",
        show_if = { "autoplay": ["false"] },
        advanced
    )]
    pub pressed_index: i32,
    #[setting(
        "Time per letter",
        default = 0.0833,
        range(min = 0.0333, max = 0.4),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub letter_seconds: f64,
    #[setting(
        "Pause before picking",
        default = 0.6,
        range(min = 0, max = 3),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub pause_seconds: f64,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4))]
    pub speed: f64,
    #[setting(
        "Match picked",
        default = 0,
        range(min = 0, max = 20),
        step = 1,
        help = "Which of the matching items gets picked, counting from 0.",
        show_if = { "autoplay": ["true"] },
        advanced
    )]
    pub pick_index: u32,
    #[setting("Placeholder", default = "Type a command or search…", advanced)]
    pub placeholder: String,
    #[setting("No match text", default = "No results found.", advanced)]
    pub empty_text: String,
    #[setting("Backdrop dimming", default = 0.5, range(min = 0, max = 1), advanced)]
    pub backdrop_opacity: f64,
    #[setting(
        "Menu width",
        default = 440,
        range(min = 280, max = 900),
        step = 1,
        unit = "px",
        advanced
    )]
    pub panel_width: u32,
    #[setting(
        "Distance from top",
        default = 18.0,
        range(min = 0, max = 60),
        unit = "%",
        advanced
    )]
    pub top: f64,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
    #[setting(
        "Corner radius",
        default = 10,
        range(min = 0, max = 32),
        step = 1,
        unit = "px",
        advanced
    )]
    pub radius: u32,
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
