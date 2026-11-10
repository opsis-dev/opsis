use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::LazyLock;

use opsis_types::numbers::whole;
use serde::Deserialize;

const STYLE_TEMPLATE: &str = include_str!("../resources/style.qss");

const TOKENS: [(&str, &str); 44] = [
    ("background", "oklch(0.175 0.007 286)"),
    ("track-video", "oklch(0.38 0.05 250)"),
    ("track-graphics", "oklch(0.38 0.06 300)"),
    ("track-text", "oklch(0.38 0.05 175)"),
    ("track-media", "oklch(0.39 0.05 50)"),
    ("track-music", "oklch(0.38 0.06 150)"),
    ("track-effect", "oklch(0.4 0.06 80)"),
    ("track-clip", "oklch(0.36 0.04 220)"),
    ("playhead", "oklch(0.7 0.18 38)"),
    ("keyframe", "oklch(0.9 0.1 90)"),
    ("waveform", "oklch(0.97 0.004 286 / 55%)"),
    ("meter-low", "oklch(0.72 0.16 150)"),
    ("meter-mid", "oklch(0.84 0.15 90)"),
    ("meter-high", "oklch(0.66 0.2 27)"),
    ("foreground", "oklch(0.93 0.004 286)"),
    ("card", "oklch(0.225 0.008 286)"),
    ("popover", "oklch(0.235 0.008 286)"),
    ("primary", "oklch(0.9 0.005 286)"),
    ("primary-foreground", "oklch(0.21 0.008 286)"),
    ("secondary", "oklch(0.26 0.009 286)"),
    ("muted", "oklch(0.255 0.009 286)"),
    ("muted-foreground", "oklch(0.67 0.013 286)"),
    ("accent", "oklch(0.285 0.01 286)"),
    ("accent-foreground", "oklch(0.95 0.004 286)"),
    ("destructive", "oklch(0.68 0.16 22.216)"),
    ("warning", "oklch(0.8 0.13 79.5)"),
    ("success", "oklch(0.74 0.15 152)"),
    ("border", "oklch(1 0 0 / 7%)"),
    ("input", "oklch(1 0 0 / 10%)"),
    ("ring", "oklch(0.55 0.015 286)"),
    ("sidebar", "oklch(0.205 0.008 286)"),
    ("sidebar-accent", "oklch(0.285 0.01 286)"),
    ("scrollbar-thumb", "oklch(0.93 0.004 286 / 16%)"),
    ("scrollbar-thumb-hover", "oklch(0.93 0.004 286 / 30%)"),
    ("toggle-on", "oklch(0.36 0.012 286)"),
    ("toggle-on-border", "oklch(0.93 0.004 286 / 24%)"),
    ("live", "oklch(0.6 0.22 27)"),
    ("live-foreground", "oklch(0.985 0.002 286)"),
    ("grid-dot", "oklch(0.93 0.004 286 / 9%)"),
    ("edge", "oklch(0.67 0.013 286)"),
    ("selection", "oklch(0.9 0.005 286 / 70%)"),
    ("rubber-band", "oklch(0.9 0.005 286 / 8%)"),
    ("window-border", "oklch(1 0 0 / 12%)"),
    ("shadow", "oklch(0 0 0 / 35%)"),
];

const RADIUS: f64 = 6.0;

const CATEGORY_COLORS: [(&str, &str); 17] = [
    ("Browser", "#38bdf8"),
    ("Terminal", "#4ade80"),
    ("Capture", "#f97316"),
    ("Camera", "#fb923c"),
    ("Cursor", "#fbbf24"),
    ("Text & overlays", "#a78bfa"),
    ("Transitions", "#c084fc"),
    ("Backgrounds", "#818cf8"),
    ("Effects", "#e879f9"),
    ("Grade", "#fb7185"),
    ("Elements", "#8b5cf6"),
    ("Audio", "#34d399"),
    ("Timeline", "#f472b6"),
    ("Media", "#60a5fa"),
    ("Image", "#2dd4bf"),
    ("Output", "#f43f5e"),
    ("Utility", NEUTRAL),
];

const CATEGORY_ICONS: [(&str, &str); 17] = [
    ("Browser", "globe"),
    ("Terminal", "square-terminal"),
    ("Capture", "video"),
    ("Camera", "scan-search"),
    ("Cursor", "mouse-pointer-2"),
    ("Text & overlays", "type"),
    ("Transitions", "blend"),
    ("Backgrounds", "wallpaper"),
    ("Effects", "sparkles"),
    ("Grade", "palette"),
    ("Elements", "shapes"),
    ("Audio", "music"),
    ("Timeline", "layers"),
    ("Media", "file-play"),
    ("Image", "image"),
    ("Output", "clapperboard"),
    ("Utility", "sticky-note"),
];

