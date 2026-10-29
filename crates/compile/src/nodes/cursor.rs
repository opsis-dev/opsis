use opsis_macros::settings;
use opsis_types::cards::{CardRow, Icon};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::recording::{
    AnnotationKind, AnnotationSpec, CursorLook, HighlightLook, KeystrokeLook, KeystrokePosition,
    RecordingProps, RippleLook,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::card_rows::{LABELLED_TEXT_MAX, join_parts, quote, quoted, toggle_row};
use crate::color::color_or;
use crate::fonts::font_prop;
use crate::node::{CardCtx, Category, NodeDef, RowExt, row};
use crate::params::Params;

pub const DEFAULT_CURSOR_FILL: &str = "#111418";
pub const DEFAULT_CURSOR_EDGE: &str = "#ffffff";
pub const DEFAULT_RIPPLE_COLOR: &str = "#0f172a";
pub const DEFAULT_RIPPLE_SIZE: f64 = 1.0;
pub const DEFAULT_RIPPLE_WIDTH: f64 = 2.0;
pub const DEFAULT_KEYCAP: &str = "#0a0a0a";
pub const DEFAULT_KEYCAP_INK: &str = "#ffffff";
pub const CURSOR_STYLE_TYPE: &str = "cursor.style";

pub fn default_cursor_look() -> CursorLook {
    CursorLook::Arrow {
        fill: DEFAULT_CURSOR_FILL.to_owned(),
        edge: DEFAULT_CURSOR_EDGE.to_owned(),
    }
}

pub fn default_ripple() -> RippleLook {
    RippleLook {
        color: DEFAULT_RIPPLE_COLOR.to_owned(),
        size: DEFAULT_RIPPLE_SIZE,
        width: DEFAULT_RIPPLE_WIDTH,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CursorShape {
    Arrow,
    Mac,
}

impl CursorShape {
    fn label(self) -> &'static str {
        match self {
            Self::Arrow => "Simple arrow",
            Self::Mac => "Mac",
        }
    }
}

#[settings]
#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct CursorStyle {
    #[setting("Show the cursor", default = true)]
    pub visible: bool,
    #[setting(
        "Look",
        default = CursorShape::Arrow,
        labels = { "arrow": "Simple arrow", "mac": "Mac" },
        help = "Mac turns into a pointing hand over links and buttons and a text cursor over fields, as the page asks."
    )]
    pub look: CursorShape,
    #[setting(
        "Size",
        default = 1.0,
        range(min = 0.4, max = 3),
        step = 0.05,
        unit = "×"
    )]
    pub size: f64,
    #[setting(
        "Colour",
        default = "#111418",
        widget = "color",
        zero_label = "Default",
        show_if = { "look": ["arrow"] },
        check = colour_or_empty
    )]
    pub color: String,
    #[setting(
        "Outline colour",
        default = "#ffffff",
        widget = "color",
        zero_label = "Default",
        show_if = { "look": ["arrow"] },
        check = colour_or_empty
    )]
    pub outline: String,
}

impl Params for CursorStyle {}

#[settings]
#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ClickRipple {
    #[setting("Ripple on every click", default = true)]
    pub enabled: bool,
    #[setting(
        "Colour",
        default = "#0f172a",
        widget = "color",
        zero_label = "Default",
        check = colour_or_empty
    )]
    pub color: String,
    #[setting(
        "Size",
        default = 1.0,
        range(min = 0.3, max = 4),
        step = 0.05,
        unit = "×"
    )]
    pub size: f64,
    #[setting(
        "Ring width",
        default = 2.0,
        range(min = 0.5, max = 12),
        step = 0.5,
        unit = "px",
        advanced
    )]
    pub width: f64,
}

impl Params for ClickRipple {}

#[settings]
#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct KeystrokeChips {
    #[setting(
        "Position",
        default = KeystrokePosition::Bottom,
        labels = { "bottom": "Bottom", "top": "Top" }
    )]
    pub position: KeystrokePosition,
    #[setting("Show typed text", default = true)]
    pub typed: bool,
    #[setting("Show shortcuts", default = true)]
    pub chords: bool,
    #[setting(
        "Keycap colour",
        default = "#0a0a0a",
        widget = "color",
        zero_label = "Default",
        check = colour_or_empty
    )]
    pub keycap: String,
    #[setting(
        "Text colour",
        default = "#ffffff",
        widget = "color",
        zero_label = "Default",
        check = colour_or_empty
    )]
    pub ink: String,
    #[setting(
        "Size",
        default = 1.0,
        range(min = 0.4, max = 3),
        step = 0.05,
        unit = "×"
    )]
    pub size: f64,
}

impl Params for KeystrokeChips {}

fn look_label(look: HighlightLook) -> &'static str {
    match look {
        HighlightLook::Clean => "Clean",
        HighlightLook::Box => "Hand-drawn box",
        HighlightLook::Circle => "Circle",
        HighlightLook::Underline => "Underline",
        HighlightLook::Marker => "Marker",
    }
}

