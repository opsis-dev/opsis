use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Corner;
use crate::elements::theme::{Theme, neutral_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum CursorVariant {
    Arrow,
    Pointer,
}

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
    #[setting("Name label", default = "Full Name")]
    pub name_label: String,
    #[setting("Email label", default = "Email")]
    pub email_label: String,
    #[setting("Email help", default = "We'll use this to contact you.")]
    pub email_help: String,
    #[setting("Password label", default = "Password")]
    pub password_label: String,
    #[setting("Password help", default = "Must be at least 8 characters long.")]
    pub password_help: String,
    #[setting("Confirm label", default = "Confirm Password")]
    pub confirm_label: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4))]
    pub speed: f64,
    #[setting(
        "Loading time",
        default = 1.4,
        range(min = 0.2, max = 6),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub loading_seconds: f64,
    #[setting(
        "Notice shown for",
        default = 2.2,
        range(min = 0.5, max = 6),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub notice_seconds: f64,
    #[setting("Cursor", default = CursorVariant::Pointer, advanced)]
    pub cursor_shape: CursorVariant,
    #[setting(
        "Cursor size",
        default = 28,
        range(min = 16, max = 64),
        step = 1,
        unit = "px",
        advanced
    )]
    pub cursor_size: u32,
    #[setting("Notice position", default = Corner::BottomRight, advanced)]
    pub notice_corner: Corner,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "None",
        check = colour_or_empty
    )]
    pub background_color: String,
    #[setting(
        "Corner radius",
        default = 10,
        range(min = 0, max = 32),
        step = 1,
        unit = "px",
        advanced
    )]
    pub radius: u32,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
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
