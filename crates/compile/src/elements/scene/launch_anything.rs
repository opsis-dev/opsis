use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LaunchAnythingLogoImage {
    #[setting(
        "Image",
        default = "app:placeholders/logo-mark.svg",
        widget = "asset",
        accept = "image/*"
    )]
    pub image: String,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LaunchAnything {
    #[setting("Opening: first word", default = "Good")]
    pub opening: String,
    #[setting("Opening: second word", default = "ideas")]
    pub subject: String,
    #[setting("Closing: web address", default = "yourproduct.example")]
    pub brand_url: String,
    #[setting("Accent colour", default = "#245744", check = colour)]
    pub accent_color: String,
    #[setting("Opening: after the subject", default = "deserve a stage", advanced)]
    pub copy_ready: String,
    #[setting("Action: lead-in", default = "Let’s make", advanced)]
    pub copy_next: String,
    #[setting("Action: button text", default = "them real", advanced)]
    pub copy_action: String,
    #[setting(
        "Proof: headline",
        default = "Your work.\nA clearer view.",
        widget = "textarea",
        advanced
    )]
    pub copy_proof: String,
    #[setting("Industry: headline", default = "Built around you", advanced)]
    pub copy_industry: String,
    #[setting("Industry: promise", default = "Made to be yours.", advanced)]
    pub copy_promise: String,
    #[setting("Closing: call to action", default = "Begin", advanced)]
    pub copy_launch: String,
    #[setting(
        "Shop screen: headline",
        default = "Small things. Well made.",
        advanced
    )]
    pub copy_shoppers: String,
    #[setting("Shop screen: shop name", default = "Object Study", advanced)]
    pub copy_shop_name: String,
    #[setting(
        "Shop screen: shop headline",
        default = "Everyday, considered.",
        advanced
    )]
    pub copy_shop_headline: String,
    #[setting(
        "Shop screen: shop detail",
        default = "Useful objects. Quiet details.",
        advanced
    )]
    pub copy_shop_detail: String,
    #[setting("Shop screen: shop button", default = "Explore the objects", advanced)]
    pub copy_shop_action: String,
    #[setting(
        "Builder screen: headline",
        default = "Turn a brief into a workspace",
        advanced
    )]
    pub copy_builder: String,
    #[setting(
        "Builder screen: prompt",
        default = "Create a workspace for our next product release",
        advanced
    )]
    pub copy_builder_prompt: String,
    #[setting(
        "Review screen: headline",
        default = "Review once.\nMove forward.",
        widget = "textarea",
        advanced
    )]
    pub copy_trading: String,
    #[setting("Ideas screen: headline", default = "A place for every idea", advanced)]
    pub copy_loyalty: String,
    #[setting("Ideas screen: payoff", default = "Room for the next one", advanced)]
    pub copy_loyalty_payoff: String,
    #[setting(
        "Tools screen: headline",
        default = "Bring your work together",
        advanced
    )]
    pub copy_integration: String,
    #[setting(
        "Tools screen: closing line",
        default = "Connect your tools.\nKeep work moving.",
        widget = "textarea",
        advanced
    )]
    pub copy_connections: String,
    #[setting(
        "Tools screen: tools",
        default = vec!["Files".into(), "Messages".into(), "Calendar".into()],
        advanced
    )]
    pub copy_integrations: Vec<String>,
    #[setting(
        "Screen: dashboard",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in screen",
        help = "Shown on the laptop for this part. Leave empty for the built-in screen.",
        advanced
    )]
    pub screen_dashboard: String,
    #[setting(
        "Screen: shoppers",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in screen",
        help = "Shown on the laptop for this part. Leave empty for the built-in screen.",
        advanced
    )]
    pub screen_shoppers: String,
    #[setting(
        "Screen: exchange",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in screen",
        help = "Shown on the laptop for this part. Leave empty for the built-in screen.",
        advanced
    )]
    pub screen_exchange: String,
    #[setting(
        "Screen: builder",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in screen",
        help = "Shown on the laptop for this part. Leave empty for the built-in screen.",
        advanced
    )]
    pub screen_builder: String,
    #[setting(
        "Screen: trading",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in screen",
        help = "Shown on the laptop for this part. Leave empty for the built-in screen.",
        advanced
    )]
    pub screen_trading: String,
    #[setting(
        "Screen: loyalty",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in screen",
        help = "Shown on the laptop for this part. Leave empty for the built-in screen.",
        advanced
    )]
    pub screen_loyalty: String,
    #[setting(
        "Screen: integrations",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in screen",
        help = "Shown on the laptop for this part. Leave empty for the built-in screen.",
        advanced
    )]
    pub screen_integrations: String,
    #[setting(
        "Photo: desk",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in photo",
        advanced
    )]
    pub photo_desk: String,
    #[setting(
        "Photo: horizon",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in photo",
        advanced
    )]
    pub photo_horizon: String,
    #[setting(
        "Photo: rocket",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in photo",
        advanced
    )]
    pub photo_rocket: String,
    #[setting(
        "Photo: earth",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in photo",
        advanced
    )]
    pub photo_earth: String,
    #[setting("Logos", help = "Marks for the floating grid in the proof scene.")]
    pub logo_images: Vec<LaunchAnythingLogoImage>,
    #[setting(
        "Logo",
        widget = "asset",
        accept = "image/*",
        zero_label = "Built-in mark"
    )]
    pub logo_src: String,
    #[setting(
        "Music",
        zero_label = "None",
        help = "Plays through the Mixer like any other music.",
        widget = "sound"
    )]
    pub music: String,
    #[setting("Music volume", default = 1.0, range(min = 0, max = 1), step = 0.01)]
    pub music_volume: f64,
}

pub fn definition() -> ElementMeta {
    meta::<LaunchAnything>(
            element(
                ElementKind::Component,
                "launch-anything",
                "Launch anything",
                "An independent 26.7-second editorial product film with forest-green accents, original photography, seven editable demos, and geometric marks.",
                ElementCategory::Scene,
                600,
            )
                .fonts(&["Manrope"])
                .look(NodeLook::of("lucide:film")),
        )
}
