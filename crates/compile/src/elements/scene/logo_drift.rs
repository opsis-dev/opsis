use opsis_macros::settings;
use opsis_scene::css::{font_family, with_background};
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::{Filter, Transform};
use opsis_scene::random::js_number_text;
use opsis_scene::style::{
    Align, Display, FlexDirection, FlexWrap, LineHeight, Overflow, Position, Style, TextRendering,
    WhiteSpace,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::anim::clamped;
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::oklch_mix::with_alpha;
use crate::elements::shared::FontWeight;
use crate::elements::theme::{Theme, warm_themes};
use crate::elements::warm_theme::{resolve_font, warm_theme};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct LogoDriftTile {
    #[setting("Mark", default = "Ab", help = "One to three characters.")]
    pub glyph: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Label", "type" = "string")]
    pub label: Option<String>,
    #[setting("Background", default = "#1f2430", check = colour)]
    pub background: String,
    #[setting("Text colour", widget = "color", zero_label = "Default")]
    pub color: String,
    #[setting("Start across", range(min = -1200, max = 1200), step = 1, advanced)]
    pub x: i32,
    #[setting("Start down", range(min = -800, max = 800), step = 1, advanced)]
    pub y: i32,
    #[setting("Drift across", range(min = -200, max = 200), step = 1, advanced)]
    pub vx: i32,
    #[setting("Drift down", range(min = -200, max = 200), step = 1, advanced)]
    pub vy: i32,
    #[setting("Size", default = 140, range(min = 20, max = 400), step = 1, advanced)]
    pub size: u32,
    #[setting(
        "Appears at",
        default = 1.0,
        range(min = 0, max = 10),
        step = 0.01,
        advanced
    )]
    pub at: f64,
}