const PORT_COLORS: [(&str, &str); 11] = [
    ("browser-session", "#38bdf8"),
    ("terminal-session", "#4ade80"),
    ("recording", "#f97316"),
    ("recording-style", "#fbbf24"),
    ("grade", "#fb7185"),
    ("clip", "#a78bfa"),
    ("audio", "#34d399"),
    ("captions", "#facc15"),
    ("image", "#2dd4bf"),
    ("media", "#60a5fa"),
    ("transition", "#e879f9"),
];

pub const NEUTRAL: &str = "#71717a";

const FALLBACK_GLYPH: &str = "lucide:box";

type IconNode = Vec<(String, HashMap<String, String>)>;

#[derive(Debug, Deserialize)]
pub struct BakedMorph {
    pub from: f64,
    pub step: f64,
    pub closed: Vec<bool>,
    pub frames: Vec<Vec<Vec<f64>>>,
}

#[derive(Debug, Deserialize)]
struct GlyphFile {
    glyphs: HashMap<String, IconNode>,
    icons: HashMap<String, IconNode>,
    morphs: HashMap<String, BakedMorph>,
}

static GLYPHS: LazyLock<GlyphFile> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../glyphs.generated.json")).expect("glyphs parse")
});

static RESOLVED: LazyLock<HashMap<&'static str, String>> = LazyLock::new(|| {
    TOKENS
        .iter()
        .map(|(name, value)| (*name, css_color(value)))
        .collect()
});

pub fn token(name: &str) -> String {
    RESOLVED
        .get(name)
        .cloned()
        .unwrap_or_else(|| NEUTRAL.to_owned())
}

pub fn category_color(category: &str) -> &'static str {
    lookup(&CATEGORY_COLORS, category)
}

pub fn category_icon(category: &str) -> &'static str {
    CATEGORY_ICONS
        .iter()
        .find(|(name, _)| *name == category)
        .map_or("box", |(_, icon)| icon)
}

pub fn category_order(category: &str) -> usize {
    CATEGORY_COLORS
        .iter()
        .position(|(name, _)| *name == category)
        .unwrap_or(CATEGORY_COLORS.len())
}

pub fn port_color(kind: &str) -> &'static str {
    lookup(&PORT_COLORS, kind)
}