pub struct AnnotationFields<'a> {
    pub match_text: &'a str,
    pub text: &'a str,
    pub color: &'a str,
    pub padding: u32,
    pub lead_ms: u32,
    pub hold_ms: u32,
    pub look: HighlightLook,
    pub roughness: f64,
    pub text_color: Option<&'a str>,
    pub font_size: Option<u32>,
    pub font: Option<&'a str>,
}

pub trait Annotation: Params {
    const KIND: AnnotationKind;
    const ICON: Icon;
    const COLOR: &'static str;
    const TEXT_COLOR: Option<&'static str> = None;

    fn fields(&self) -> AnnotationFields<'_>;
}

macro_rules! annotation_params {
    ($name:ident, look_hidden = $hidden:literal, color = $color:literal) => {
        #[settings]
        #[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
        #[serde(rename_all = "camelCase", default)]
        pub struct $name {
            #[serde(rename = "match")]
            #[setting(
                "Show on actions named",
                help = "Leave empty to show on every click, typing, hover and drag. Matches part of an action's name."
            )]
            pub match_text: String,
            #[setting("Text", hidden)]
            pub text: String,
            #[setting(
                "Colour",
                default = $color,
                widget = "color",
                zero_label = "Default",
                check = colour_or_empty
            )]
            pub color: String,
            #[setting("Space around", default = 10, range(min = 0, max = 80), unit = "px")]
            pub padding: u32,
            #[setting(
                "Appear early by",
                default = 250,
                range(min = 0, max = 5000),
                step = 50,
                unit = "ms"
            )]
            pub lead_ms: u32,
            #[setting(
                "Stay for",
                default = 1400,
                range(min = 0, max = 10000),
                step = 50,
                unit = "ms"
            )]
            pub hold_ms: u32,
            #[setting(
                "Look",
                default = HighlightLook::Clean,
                labels = { "clean": "Clean", "box": "Hand-drawn box", "circle": "Circle", "underline": "Underline", "marker": "Marker" },
                help = "The hand-drawn looks draw themselves in as the action starts.",
                hidden = $hidden
            )]
            pub look: HighlightLook,
            #[setting(
                "Sketchiness",
                default = 1.0,
                range(min = 0, max = 3),
                step = 0.1,
                advanced,
                hidden = $hidden,
                show_if = { "look": ["box", "circle", "underline", "marker"] }
            )]
            pub roughness: f64,
        }
    };
}

annotation_params!(Spotlight, look_hidden = true, color = "rgba(0,0,0,0.55)");
annotation_params!(HighlightBox, look_hidden = false, color = "#a78bfa");

impl Params for Spotlight {}
impl Params for HighlightBox {}

fn plain_fields<'a>(
    match_text: &'a str,
    text: &'a str,
    color: &'a str,
    (padding, lead_ms, hold_ms): (u32, u32, u32),
    (look, roughness): (HighlightLook, f64),
) -> AnnotationFields<'a> {
    AnnotationFields {
        match_text,
        text,
        color,
        padding,
        lead_ms,
        hold_ms,
        look,
        roughness,
        text_color: None,
        font_size: None,
        font: None,
    }
}

impl Annotation for Spotlight {
    const KIND: AnnotationKind = AnnotationKind::Spotlight;
    const ICON: Icon = Icon::Flashlight;
    const COLOR: &'static str = "rgba(0,0,0,0.55)";

    fn fields(&self) -> AnnotationFields<'_> {
        plain_fields(
            &self.match_text,
            &self.text,
            &self.color,
            (self.padding, self.lead_ms, self.hold_ms),
            (self.look, self.roughness),
        )
    }
}

impl Annotation for HighlightBox {
    const KIND: AnnotationKind = AnnotationKind::Highlight;
    const ICON: Icon = Icon::SquareDashed;
    const COLOR: &'static str = "#a78bfa";

    fn fields(&self) -> AnnotationFields<'_> {
        plain_fields(
            &self.match_text,
            &self.text,
            &self.color,
            (self.padding, self.lead_ms, self.hold_ms),
            (self.look, self.roughness),
        )
    }
}

fn annotation_rows(f: &AnnotationFields<'_>, icon: Icon) -> Vec<Option<CardRow>> {
    let on = if f.match_text.trim().is_empty() {
        "On every action".to_owned()
    } else {
        format!(
            "On actions named {}",
            quote(f.match_text, LABELLED_TEXT_MAX)
        )
    };
    let look = (f.look != HighlightLook::Clean).then(|| look_label(f.look).to_owned());
    vec![
        Some(row(icon, join_parts([Some(on), look]))),
        (!f.text.trim().is_empty()).then(|| row(Icon::Type, quoted(f.text))),
    ]
}

fn annotation_spec<A: Annotation>(p: &A) -> AnnotationSpec {
    let f = p.fields();
    AnnotationSpec {
        kind: A::KIND,
        look: Some(f.look),
        roughness: Some(f.roughness),
        match_text: f.match_text.to_owned(),
        text: f.text.to_owned(),
        color: color_or(f.color, A::COLOR),
        padding: f64::from(f.padding),
        lead_ms: f64::from(f.lead_ms),
        hold_ms: f64::from(f.hold_ms),
        text_color: match (A::TEXT_COLOR, f.text_color) {
            (Some(fallback), Some(color)) => Some(color_or(color, fallback)),
            (None, Some(color)) => Some(color.to_owned()),
            _ => None,
        },
        font_size: f.font_size.map(f64::from),
        font: f.font.and_then(font_prop),
    }
}

