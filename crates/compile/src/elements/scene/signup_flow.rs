use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, neutral_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct SignupFlow {
    #[setting("Theme", transform = neutral_themes, widget = "theme")]
    pub theme: Theme,
    #[setting("Title", default = "Create an account")]
    pub title: String,
    #[setting(
        "Description",
        default = "Enter your information below to create your account"
    )]
    pub description: String,
    #[setting("Name typed in", default = "John Doe")]
    pub full_name: String,
    #[setting("Email typed in", default = "m@example.com")]
    pub email: String,
    #[setting("Password shown", default = "••••••••")]
    pub password: String,
    #[setting("Button label", default = "Create account")]
    pub create_label: String,
    #[setting("Notice", default = "Account created")]
    pub toast_title: String,
}

pub fn definition() -> ElementMeta {
    meta::<SignupFlow>(
        element(
            ElementKind::Component,
            "signup-flow",
            "Signup flow",
            "A sign-up form that fills itself in, step by step.",
            ElementCategory::Scene,
            240,
        )
        .authored_clock()
        .exact_design(1280, 720)
        .look(NodeLook::of("lucide:user-plus")),
    )
}