fn lookup(table: &[(&str, &'static str)], key: &str) -> &'static str {
    table
        .iter()
        .find(|(name, _)| *name == key)
        .map_or(NEUTRAL, |(_, color)| color)
}

pub fn style_sheet() -> String {
    let mut sheet = STYLE_TEMPLATE.to_owned();
    let mut names: Vec<&&str> = RESOLVED.keys().collect();
    names.sort_by_key(|name| std::cmp::Reverse(name.len()));
    for name in names {
        sheet = sheet.replace(&format!("@{name}"), &RESOLVED[*name]);
    }
    sheet.replace("@radius", &format!("{RADIUS}px"))
}

pub fn baked_morph(glyph: &str, morph: &str) -> Option<&'static BakedMorph> {
    GLYPHS.morphs.get(&format!("{glyph}>{morph}"))
}

pub fn icon_svg(name: &str, color: &str, stroke: f64) -> String {
    let node = GLYPHS
        .glyphs
        .get(name)
        .or_else(|| GLYPHS.icons.get(name))
        .or_else(|| GLYPHS.glyphs.get(FALLBACK_GLYPH))
        .or_else(|| GLYPHS.icons.get("box"));
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{color}" stroke-width="{stroke}" stroke-linecap="round" stroke-linejoin="round">"#
    );
    for (tag, attrs) in node.into_iter().flatten() {
        let _ = write!(svg, "<{tag}");
        for (key, value) in attrs {
            let value = if value == "currentColor" {
                color
            } else {
                value
            };
            let _ = write!(svg, r#" {key}="{value}""#);
        }
        svg.push_str("/>");
    }
    svg.push_str("</svg>");
    svg
}

fn css_color(text: &str) -> String {
    let Some((lightness, chroma, hue, alpha)) = parse_oklch(text) else {
        return text.to_owned();
    };
    let (red, green, blue) = oklch_to_srgb(lightness, chroma, hue);
    let channel = |v: f64| whole(v.clamp(0.0, 1.0) * 255.0);
    if alpha < 1.0 {
        format!(
            "rgba({}, {}, {}, {})",
            channel(red),
            channel(green),
            channel(blue),
            whole(alpha * 255.0)
        )
    } else {
        format!(
            "#{:02x}{:02x}{:02x}",
            channel(red),
            channel(green),
            channel(blue)
        )
    }
}

fn parse_oklch(text: &str) -> Option<(f64, f64, f64, f64)> {
    let inner = text.strip_prefix("oklch(")?.strip_suffix(')')?;
    let (values, alpha) = match inner.split_once('/') {
        Some((values, alpha)) => (values, percent(alpha.trim())?),
        None => (inner, 1.0),
    };
    let mut parts = values.split_whitespace().map(str::parse::<f64>);
    Some((
        parts.next()?.ok()?,
        parts.next()?.ok()?,
        parts.next()?.ok()?,
        alpha,
    ))
}

fn percent(text: &str) -> Option<f64> {
    text.strip_suffix('%').map_or_else(
        || text.parse().ok(),
        |number| number.parse::<f64>().ok().map(|v| v / 100.0),
    )
}

fn oklch_to_srgb(lightness: f64, chroma: f64, hue: f64) -> (f64, f64, f64) {
    let green_red = chroma * hue.to_radians().cos();
    let blue_yellow = chroma * hue.to_radians().sin();
    let long = 0.215_803_757_3f64.mul_add(
        blue_yellow,
        0.396_337_777_4f64.mul_add(green_red, lightness),
    );
    let medium = (-0.063_854_172_8f64).mul_add(
        blue_yellow,
        (-0.105_561_345_8f64).mul_add(green_red, lightness),
    );
    let short = (-1.291_485_548_0f64).mul_add(
        blue_yellow,
        (-0.089_484_177_5f64).mul_add(green_red, lightness),
    );
    let (long, medium, short) = (long.powi(3), medium.powi(3), short.powi(3));
    let red = 0.230_969_929_2f64.mul_add(
        short,
        4.076_741_662_1f64.mul_add(long, -3.307_711_591_3 * medium),
    );
    let green = (-0.341_319_396_5f64).mul_add(
        short,
        (-1.268_438_004_6f64).mul_add(long, 2.609_757_401_1 * medium),
    );
    let blue = 1.707_614_701_0f64.mul_add(
        short,
        (-0.004_196_086_3f64).mul_add(long, -0.703_418_614_7 * medium),
    );
    (gamma(red), gamma(green), gamma(blue))
}

fn gamma(linear: f64) -> f64 {
    if linear <= 0.003_130_8 {
        12.92 * linear
    } else {
        1.055f64.mul_add(linear.powf(1.0 / 2.4), -0.055)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn channels(color: &str) -> (i64, i64, i64) {
        let hex = color.strip_prefix('#').expect("an opaque colour");
        let channel = |at: usize| i64::from_str_radix(&hex[at..at + 2], 16).expect("hex");
        (channel(0), channel(2), channel(4))
    }

    fn enum_names(schema: &serde_json::Value, names: &mut Vec<String>) {
        match schema {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    match (key.as_str(), value) {
                        ("enum", serde_json::Value::Array(items)) => names.extend(
                            items
                                .iter()
                                .filter_map(|item| item.as_str().map(str::to_owned)),
                        ),
                        ("const", serde_json::Value::String(name)) => names.push(name.clone()),
                        _ => enum_names(value, names),
                    }
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    enum_names(item, names);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn every_card_icon_is_bundled() {
        let schema = serde_json::to_value(schemars::schema_for!(opsis_types::cards::Icon))
            .expect("the icon schema");
        let mut names = Vec::new();
        enum_names(&schema, &mut names);
        assert!(!names.is_empty(), "no icon names in {schema}");
        let missing: Vec<&String> = names
            .iter()
            .filter(|name| !GLYPHS.icons.contains_key(name.as_str()))
            .collect();
        assert!(
            missing.is_empty(),
            "not bundled: {missing:?} (run bun run glyphs:bundle)"
        );
    }

    #[test]
    fn every_style_sheet_token_resolves() {
        let sheet = style_sheet();
        let unresolved: Vec<&str> = sheet
            .match_indices('@')
            .map(|(at, _)| &sheet[at..sheet.len().min(at + 24)])
            .collect();
        assert!(unresolved.is_empty(), "{unresolved:?}");
    }

    proptest! {
        #[test]
        fn greys_stay_grey_and_lighter_reads_lighter(
            lightness in 0.0f64..1.0,
            step in 0.01f64..0.2,
        ) {
            let darker = channels(&css_color(&format!("oklch({lightness} 0 0)")));
            let lighter = channels(&css_color(&format!("oklch({} 0 0)", (lightness + step).min(1.0))));
            prop_assert_eq!(darker.0, darker.1);
            prop_assert_eq!(darker.1, darker.2);
            prop_assert!(lighter.0 >= darker.0);
        }
    }
}