#[settings(export)]
pub struct LogoDrift {
    #[setting("Headline", default = "Built for the stack you already use.")]
    pub headline: String,
    #[setting(
        "Pull-back / sec",
        default = 0.1762,
        range(min = 0, max = 0.4),
        step = 0.001,
        advanced
    )]
    pub pullback: f64,
    #[setting("Drift", default = 1.0, range(min = 0, max = 3), step = 0.05)]
    pub tile_speed: f64,
    #[setting("Tile size", default = 1.0, range(min = 0.3, max = 2), step = 0.05)]
    pub tile_scale: f64,
    #[setting(
        "Tile fade τ (s)",
        default = 0.4,
        range(min = 0.05, max = 2),
        step = 0.01,
        advanced
    )]
    pub tile_fade: f64,
    #[setting(
        "Tile radius",
        default = 0.02,
        range(min = 0, max = 0.5),
        step = 0.01,
        advanced
    )]
    pub tile_radius: f64,
    #[setting(
        "Glyph size",
        default = 0.34,
        range(min = 0.1, max = 0.7),
        step = 0.01,
        advanced
    )]
    pub glyph_scale: f64,
    #[setting(
        "Word stagger (s)",
        default = 0.15,
        range(min = 0, max = 0.6),
        step = 0.005,
        advanced
    )]
    pub word_stagger: f64,
    #[setting(
        "Word lands in (s)",
        default = 0.14,
        range(min = 0.02, max = 1),
        step = 0.005,
        advanced
    )]
    pub word_duration: f64,
    #[setting(
        "Word arrives at",
        default = 1.216,
        range(min = 0.5, max = 2),
        step = 0.001,
        advanced
    )]
    pub word_scale: f64,
    #[setting(
        "Word blur (px)",
        default = 4.3,
        range(min = 0, max = 20),
        step = 0.1,
        advanced
    )]
    pub word_blur: f64,
    #[setting("Exit at (s)", default = 3.13, range(min = 0.5, max = 10), step = 0.01)]
    pub exit_at: f64,
    #[setting(
        "Exit over (s)",
        default = 0.34,
        range(min = 0.05, max = 2),
        step = 0.01,
        advanced
    )]
    pub exit_duration: f64,
    #[setting("Headline size", default = 20.0, range(min = 8, max = 64), step = 0.5)]
    pub font_size: f64,
    #[setting("Wash", default = true)]
    pub glow: bool,
    #[setting(
        "Wash strength",
        default = 0.07,
        range(min = 0, max = 0.5),
        step = 0.005,
        advanced
    )]
    pub glow_opacity: f64,
    #[setting("Accent", widget = "color", zero_label = "Theme")]
    pub accent_color: String,
    #[setting("Theme", transform = warm_themes, widget = "theme")]
    pub theme: Theme,
    #[setting("Font weight", default = FontWeight::W500)]
    pub font_weight: FontWeight,
    #[setting("Glow colour", widget = "color", zero_label = "Accent", advanced)]
    pub glow_color: String,
    #[setting(
        "Tiles",
        default = vec![
            LogoDriftTile {
                glyph: "Rc".into(),
                label: Some("Record".into()),
                color: "#ffffff".into(),
                x: 163,
                y: 232,
                vx: -54,
                vy: 12,
                size: 109,
                at: 0.28,
                ..Default::default()
            },
            LogoDriftTile {
                glyph: "Ed".into(),
                label: Some("Edit".into()),
                background: "#f2f4f7".into(),
                color: "#16181d".into(),
                x: 128,
                y: -209,
                vx: 81,
                vy: 32,
                size: 122,
                at: 0.48,
            },
            LogoDriftTile {
                glyph: "Zm".into(),
                label: Some("Zoom".into()),
                background: "#10131a".into(),
                color: "#7dd3fc".into(),
                x: -384,
                y: 344,
                vx: -27,
                vy: -68,
                size: 200,
                at: 0.95,
            },
            LogoDriftTile {
                glyph: "Cc".into(),
                label: Some("Captions".into()),
                background: "#e7e8ea".into(),
                color: "#16181d".into(),
                x: 519,
                y: 45,
                vx: -31,
                vy: 64,
                size: 144,
                at: 1.51,
            },
            LogoDriftTile {
                glyph: "Mx".into(),
                label: Some("Mix".into()),
                background: "#f1ece2".into(),
                color: "#8a5a2b".into(),
                x: -692,
                y: -125,
                vx: 41,
                vy: -71,
                size: 214,
                at: 1.55,
            },
        ]
    )]
    pub tiles: Vec<LogoDriftTile>,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project."
    )]
    pub font_family: String,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Headline starts at",
        default = 0.017,
        range(min = 0, max = 10),
        step = 0.01,
        unit = "s",
        advanced
    )]
    pub word_at: f64,
    #[setting(
        "Exit zoom",
        default = 0.8,
        range(min = 0.2, max = 2),
        step = 0.05,
        advanced
    )]
    pub exit_scale: f64,
    #[setting(
        "Exit blur",
        default = 6.5,
        range(min = 0, max = 30),
        step = 0.5,
        unit = "px",
        advanced
    )]
    pub exit_blur: f64,
    #[setting(
        "Wash size",
        default = 1.0,
        range(min = 0.2, max = 3),
        step = 0.05,
        show_if = { "glow": ["true"] },
        advanced
    )]
    pub glow_size: f64,
    #[setting(
        "Mark weight",
        default = 600,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub glyph_weight: u32,
    #[setting(
        "Label size",
        default = 0.1,
        range(min = 0.04, max = 0.3),
        step = 0.01,
        help = "As a share of the tile.",
        advanced
    )]
    pub label_scale: f64,
    #[setting(
        "Label opacity",
        default = 0.72,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub label_opacity: f64,
    #[setting(
        "Headline width",
        default = 88,
        range(min = 20, max = 100),
        step = 1,
        unit = "%",
        advanced
    )]
    pub headline_width: u32,
}

