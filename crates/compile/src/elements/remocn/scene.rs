use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::choices::{MenuIcon, MenuState};
use crate::elements::entry::{element, meta};
use crate::elements::theme::Theme;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct CommandMenuItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Icon", extend("type" = "string", "enum" = ["search","settings","user","file"]))]
    pub icon: Option<MenuIcon>,
    #[default = "New item"]
    #[schemars(description = "Label")]
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Shortcut", extend("type" = "string"))]
    pub shortcut: Option<String>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct LaunchAnythingLogoImage {
    #[default = "app:placeholders/logo-mark.svg"]
    #[schemars(description = "Image", extend("x-widget" = "asset", "x-accept" = "image/*"))]
    pub image: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SpringSettle {
    #[default = 1.24]
    #[schemars(description = "Enter scale", range(min = 1, max = 1.8), extend("x-step" = 0.02))]
    pub enter_scale: f64,
    #[default = 3.0]
    #[schemars(description = "Time between items", range(min = 0, max = 10), extend("x-step" = 0.5, "x-unit" = "frames", "x-help" = "How long each item in a scene waits after the one before it."))]
    pub enter_stagger: f64,
    #[default = 0.84]
    #[schemars(description = "Exit scale", range(min = 0.5, max = 1), extend("x-step" = 0.02))]
    pub exit_scale: f64,
    #[default = 6.0]
    #[schemars(description = "Exit time", range(min = 2, max = 24), extend("x-step" = 1, "x-unit" = "frames"))]
    pub exit_frames: f64,
    #[default = 5.0]
    #[schemars(description = "Fade-in time", range(min = 0, max = 30), extend("x-step" = 1, "x-unit" = "frames", "x-advanced" = true))]
    pub enter_fade_frames: f64,
    #[default = 30.0]
    #[schemars(description = "Spring damping", range(min = 1, max = 100), extend("x-step" = 1, "x-help" = "Lower values bounce more as items land.", "x-advanced" = true))]
    pub spring_damping: f64,
    #[default = 320.0]
    #[schemars(description = "Spring stiffness", range(min = 10, max = 1000), extend("x-step" = 10, "x-help" = "Higher values land faster.", "x-advanced" = true))]
    pub spring_stiffness: f64,
    #[default = 4.0]
    #[schemars(description = "Background fade", range(min = 0, max = 30), extend("x-step" = 1, "x-unit" = "frames", "x-advanced" = true))]
    pub bg_fade_frames: f64,
    #[schemars(description = "Loop", extend("x-help" = "Start again from the first scene after the last."))]
    pub r#loop: bool,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SignupFlow {
    #[schemars(transform = super::remocn_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[default = "Create an account"]
    #[schemars(description = "Title")]
    pub title: String,
    #[default = "Enter your information below to create your account"]
    #[schemars(description = "Description")]
    pub description: String,
    #[default = "John Doe"]
    #[schemars(description = "Name typed in")]
    pub full_name: String,
    #[default = "m@example.com"]
    #[schemars(description = "Email typed in")]
    pub email: String,
    #[default = "••••••••"]
    #[schemars(description = "Password shown")]
    pub password: String,
    #[default = "Create account"]
    #[schemars(description = "Button label")]
    pub create_label: String,
    #[default = "Account created"]
    #[schemars(description = "Notice")]
    pub toast_title: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct AiPromptFlow {
    #[schemars(transform = super::remocn_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[default = "Summarize this thread"]
    #[schemars(description = "Prompt")]
    pub prompt: String,
    #[default = "Generate"]
    #[schemars(description = "Button label")]
    pub button_label: String,
    #[default(vec!["The thread debates the Q3 roadmap: ship the editor first,".into(), "defer billing to Q4, and pull the migration forward so".into(), "infra is unblocked before the team scales next quarter.".into()])]
    #[schemars(description = "Answer", extend("x-help" = "One line of the answer per item."))]
    pub answer_lines: Vec<String>,
    #[default = "Response ready"]
    #[schemars(description = "Notice")]
    pub toast_title: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct CommandMenu {
    #[default(MenuState::Opened)]
    #[schemars(description = "Menu", extend("x-show-if" = {"autoplay":["false"]}, "x-labels" = {"opened":"Open","closed":"Closed"}))]
    pub state: MenuState,
    #[default = "new"]
    #[schemars(description = "Search")]
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Letters typed", range(min = 0, max = 200), extend("x-zero-label" = "All", "x-step" = 1, "x-show-if" = {"autoplay":["false"]}, "type" = "number"))]
    pub reveal_count: Option<f64>,
    #[default(-1.0)]
    #[schemars(description = "Selected item", range(min = -1, max = 20), extend("x-step" = 1, "x-help" = "Counting from 0. -1 selects none.", "x-show-if" = {"autoplay":["false"]}))]
    pub selected_index: f64,
    #[default(-1.0)]
    #[schemars(description = "Hovered item", range(min = -1, max = 20), extend("x-step" = 1, "x-help" = "Counting from 0. -1 hovers none.", "x-show-if" = {"autoplay":["false"]}))]
    pub highlighted_index: f64,
    #[schemars(transform = super::remocn_themes)]
    #[schemars(description = "Theme", extend("x-widget" = "theme"))]
    pub theme: Theme,
    #[default(vec![
        CommandMenuItem { icon: Some(MenuIcon::User), label: "Profile".into(), shortcut: Some("⌘ P".into()) },
        CommandMenuItem { icon: Some(MenuIcon::Settings), label: "Settings".into(), shortcut: Some("⌘ S".into()) },
        CommandMenuItem { icon: Some(MenuIcon::File), label: "New File".into(), shortcut: Some("⌘ N".into()) },
        CommandMenuItem { icon: Some(MenuIcon::Search), label: "Search docs".into(), ..Default::default() },
    ])]
    #[schemars(description = "Items")]
    pub items: Vec<CommandMenuItem>,
    #[default = true]
    #[schemars(description = "Play it out", extend("x-help" = "Opens the menu, types the search, then picks the first match. Off shows one moment you set below."))]
    pub autoplay: bool,
    #[default(-1.0)]
    #[schemars(description = "Press item", range(min = -1, max = 20), extend("x-step" = 1, "x-help" = "The item that gets pressed, counting from 0. -1 presses none.", "x-show-if" = {"autoplay":["false"]}))]
    pub pressed_index: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ReleaseTeaser {
    #[default = "Orvio"]
    #[schemars(description = "Name in the final release lockup")]
    pub brand_name: String,
    #[default = "2"]
    #[schemars(description = "Version suffix; leave empty to hide")]
    pub release: String,
    #[default = "Your next chapter starts here."]
    #[schemars(description = "Final supporting line")]
    pub tagline: String,
    #[default = "#9BC8DD"]
    #[schemars(description = "Mark and dimensional edge lighting", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[schemars(description = "Static mark and complete statements, without camera motion or blur")]
    pub reduced_motion: bool,
    #[default(vec!["Start with an idea".into(), "Make room\nfor better work".into(), "Bring every detail to life".into(), "Move together.\nGo further.".into(), "A fresh chapter.\nNo starting over.".into()])]
    #[schemars(description = "Statements", length(min = 5, max = 5), extend("x-help" = "The five lines, in order."))]
    pub statements: Vec<String>,
    #[schemars(description = "Logo", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Name only"))]
    pub logo_src: String,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Default"))]
    pub theme_background: String,
    #[schemars(description = "Panel colour", extend("x-widget" = "color", "x-zero-label" = "Default"))]
    pub theme_surface: String,
    #[schemars(description = "Text colour", extend("x-widget" = "color", "x-zero-label" = "Default"))]
    pub theme_foreground: String,
    #[schemars(description = "Soft text colour", extend("x-widget" = "color", "x-zero-label" = "Default"))]
    pub theme_muted: String,
    #[default = 1.0]
    #[schemars(description = "Light", range(min = 0, max = 2), extend("x-step" = 0.05, "x-advanced" = true))]
    pub light_intensity: f64,
    #[schemars(description = "Music", extend("x-zero-label" = "None", "x-help" = "Plays through the Mixer like any other music.", "x-widget" = "sound"))]
    pub music: String,
    #[default = 1.0]
    #[schemars(description = "Music volume", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub music_volume: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct LaunchAnything {
    #[default = "Good"]
    #[schemars(description = "Opening: first word")]
    pub opening: String,
    #[default = "ideas"]
    #[schemars(description = "Opening: second word")]
    pub subject: String,
    #[default = "yourproduct.example"]
    #[schemars(description = "Closing: web address")]
    pub brand_url: String,
    #[default = "#245744"]
    #[schemars(description = "Accent colour", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default = "deserve a stage"]
    #[schemars(description = "Opening: after the subject", extend("x-advanced" = true))]
    pub copy_ready: String,
    #[default = "Let’s make"]
    #[schemars(description = "Action: lead-in", extend("x-advanced" = true))]
    pub copy_next: String,
    #[default = "them real"]
    #[schemars(description = "Action: button text", extend("x-advanced" = true))]
    pub copy_action: String,
    #[default = "Your work.\nA clearer view."]
    #[schemars(description = "Proof: headline", extend("x-widget" = "textarea", "x-advanced" = true))]
    pub copy_proof: String,
    #[default = "Built around you"]
    #[schemars(description = "Industry: headline", extend("x-advanced" = true))]
    pub copy_industry: String,
    #[default = "Made to be yours."]
    #[schemars(description = "Industry: promise", extend("x-advanced" = true))]
    pub copy_promise: String,
    #[default = "Begin"]
    #[schemars(description = "Closing: call to action", extend("x-advanced" = true))]
    pub copy_launch: String,
    #[default = "Small things. Well made."]
    #[schemars(description = "Shop screen: headline", extend("x-advanced" = true))]
    pub copy_shoppers: String,
    #[default = "Object Study"]
    #[schemars(description = "Shop screen: shop name", extend("x-advanced" = true))]
    pub copy_shop_name: String,
    #[default = "Everyday, considered."]
    #[schemars(description = "Shop screen: shop headline", extend("x-advanced" = true))]
    pub copy_shop_headline: String,
    #[default = "Useful objects. Quiet details."]
    #[schemars(description = "Shop screen: shop detail", extend("x-advanced" = true))]
    pub copy_shop_detail: String,
    #[default = "Explore the objects"]
    #[schemars(description = "Shop screen: shop button", extend("x-advanced" = true))]
    pub copy_shop_action: String,
    #[default = "Turn a brief into a workspace"]
    #[schemars(description = "Builder screen: headline", extend("x-advanced" = true))]
    pub copy_builder: String,
    #[default = "Create a workspace for our next product release"]
    #[schemars(description = "Builder screen: prompt", extend("x-advanced" = true))]
    pub copy_builder_prompt: String,
    #[default = "Review once.\nMove forward."]
    #[schemars(description = "Review screen: headline", extend("x-widget" = "textarea", "x-advanced" = true))]
    pub copy_trading: String,
    #[default = "A place for every idea"]
    #[schemars(description = "Ideas screen: headline", extend("x-advanced" = true))]
    pub copy_loyalty: String,
    #[default = "Room for the next one"]
    #[schemars(description = "Ideas screen: payoff", extend("x-advanced" = true))]
    pub copy_loyalty_payoff: String,
    #[default = "Bring your work together"]
    #[schemars(description = "Tools screen: headline", extend("x-advanced" = true))]
    pub copy_integration: String,
    #[default = "Connect your tools.\nKeep work moving."]
    #[schemars(description = "Tools screen: closing line", extend("x-widget" = "textarea", "x-advanced" = true))]
    pub copy_connections: String,
    #[default(vec!["Files".into(), "Messages".into(), "Calendar".into()])]
    #[schemars(description = "Tools screen: tools", extend("x-advanced" = true))]
    pub copy_integrations: Vec<String>,
    #[schemars(description = "Screen: dashboard", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Built-in screen", "x-help" = "Shown on the laptop for this part. Leave empty for the built-in screen.", "x-advanced" = true))]
    pub screen_dashboard: String,
    #[schemars(description = "Screen: shoppers", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Built-in screen", "x-help" = "Shown on the laptop for this part. Leave empty for the built-in screen.", "x-advanced" = true))]
    pub screen_shoppers: String,
    #[schemars(description = "Screen: exchange", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Built-in screen", "x-help" = "Shown on the laptop for this part. Leave empty for the built-in screen.", "x-advanced" = true))]
    pub screen_exchange: String,
    #[schemars(description = "Screen: builder", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Built-in screen", "x-help" = "Shown on the laptop for this part. Leave empty for the built-in screen.", "x-advanced" = true))]
    pub screen_builder: String,
    #[schemars(description = "Screen: trading", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Built-in screen", "x-help" = "Shown on the laptop for this part. Leave empty for the built-in screen.", "x-advanced" = true))]
    pub screen_trading: String,
    #[schemars(description = "Screen: loyalty", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Built-in screen", "x-help" = "Shown on the laptop for this part. Leave empty for the built-in screen.", "x-advanced" = true))]
    pub screen_loyalty: String,
    #[schemars(description = "Screen: integrations", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Built-in screen", "x-help" = "Shown on the laptop for this part. Leave empty for the built-in screen.", "x-advanced" = true))]
    pub screen_integrations: String,
    #[schemars(description = "Photo: desk", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Built-in photo", "x-advanced" = true))]
    pub photo_desk: String,
    #[schemars(description = "Photo: horizon", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Built-in photo", "x-advanced" = true))]
    pub photo_horizon: String,
    #[schemars(description = "Photo: rocket", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Built-in photo", "x-advanced" = true))]
    pub photo_rocket: String,
    #[schemars(description = "Photo: earth", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Built-in photo", "x-advanced" = true))]
    pub photo_earth: String,
    #[schemars(description = "Logos", extend("x-help" = "Marks for the floating grid in the proof scene."))]
    pub logo_images: Vec<LaunchAnythingLogoImage>,
    #[schemars(description = "Logo", extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Built-in mark"))]
    pub logo_src: String,
    #[schemars(description = "Music", extend("x-zero-label" = "None", "x-help" = "Plays through the Mixer like any other music.", "x-widget" = "sound"))]
    pub music: String,
    #[default = 1.0]
    #[schemars(description = "Music volume", range(min = 0, max = 1), extend("x-step" = 0.01))]
    pub music_volume: f64,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<SpringSettle>(
            element(
                ElementKind::Sequencer,
                "spring-settle",
                "Spring settle",
                "Plays clips as scenes: each scene shrinks away, the stage holds empty for a beat, and the next one lands. Connect Layers to make a scene whose layers land one after another.",
                C::Scene,
                213,
            )
                .source("Remocn", "https://remocn.dev/docs/components/spring-settle", "MIT")
                .look(NodeLook::of("tabler:carousel-vertical")),
        ),
        meta::<SignupFlow>(
            element(
                ElementKind::Component,
                "signup-flow",
                "Signup flow",
                "A sign-up form that fills itself in, step by step.",
                C::Scene,
                240,
            )
                .source("Remocn", "https://remocn.dev/docs/components/signup-flow", "MIT")
                .authored_clock()
                .exact_design(1280, 720)
                .look(NodeLook::of("lucide:user-plus")),
        ),
        meta::<AiPromptFlow>(
            element(
                ElementKind::Component,
                "ai-prompt-flow",
                "AI prompt flow",
                "An AI prompt composition: a prompt types into the field, the Generate button runs hover → press → loading, a skeleton shimmer reveals, then crossfades into the generated answer, and a ready toast slides in. A pure orchestrator — the shimmer is owned by the skeleton-block motion atom; every other channel comes from a composed primitive's hook.",
                C::Scene,
                240,
            )
                .source("Remocn", "https://remocn.dev/docs/components/ai-prompt-flow", "MIT")
                .authored_clock()
                .exact_design(1280, 720)
                .look(NodeLook::of("lucide:wand-sparkles")),
        ),
        meta::<CommandMenu>(
            element(
                ElementKind::Component,
                "command-menu",
                "Command menu",
                "A command palette that opens, filters and picks an item.",
                C::Scene,
                240,
            )
                .source("Remocn", "https://remocn.dev/docs/components/command-menu", "MIT")
                .authored_clock()
                .exact_design(1280, 720)
                .look(NodeLook::of("lucide:search-code")),
        ),
        meta::<ReleaseTeaser>(
            element(
                ElementKind::Component,
                "release-teaser",
                "Release teaser",
                "A 16-second cinematic release teaser with center-out typography, a procedural dimensional mark, moving edge light, and an independent fictional identity.",
                C::Scene,
                300,
            )
                .source("Remocn", "https://remocn.dev/docs/components/release-teaser", "MIT")
                .look(NodeLook::of("lucide:rocket")),
        ),
        meta::<LaunchAnything>(
            element(
                ElementKind::Component,
                "launch-anything",
                "Launch anything",
                "An independent 26.7-second editorial product film with forest-green accents, original photography, seven editable demos, and geometric marks.",
                C::Scene,
                600,
            )
                .source("Remocn", "https://remocn.dev/docs/components/launch-anything", "MIT")
                .look(NodeLook::of("lucide:film")),
        ),
    ]
}
