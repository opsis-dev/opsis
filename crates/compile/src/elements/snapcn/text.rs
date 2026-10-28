use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::choices::{FontWeight, HighlightPreset};
use crate::elements::core::Colour;
use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, ThemeMode};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct AnnounceTitle {
    #[default = "Introducing"]
    #[schemars(description = "Eyebrow")]
    pub eyebrow: String,
    #[default = "Acme 2.0"]
    #[schemars(description = "Title")]
    pub title: String,
    #[default = "Everything your team needs to ship, in one place."]
    #[schemars(description = "Tagline")]
    pub tagline: String,
    #[schemars(description = "Symbol path")]
    pub symbol_path: String,
    #[schemars(description = "Symbol colours", length(min = 0, max = 8), extend("x-help" = "Leave empty for the built-in colours."))]
    pub symbol_colors: Vec<Colour>,
    #[default = 1.0]
    #[schemars(description = "Symbol size", range(min = 0, max = 3), extend("x-step" = 0.05))]
    pub symbol_scale: f64,
    #[default = "#5600f5"]
    #[schemars(description = "Colour field", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub field_color: String,
    #[default = "#4800c9"]
    #[schemars(description = "Title ink", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub title_color: String,
    #[default = "#fcfcfa"]
    #[schemars(description = "Title paper", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub paper_color: String,
    #[default = "#100022"]
    #[schemars(description = "Opening ground", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub void_color: String,
    #[default = "#000028"]
    #[schemars(description = "Closing ground", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub night_color: String,
    #[default = "#08ff4b"]
    #[schemars(description = "Closing glow", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub glow_color: String,
    #[default = 0.139]
    #[schemars(description = "Glow strength", range(min = 0, max = 0.4), extend("x-step" = 0.005))]
    pub glow_strength: f64,
    #[default = "#f2f8ff"]
    #[schemars(description = "Tagline ink", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub tagline_color: String,
    #[default = "#ffffff"]
    #[schemars(description = "Shot ink", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub ink_color: String,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct PunchLines {
    #[default = "Record it. / Edit it. | One take. | Ship."]
    #[schemars(description = "Script (| card, / line)")]
    pub script: String,
    #[default = "slide,punch"]
    #[schemars(description = "Card looks")]
    pub styles: String,
    #[default = "1,0.945,2.7"]
    #[schemars(description = "Card sizes")]
    pub sizes: String,
    #[default = "1.6"]
    #[schemars(description = "Card lengths", extend("x-frames-at" = 30))]
    pub holds: String,
    #[default = 3.0]
    #[schemars(description = "Accent card", range(min = 0, max = 8), extend("x-step" = 1))]
    pub accent_beat: f64,
    #[default = 93.0]
    #[schemars(description = "Font size", range(min = 24, max = 200), extend("x-step" = 1))]
    pub font_size: f64,
    #[default(FontWeight::W600)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[default = 0.89]
    #[schemars(description = "Line height (em)", range(min = 0.7, max = 1.6), extend("x-step" = 0.005))]
    pub line_height: f64,
    #[default = "-0.022em"]
    #[schemars(description = "Letter spacing")]
    pub letter_spacing: String,
    #[default = "-0.02em"]
    #[schemars(description = "Word spacing")]
    pub word_spacing: String,
    #[schemars(description = "Ground", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub ground: String,
    #[schemars(description = "Type", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub ink: String,
    #[default = "#ff1c8e"]
    #[schemars(description = "Accent", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default = 0.1333]
    #[schemars(description = "Entrance lead", range(min = 0, max = 1), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub lead: f64,
    #[default(-0.737)]
    #[schemars(description = "Slide from (em)", range(min = -3, max = 0), extend("x-step" = 0.01))]
    pub slide_from: f64,
    #[default = 7.34]
    #[schemars(description = "Slide kick (em/s)", range(min = 0, max = 20), extend("x-step" = 0.1))]
    pub slide_kick: f64,
    #[default = 2.846]
    #[schemars(description = "Slide settle", range(min = 0.5, max = 8), extend("x-step" = 0.05))]
    pub slide_settle: f64,
    #[default = 0.1167]
    #[schemars(description = "Line stagger", range(min = 0, max = 0.6667), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub line_stagger: f64,
    #[default = 0.313]
    #[schemars(description = "Slide drop (em)", range(min = -1, max = 1), extend("x-step" = 0.01))]
    pub slide_drop: f64,
    #[default = 1.0]
    #[schemars(description = "Motion blur", range(min = 0, max = 1), extend("x-step" = 0.05))]
    pub motion_blur: f64,
    #[default = 0.32]
    #[schemars(description = "Punch from", range(min = 0.05, max = 1), extend("x-step" = 0.01))]
    pub punch_from: f64,
    #[default = 0.5]
    #[schemars(description = "Punch", range(min = 0.0667, max = 1.3333), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub punch_frames: f64,
    #[default = 0.0933]
    #[schemars(description = "Word stagger", range(min = 0, max = 0.4), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub word_stagger: f64,
    #[default = 0.778]
    #[schemars(description = "Push from", range(min = 0.3, max = 1), extend("x-step" = 0.005))]
    pub push_from: f64,
    #[default = 0.205]
    #[schemars(description = "Push per second", range(min = 0, max = 1), extend("x-step" = 0.005))]
    pub push_rate: f64,
    #[default = 0.4067]
    #[schemars(description = "Rush lead", range(min = 0, max = 1.3333), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub rush_lead: f64,
    #[default = 0.6]
    #[schemars(description = "Rush", range(min = 0.1333, max = 2), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub rush_frames: f64,
    #[default = 0.85]
    #[schemars(description = "Rush travel", range(min = 0.2, max = 0.95), extend("x-step" = 0.01))]
    pub rush_travel: f64,
    #[default = 0.92]
    #[schemars(description = "Fit width", range(min = 0.4, max = 1), extend("x-step" = 0.01))]
    pub fit: f64,
    #[schemars(transform = super::snapcn_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct TextHighlight {
    #[default = "Powered by "]
    #[schemars(description = "Before", extend("x-show-if" = {"preset":["marker","color","underline","strikethrough","shimmer"]}))]
    pub before: String,
    #[default = "Northwind"]
    #[schemars(description = "Highlight / wordmark")]
    pub highlight: String,
    #[schemars(description = "After", extend("x-show-if" = {"preset":["marker","color","underline","strikethrough","shimmer"]}))]
    pub after: String,
    #[default(HighlightPreset::LogoWipe)]
    #[schemars(description = "Preset")]
    pub preset: HighlightPreset,
    #[schemars(description = "Base color", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub base_color: String,
    #[default = "#266DF0"]
    #[schemars(description = "Accent color", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub accent_color: String,
    #[schemars(description = "Highlighted text colour", extend("x-widget" = "color", "x-zero-label" = "Auto"))]
    pub highlighted_text_color: String,
    #[default = "Acme"]
    #[schemars(description = "Replace with (strikethrough)")]
    pub replace_with: String,
    #[default = 0.2]
    #[schemars(description = "Start at", range(min = 0, max = 4), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub start_at: f64,
    #[default = 0.4667]
    #[schemars(description = "Draw time", range(min = 0.1667, max = 4), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub draw_duration: f64,
    #[default = 4.0]
    #[schemars(description = "Line thickness", range(min = 1, max = 20), extend("x-step" = 1))]
    pub thickness: f64,
    #[default = "#FAFAFA"]
    #[schemars(description = "Shine color (shimmer)", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub shine_color: String,
    #[default = "app:placeholders/logo-mark.svg"]
    #[schemars(description = "Logo", extend("x-widget" = "asset", "x-accept" = "image/*", "x-help" = "The mark the logo wipe uses."))]
    pub logo_src: String,
    #[default = 1.05]
    #[schemars(description = "Mark size (× font)", range(min = 0.4, max = 3), extend("x-step" = 0.05))]
    pub logo_scale: f64,
    #[default = 1.0]
    #[schemars(description = "Mark spin (turns)", range(min = 0, max = 4), extend("x-step" = 0.25))]
    pub spin_turns: f64,
    #[default = 0.4]
    #[schemars(description = "Lockup hold", range(min = 0, max = 2), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub hold_duration: f64,
    #[default = 0.8]
    #[schemars(description = "Rush time", range(min = 0.2, max = 2.6667), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub rush_duration: f64,
    #[default = 42.0]
    #[schemars(description = "Rush scale (× mark)", range(min = 2, max = 80), extend("x-step" = 1))]
    pub rush_scale: f64,
    #[default = 0.55]
    #[schemars(description = "Swing across (× width)", range(min = -0.8, max = 0.8), extend("x-step" = 0.05))]
    pub rush_drift: f64,
    #[default = 0.25]
    #[schemars(description = "Swing arc (× width)", range(min = 0, max = 0.33), extend("x-step" = 0.01))]
    pub swing_out: f64,
    #[default = "#266DF0"]
    #[schemars(description = "Ink color (end of rush)", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub cover_color: String,
    #[default = 56.0]
    #[schemars(description = "Font size", range(min = 12, max = 160), extend("x-step" = 1))]
    pub font_size: f64,
    #[default(FontWeight::W600)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[default(Theme::of(ThemeMode::Dark))]
    #[schemars(transform = super::snapcn_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Set it to transparent to lay the text over other layers."))]
    pub background: String,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct StatusCycle {
    #[default = "Acme is"]
    #[schemars(description = "Prefix")]
    pub prefix: String,
    #[default = "recording, editing, exporting, live"]
    #[schemars(description = "Statuses (comma-separated)")]
    pub statuses: String,
    #[default = "Record, Auto zoom, Captions, Export, Share"]
    #[schemars(description = "Chips (comma-separated)")]
    pub chips: String,
    #[schemars(description = "Field (act 1)", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub field_color: String,
    #[schemars(description = "Page (act 2)", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub page_color: String,
    #[schemars(description = "Pill", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub pill_color: String,
    #[schemars(description = "Pill label", extend("x-widget" = "color", "x-zero-label" = "Default"))]
    pub pill_label_color: String,
    #[schemars(description = "Prefix ink", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub prefix_color: String,
    #[schemars(description = "Chip fills", length(min = 0, max = 8), extend("x-help" = "Leave empty for the theme's chip colours."))]
    pub chip_fills: Vec<Colour>,
    #[schemars(description = "Chip ink", extend("x-widget" = "color", "x-zero-label" = "Theme"))]
    pub chip_text_color: String,
    #[default = "rgb(160, 160, 160)"]
    #[schemars(description = "Chip hairline", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub chip_border_color: String,
    #[default(FontWeight::W400)]
    #[schemars(description = "Font weight")]
    pub font_weight: FontWeight,
    #[default(FontWeight::W400)]
    #[schemars(description = "Chip font weight")]
    pub chip_font_weight: FontWeight,
    #[default = 63.0]
    #[schemars(description = "Status size", range(min = 24, max = 140), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = 160.0]
    #[schemars(description = "Chip size", range(min = 48, max = 260), extend("x-step" = 2))]
    pub chip_font_size: f64,
    #[default = 0.8]
    #[schemars(description = "Intro length", range(min = 0.2667, max = 2), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub intro_frames: f64,
    #[default = 1.0]
    #[schemars(description = "Status hold", range(min = 0.3333, max = 2), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub status_hold: f64,
    #[default = 0.2667]
    #[schemars(description = "Chip stagger", range(min = 0.1333, max = 1), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub chip_stagger: f64,
    #[default = 0.516]
    #[schemars(description = "Lockup height", range(min = 0.2, max = 0.8), extend("x-step" = 0.004))]
    pub pill_center_y: f64,
    #[schemars(description = "Start at", range(min = 0, max = 2), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub start_at: f64,
    #[default(Theme::of(ThemeMode::Dark))]
    #[schemars(transform = super::snapcn_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Status corner radius", range(min = 0, max = 200), extend("x-zero-label" = "Auto", "x-step" = 1, "x-unit" = "px", "x-advanced" = true, "type" = "number"))]
    pub pill_radius: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Chip corner radius", range(min = 0, max = 200), extend("x-zero-label" = "Auto", "x-step" = 1, "x-unit" = "px", "x-advanced" = true, "type" = "number"))]
    pub chip_radius: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Chip border width", range(min = 0, max = 20), extend("x-zero-label" = "Auto", "x-step" = 0.5, "x-unit" = "px", "x-advanced" = true, "type" = "number"))]
    pub chip_border_width: Option<f64>,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<AnnounceTitle>(
            element(
                ElementKind::Component,
                "announce-title",
                "Announce title",
                "Launch title animation, product announcement intro with kinetic typography. A four-shot launch title: the eyebrow rushes past the camera on a receding type plane and lands on a colour field, the name assembles word by word on paper, and a macro pan across the tagline cuts wide as the line builds itself last word first while a white sweep drains it out of colour.",
                C::Text,
                170,
            )
                .source("snapcn", "https://snapcn.dev/docs/announce-title", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:megaphone")),
        ),
        meta::<PunchLines>(
            element(
                ElementKind::Component,
                "punch-lines",
                "Punch lines",
                "Kinetic typography title cards: bold statements that cut hard between beats. A run of full-frame title cards that cut hard between beats.",
                C::Text,
                144,
            )
                .source("snapcn", "https://snapcn.dev/docs/punch-lines", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:type").moves(NodeMotion::HardCut)),
        ),
        meta::<TextHighlight>(
            element(
                ElementKind::Component,
                "text-highlight",
                "Text highlight",
                "Text highlight animation: marker highlighter, underline, strikethrough and shimmer on a word. Animated emphasis on one span inside a static sentence — marker sweep, color shift, underline, strikethrough-replace, or shimmer.",
                C::Text,
                56,
            )
                .source("snapcn", "https://snapcn.dev/docs/text-highlight", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:baseline")),
        ),
        meta::<StatusCycle>(
            element(
                ElementKind::Component,
                "status-cycle",
                "Status cycle",
                "Status badge animation: a pill whose label cycles through states. A centred line whose status pill cycles: the label rolls up behind a hard clip while the pill's width springs past its target and back, and the line re-centres around it.",
                C::Text,
                198,
            )
                .source("snapcn", "https://snapcn.dev/docs/status-cycle", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:badge")),
        ),
    ]
}
