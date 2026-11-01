use opsis_macros::settings;
use opsis_types::looks::NodeLook;

use crate::backdrops::Backdrop;
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct StudioBackdrop {
    #[setting("Backdrop", default = Backdrop::Glaze)]
    pub name: Backdrop,
}

pub fn definition() -> ElementMeta {
    meta::<StudioBackdrop>(
        element(
            ElementKind::Component,
            "backdrop",
            "Studio backdrop",
            "A ready-made studio background image.",
            ElementCategory::Background,
            150,
        )
        .look(NodeLook::of("lucide:wallpaper")),
    )
}
