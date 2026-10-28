mod background;
mod data;
mod effect;
mod overlay;
mod scene;
mod text;
mod transition;
mod ui;

use schemars::Schema;
use serde_json::json;

use crate::elements::ElementMeta;
use crate::elements::theme::with_themes;

fn remocn_themes(schema: &mut Schema) {
    with_themes(
        schema,
        json!({
            "light": {
                "background": "oklch(1 0 0)",
                "foreground": "oklch(0.145 0 0)",
                "card": "oklch(1 0 0)",
                "cardForeground": "oklch(0.145 0 0)",
                "popover": "oklch(1 0 0)",
                "popoverForeground": "oklch(0.145 0 0)",
                "primary": "oklch(0.205 0 0)",
                "primaryForeground": "oklch(0.985 0 0)",
                "secondary": "oklch(0.97 0 0)",
                "secondaryForeground": "oklch(0.205 0 0)",
                "muted": "oklch(0.97 0 0)",
                "mutedForeground": "oklch(0.556 0 0)",
                "accent": "oklch(0.97 0 0)",
                "accentForeground": "oklch(0.205 0 0)",
                "destructive": "oklch(0.577 0.245 27.325)",
                "destructiveForeground": "oklch(0.985 0 0)",
                "border": "oklch(0.922 0 0)",
                "input": "oklch(0.922 0 0)",
                "ring": "oklch(0.708 0 0)"
            },
            "dark": {
                "background": "oklch(0.145 0 0)",
                "foreground": "oklch(0.985 0 0)",
                "card": "oklch(0.205 0 0)",
                "cardForeground": "oklch(0.985 0 0)",
                "popover": "oklch(0.205 0 0)",
                "popoverForeground": "oklch(0.985 0 0)",
                "primary": "oklch(0.922 0 0)",
                "primaryForeground": "oklch(0.205 0 0)",
                "secondary": "oklch(0.269 0 0)",
                "secondaryForeground": "oklch(0.985 0 0)",
                "muted": "oklch(0.269 0 0)",
                "mutedForeground": "oklch(0.708 0 0)",
                "accent": "oklch(0.269 0 0)",
                "accentForeground": "oklch(0.985 0 0)",
                "destructive": "oklch(0.704 0.191 22.216)",
                "destructiveForeground": "oklch(0.985 0 0)",
                "border": "oklch(1 0 0 / 10%)",
                "input": "oklch(1 0 0 / 15%)",
                "ring": "oklch(0.556 0 0)"
            }
        }),
    );
}

pub fn elements() -> Vec<ElementMeta> {
    [
        text::elements(),
        overlay::elements(),
        transition::elements(),
        background::elements(),
        effect::elements(),
        ui::elements(),
        data::elements(),
        scene::elements(),
    ]
    .concat()
}
