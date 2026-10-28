use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, neutral_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct SignupFlow {
    #[schemars(transform = neutral_themes)]
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
