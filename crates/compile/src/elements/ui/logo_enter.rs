use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::Orientation;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoEnterLogo {
    #[setting(
        "Image",
        default = "app:placeholders/logo-mark.svg",
        widget = "asset",
        accept = "image/*"
    )]
    pub image: String,
    #[setting("Background", default = "#2563eb", check = colour)]
    pub background: String,
}

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoEnter {
    #[setting(
        "Diameter",
        default = 118,
        range(min = 48, max = 320),
        step = 1,
        unit = "px"
    )]
    pub diameter: u32,
    #[setting(
        "Overlap",
        default = 38,
        range(min = 0, max = 200),
        step = 1,
        unit = "px"
    )]
    pub overlap: u32,
    #[setting("Ring", default = "#fff", check = colour)]
    pub ring_color: String,
    #[setting("Orientation", default = Orientation::Horizontal)]
    pub orientation: Orientation,
    #[setting(
        "Stagger",
        default = 0.2333,
        range(min = 0),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub stagger: f64,
    #[setting(
        "Logos",
        default = vec![
            LogoEnterLogo::default(),
            LogoEnterLogo { background: "#0f766e".into(), ..Default::default() },
            LogoEnterLogo { background: "#b45309".into(), ..Default::default() },
            LogoEnterLogo { background: "#7c3aed".into(), ..Default::default() },
        ],
        length(min = 1, max = 16)
    )]
    pub logos: Vec<LogoEnterLogo>,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<LogoEnter>(
            element(
                ElementKind::Component,
                "logo-enter",
                "Logo enter",
                "A stacked group of round brand chips that spring in one-by-one, then hold. Swap the logos to feature any set of marks.",
                ElementCategory::Ui,
                90,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:layout-grid-circles")),
        )
}