pub fn definition() -> ElementMeta {
    meta::<LogoDrift>(
            element(
                ElementKind::Component,
                "logo-drift",
                "Logo drift",
                "Logo wall animation: a logo cloud of integrations drifting behind a headline. A field of tiles drifting past a headline while the camera pulls steadily back, so the wall of things-it-works-with keeps arriving from every edge.",
                ElementCategory::Scene,
                180,
            )
                .authored_clock()
                .fonts(&["Geist", "Instrument Serif", "Inter", "Montserrat", "Outfit", "Space Grotesk"])
                .look(NodeLook::of("lucide:cloud")),
        )
}

const SANS: &str = "\"Inter\"";
const REF: (f64, f64) = (810.0, 458.0);
const PULLBACK_FLOOR: f64 = 0.02;

struct Placed {
    left: f64,
    top: f64,
    size: f64,
    opacity: f64,
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film places each tile with separate multiplies and adds"
)]
fn place_tile(props: &LogoDrift, tile: &LogoDriftTile, t: f64) -> Placed {
    let k = PULLBACK_FLOOR.max(1.0 - props.pullback * t);
    let wx = f64::from(tile.x) + f64::from(tile.vx) * props.tile_speed * t;
    let wy = f64::from(tile.y) + f64::from(tile.vy) * props.tile_speed * t;
    let size = k * f64::from(tile.size) * props.tile_scale;
    let age = t - tile.at;
    let opacity = if age <= 0.0 {
        0.0
    } else {
        1.0 - (-age / props.tile_fade.max(1e-3)).exp()
    };
    Placed {
        left: REF.0 / 2.0 + k * wx - size / 2.0,
        top: REF.1 / 2.0 + k * wy - size / 2.0,
        size,
        opacity,
    }
}

struct Word {
    scale: f64,
    blur: f64,
    opacity: f64,
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film eases each word with separate multiplies and adds"
)]
fn word_state(props: &LogoDrift, t: f64, index: usize) -> Result<Word, DrawFailure> {
    let start = props.word_at + float(index) * props.word_stagger;
    if t < start {
        return Ok(Word {
            scale: props.word_scale,
            blur: props.word_blur,
            opacity: 0.0,
        });
    }
    let p = interpolate(
        t,
        &[start, start + props.word_duration],
        &[0.0, 1.0],
        &Interpolation {
            easing: Some(Easing::ease_out(Easing::Quad)),
            ..Interpolation::clamped()
        },
    )?;
    let q = clamped(
        t,
        [props.exit_at, props.exit_at + props.exit_duration],
        [0.0, 1.0],
    )?;
    Ok(Word {
        scale: (props.word_scale + (1.0 - props.word_scale) * p)
            * (1.0 + (props.exit_scale - 1.0) * q),
        blur: props.word_blur * (1.0 - p) + props.exit_blur * q,
        opacity: 1.0 - q,
    })
}

const fn or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty() { fallback } else { value }
}

