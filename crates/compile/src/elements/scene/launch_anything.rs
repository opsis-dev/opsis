use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LaunchAnythingLogoImage {
    #[default = "app:placeholders/logo-mark.svg"]
    #[schemars(description = "Image", extend("x-widget" = "asset", "x-accept" = "image/*"))]
    pub image: String,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
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
    #[schemars(
        description = "Accent colour",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
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
    #[schemars(
        description = "Proof: headline",
        extend("x-widget" = "textarea", "x-advanced" = true)
    )]
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
    #[schemars(
        description = "Review screen: headline",
        extend("x-widget" = "textarea", "x-advanced" = true)
    )]
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
    #[schemars(
        description = "Tools screen: closing line",
        extend("x-widget" = "textarea", "x-advanced" = true)
    )]
    pub copy_connections: String,
    #[default(vec!["Files".into(), "Messages".into(), "Calendar".into()])]
    #[schemars(description = "Tools screen: tools", extend("x-advanced" = true))]
    pub copy_integrations: Vec<String>,
    #[schemars(
        description = "Screen: dashboard",
        extend(
            "x-widget" = "asset",
            "x-accept" = "image/*",
            "x-zero-label" = "Built-in screen",
            "x-help" = "Shown on the laptop for this part. Leave empty for the built-in screen.",
            "x-advanced" = true
        )
    )]
    pub screen_dashboard: String,
    #[schemars(
        description = "Screen: shoppers",
        extend(
            "x-widget" = "asset",
            "x-accept" = "image/*",
            "x-zero-label" = "Built-in screen",
            "x-help" = "Shown on the laptop for this part. Leave empty for the built-in screen.",
            "x-advanced" = true
        )
    )]
    pub screen_shoppers: String,
    #[schemars(
        description = "Screen: exchange",
        extend(
            "x-widget" = "asset",
            "x-accept" = "image/*",
            "x-zero-label" = "Built-in screen",
            "x-help" = "Shown on the laptop for this part. Leave empty for the built-in screen.",
            "x-advanced" = true
        )
    )]
    pub screen_exchange: String,
    #[schemars(
        description = "Screen: builder",
        extend(
            "x-widget" = "asset",
            "x-accept" = "image/*",
            "x-zero-label" = "Built-in screen",
            "x-help" = "Shown on the laptop for this part. Leave empty for the built-in screen.",
            "x-advanced" = true
        )
    )]
    pub screen_builder: String,
    #[schemars(
        description = "Screen: trading",
        extend(
            "x-widget" = "asset",
            "x-accept" = "image/*",
            "x-zero-label" = "Built-in screen",
            "x-help" = "Shown on the laptop for this part. Leave empty for the built-in screen.",
            "x-advanced" = true
        )
    )]
    pub screen_trading: String,
    #[schemars(
        description = "Screen: loyalty",
        extend(
            "x-widget" = "asset",
            "x-accept" = "image/*",
            "x-zero-label" = "Built-in screen",
            "x-help" = "Shown on the laptop for this part. Leave empty for the built-in screen.",
            "x-advanced" = true
        )
    )]
    pub screen_loyalty: String,
    #[schemars(
        description = "Screen: integrations",
        extend(
            "x-widget" = "asset",
            "x-accept" = "image/*",
            "x-zero-label" = "Built-in screen",
            "x-help" = "Shown on the laptop for this part. Leave empty for the built-in screen.",
            "x-advanced" = true
        )
    )]
    pub screen_integrations: String,
    #[schemars(
        description = "Photo: desk",
        extend(
            "x-widget" = "asset",
            "x-accept" = "image/*",
            "x-zero-label" = "Built-in photo",
            "x-advanced" = true
        )
    )]
    pub photo_desk: String,
    #[schemars(
        description = "Photo: horizon",
        extend(
            "x-widget" = "asset",
            "x-accept" = "image/*",
            "x-zero-label" = "Built-in photo",
            "x-advanced" = true
        )
    )]
    pub photo_horizon: String,
    #[schemars(
        description = "Photo: rocket",
        extend(
            "x-widget" = "asset",
            "x-accept" = "image/*",
            "x-zero-label" = "Built-in photo",
            "x-advanced" = true
        )
    )]
    pub photo_rocket: String,
    #[schemars(
        description = "Photo: earth",
        extend(
            "x-widget" = "asset",
            "x-accept" = "image/*",
            "x-zero-label" = "Built-in photo",
            "x-advanced" = true
        )
    )]
    pub photo_earth: String,
    #[schemars(
        description = "Logos",
        extend("x-help" = "Marks for the floating grid in the proof scene.")
    )]
    pub logo_images: Vec<LaunchAnythingLogoImage>,
    #[schemars(
        description = "Logo",
        extend("x-widget" = "asset", "x-accept" = "image/*", "x-zero-label" = "Built-in mark")
    )]
    pub logo_src: String,
    #[schemars(
        description = "Music",
        extend(
            "x-zero-label" = "None",
            "x-help" = "Plays through the Mixer like any other music.",
            "x-widget" = "sound"
        )
    )]
    pub music: String,
    #[default = 1.0]
    #[schemars(description = "Music volume", range(min = 0, max = 1), extend("x-step" = 0.01))]
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
                .look(NodeLook::of("lucide:film")),
        )
}