pub fn annotation<A: Annotation>(
    kind: &'static str,
    title: &str,
    category: Category,
    description: &str,
) -> NodeDef {
    NodeDef::style_modifier::<A>(kind, title, category)
        .describe(description)
        .card(|ctx: &CardCtx<'_, A>| annotation_rows(&ctx.params.fields(), A::ICON))
        .restyle(|mut props: RecordingProps, p: &A, _| {
            props.annotations.push(annotation_spec(p));
            props
        })
}

fn cursor_style() -> NodeDef {
    NodeDef::style_modifier::<CursorStyle>(CURSOR_STYLE_TYPE, "Cursor", Category::Cursor)
        .describe("Shows or hides the drawn cursor and sets its look, size and colours.")
        .card(|ctx: &CardCtx<'_, CursorStyle>| {
            let p = &ctx.params;
            let text = if p.visible {
                format!("{} · {}×", p.look.label(), p.size)
            } else {
                "Hidden".to_owned()
            };
            let shown = row(Icon::MousePointer, text).muted_if(!p.visible);
            let shown = if p.visible && p.look == CursorShape::Arrow {
                shown.swatches(vec![
                    color_or(&p.color, DEFAULT_CURSOR_FILL),
                    color_or(&p.outline, DEFAULT_CURSOR_EDGE),
                ])
            } else {
                shown
            };
            vec![Some(shown)]
        })
        .restyle(|props: RecordingProps, p: &CursorStyle, _| RecordingProps {
            cursor: p.visible,
            cursor_scale: p.size,
            cursor_look: match p.look {
                CursorShape::Mac => CursorLook::Mac,
                CursorShape::Arrow => CursorLook::Arrow {
                    fill: color_or(&p.color, DEFAULT_CURSOR_FILL),
                    edge: color_or(&p.outline, DEFAULT_CURSOR_EDGE),
                },
            },
            ..props
        })
}

fn click_ripple() -> NodeDef {
    NodeDef::style_modifier::<ClickRipple>("cursor.clickRipple", "Click ripple", Category::Cursor)
        .describe("A ring that pulses on every click.")
        .card(|ctx: &CardCtx<'_, ClickRipple>| {
            let p = &ctx.params;
            vec![
                Some(toggle_row(Icon::CircleDot, p.enabled)),
                p.enabled.then(|| {
                    row(Icon::Palette, format!("{}×", p.size))
                        .swatches(vec![color_or(&p.color, DEFAULT_RIPPLE_COLOR)])
                }),
            ]
        })
        .restyle(|props: RecordingProps, p: &ClickRipple, _| RecordingProps {
            ripple: p.enabled.then(|| RippleLook {
                color: color_or(&p.color, DEFAULT_RIPPLE_COLOR),
                size: p.size,
                width: p.width,
            }),
            ..props
        })
}

fn keystroke_chips() -> NodeDef {
    NodeDef::style_modifier::<KeystrokeChips>(
        "cursor.keystrokeChips",
        "Keystroke chips",
        Category::Cursor,
    )
    .describe("Shows keys and typed text as keycaps on screen.")
    .card(|ctx: &CardCtx<'_, KeystrokeChips>| {
        let p = &ctx.params;
        let position = match p.position {
            KeystrokePosition::Bottom => "Bottom",
            KeystrokePosition::Top => "Top",
        };
        vec![Some(row(
            Icon::Keyboard,
            join_parts([
                Some(position),
                p.typed.then_some("typed text"),
                p.chords.then_some("shortcuts"),
            ]),
        ))]
    })
    .restyle(
        |props: RecordingProps, p: &KeystrokeChips, _| RecordingProps {
            keystrokes: Some(KeystrokeLook {
                position: p.position,
                typed: p.typed,
                chords: p.chords,
                keycap: color_or(&p.keycap, DEFAULT_KEYCAP),
                ink: color_or(&p.ink, DEFAULT_KEYCAP_INK),
                size: p.size,
            }),
            ..props
        },
    )
}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        cursor_style().look(NodeLook::of("lucide:mouse-pointer-2")),
        click_ripple().look(NodeLook::of("lucide:circle-dot-dashed").moves(NodeMotion::Ripple)),
        keystroke_chips().look(NodeLook::of("lucide:keyboard")),
        annotation::<Spotlight>(
            "cursor.spotlight",
            "Spotlight",
            Category::Cursor,
            "Darkens everything except the element being used.",
        )
        .look(NodeLook::of("lucide:spotlight")),
        annotation::<HighlightBox>(
            "cursor.highlightBox",
            "Highlight box",
            Category::Cursor,
            "Draws a box around the element being used, cleanly or by hand.",
        )
        .look(NodeLook::of("lucide:square-dashed").moves(NodeMotion::March)),
    ]
}
