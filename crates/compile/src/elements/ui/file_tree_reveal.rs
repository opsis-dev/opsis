use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct FileTreeReveal {
    #[setting(
        "Files",
        default = vec![
            "src/components/cart.tsx".into(),
            "src/components/checkout.tsx".into(),
            "src/lib/api.ts".into(),
            "src/index.ts".into(),
            "public/logo.svg".into(),
            "README.md".into(),
            "package.json".into(),
        ],
        help = "One path per item, like src/app.ts. End a folder with / to show it empty."
    )]
    pub nodes: Vec<String>,
    #[setting(
        "Title",
        default = "acme-app",
        help = "Title bar over the tree. Omit to drop the header."
    )]
    pub title: String,
    #[setting(
        "Highlighted file",
        default = "src/components/checkout.tsx",
        help = "Slash-joined path of the file that lights up at the end. Omit to select nothing."
    )]
    pub selected_path: String,
    #[setting(
        "Row stagger",
        default = 0.18,
        step = 0.01,
        help = "Seconds between one row appearing and the next.",
        advanced
    )]
    pub row_stagger: f64,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Selected row's bar, tint and label.",
        check = colour
    )]
    pub accent_color: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = code_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        help = "Animation speed multiplier."
    )]
    pub speed: f64,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme."
    )]
    pub background_color: String,
    #[setting("File count word", default = "files", advanced)]
    pub count_label: String,
    #[setting(
        "Panel width",
        default = 480,
        range(min = 280, max = 1200),
        step = 1,
        unit = "px",
        advanced
    )]
    pub panel_width: u32,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<FileTreeReveal>(
        element(
            ElementKind::Component,
            "file-tree-reveal",
            "File tree",
            "A folder tree that opens one item at a time and highlights a file.",
            ElementCategory::Ui,
            120,
        )
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:folder-tree")),
    )
}