pub fn draw(props: &LogoDrift, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let theme = warm_theme(&props.theme);
    let face = resolve_font(or(&props.font_family, theme.font_family().unwrap_or("")))
        .map_or_else(|| font_family(SANS), |f| font_family(&f));
    let accent = or(&props.accent_color, theme.get("primary"));
    let wash = or(&props.glow_color, accent);
    let now = cx.authored_frame() * props.speed / Cx::authored_fps();
    let stage_scale = (width / REF.0).min(height / REF.1);
    let background = css_color(theme.get("background"));
    let mut stage = div(Style::default()
        .position(Position::Absolute)
        .left(Length::percent(50.0))
        .top(Length::percent(50.0))
        .width(Length::px(REF.0))
        .height(Length::px(REF.1))
        .transform(vec![
            Transform::Translate(Length::percent(-50.0), Length::percent(-50.0), Length::ZERO),
            Transform::Scale(Num(stage_scale), Num(stage_scale), Num(1.0)),
        ])
        .overflow(Overflow::Hidden)
        .background_color(background.clone()));
    if props.glow {
        let n = js_number_text;
        stage = stage.child(div(with_background(
            Style::default()
                .position(Position::Absolute)
                .left(Length::ZERO)
                .top(Length::ZERO)
                .width(Length::px(REF.0))
                .height(Length::px(REF.1)),
            &format!(
                "radial-gradient({}px {}px at 50% 49%, {} 0%, transparent 74%)",
                n(0.45 * props.glow_size * REF.0),
                n(0.34 * props.glow_size * REF.1),
                with_alpha(wash, props.glow_opacity)
            ),
        )));
    }
    for tile in &props.tiles {
        let p = place_tile(props, tile, now);
        if p.opacity <= 0.002 || p.size < 1.0 {
            continue;
        }
        let mut card = div(with_background(
            Style::default()
                .position(Position::Absolute)
                .left(Length::px(p.left))
                .top(Length::px(p.top))
                .width(Length::px(p.size))
                .height(Length::px(p.size))
                .border_radius_all(p.size * props.tile_radius),
            &tile.background,
        )
        .opacity(p.opacity)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .gap(p.size * 0.04)
        .color(css_color(or(&tile.color, theme.get("background"))))
        .font_family(face.clone())
        .overflow(Overflow::Hidden))
        .child(
            span(
                Style::default()
                    .font_size(p.size * props.glyph_scale)
                    .font_weight(f64::from(props.glyph_weight))
                    .line_height(LineHeight::Number(Num(1.0)))
                    .letter_spacing(Length::em(-0.03))
                    .text_rendering(TextRendering::GeometricPrecision),
            )
            .child(tile.glyph.clone()),
        );
        if let Some(label) = tile.label.as_deref().filter(|l| !l.is_empty())
            && p.size > 74.0
        {
            card = card.child(
                span(
                    Style::default()
                        .font_size(p.size * props.label_scale)
                        .font_weight(500.0)
                        .line_height(LineHeight::Number(Num(1.0)))
                        .opacity(props.label_opacity)
                        .text_rendering(TextRendering::GeometricPrecision),
                )
                .child(label.to_owned()),
            );
        }
        stage = stage.child(card);
    }
    let mut words = Vec::new();
    for (i, word) in props.headline.split_whitespace().enumerate() {
        let w = word_state(props, now, i)?;
        let mut style = Style::default()
            .display(Display::InlineBlock)
            .opacity(w.opacity)
            .transform(vec![Transform::Scale(Num(w.scale), Num(w.scale), Num(1.0))]);
        if w.blur > 0.02 {
            style = style.filter(vec![Filter::Blur(Length::px(w.blur))]);
        }
        words.push(span(style).child(word.to_owned()).into());
    }
    let size = props.font_size;
    stage = stage.child(
        div(Style::default()
            .position(Position::Absolute)
            .left(Length::ZERO)
            .top(Length::ZERO)
            .width(Length::px(REF.0))
            .height(Length::px(REF.1))
            .display(Display::Flex)
            .align_items(Align::Center)
            .justify_content(Align::Center))
        .child(
            div(Style::default()
                .display(Display::Flex)
                .flex_wrap(FlexWrap::Wrap)
                .max_width(Length::px(REF.0 * f64::from(props.headline_width) / 100.0))
                .align_items(Align::Baseline)
                .justify_content(Align::Center)
                .column_gap(Length::px(size * 0.28))
                .row_gap(Length::px(size * 0.08))
                .font_family(face)
                .font_size(size)
                .font_weight(props.font_weight.value())
                .line_height(LineHeight::Number(Num(1.2)))
                .letter_spacing(Length::em(-0.012))
                .color(css_color(theme.get("foreground")))
                .white_space(WhiteSpace::Pre))
            .children(words),
        ),
    );
    Ok(div(absolute_fill().background_color(background))
        .child(stage)
        .into())
}
