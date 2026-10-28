mod device;
mod scene;
mod text;

use schemars::Schema;
use serde_json::json;

use crate::elements::ElementMeta;
use crate::elements::theme::with_themes;

fn snapcn_themes(schema: &mut Schema) {
    with_themes(
        schema,
        json!({
            "light": {
                "background": "#faf9f6",
                "foreground": "#141414",
                "card": "#ffffff",
                "cardForeground": "#141414",
                "popover": "#ffffff",
                "popoverForeground": "#141414",
                "primary": "#3072db",
                "primaryForeground": "#ffffff",
                "secondary": "#f2f0eb",
                "secondaryForeground": "#141414",
                "muted": "#f2f0eb",
                "mutedForeground": "#6e6a63",
                "accent": "#f2f0eb",
                "accentForeground": "#141414",
                "destructive": "#d92d20",
                "destructiveForeground": "#ffffff",
                "border": "#d9d9d9",
                "input": "#d9d9d9",
                "ring": "#3072db"
            },
            "dark": {
                "background": "#0a0a0b",
                "foreground": "#fafafa",
                "card": "#141417",
                "cardForeground": "#fafafa",
                "popover": "#141417",
                "popoverForeground": "#fafafa",
                "primary": "#3072db",
                "primaryForeground": "#ffffff",
                "secondary": "#1d1d21",
                "secondaryForeground": "#fafafa",
                "muted": "#1b1b1f",
                "mutedForeground": "#a1a1aa",
                "accent": "#1d1d21",
                "accentForeground": "#fafafa",
                "destructive": "#f97066",
                "destructiveForeground": "#fafafa",
                "border": "#26272b",
                "input": "#26272b",
                "ring": "#3072db"
            }
        }),
    );
}

pub fn elements() -> Vec<ElementMeta> {
    [device::elements(), text::elements(), scene::elements()].concat()
}
