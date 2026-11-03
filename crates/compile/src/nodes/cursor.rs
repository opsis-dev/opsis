use opsis_macros::{choice, settings};
use opsis_types::cards::{CardRow, Icon};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::recording::{
    AnnotationKind, AnnotationLook, AnnotationSpec, CursorFeel, CursorLook, HighlightLook,
    KeyNames, KeystrokeLook, KeystrokePosition, LabelLook, PointerProps, RecordingProps,
    RippleLook,
};

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
        ring_opacity: None,
        fill_opacity: None,
        duration_s: None,
    }
}

#[choice]
pub enum CursorShape {
    #[label("Simple arrow")]
    Arrow,
    #[label("Mac")]
    Mac,
}

#[settings]
pub struct CursorStyle {
    #[setting("Show the cursor", default = true)]
    pub visible: bool,
    #[setting(
        "Look",
        default = CursorShape::Arrow,
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
        check = colour_or_empty,
        advanced
    )]
    pub outline: String,
    #[setting(
        "Grow with the zoom",
        default = 0.58,
        range(min = 0, max = 1),
        step = 0.02,
        help = "0 keeps the cursor the same size on screen; 1 grows it with the page.",
        advanced
    )]
    pub grow_with_zoom: f64,
    #[setting(
        "Outline width",
        default = 1.85,
        range(min = 0, max = 6),
        step = 0.05,
        show_if = { "look": ["arrow"] },
        advanced
    )]
    pub outline_width: f64,
    #[setting(
        "Shadow",
        default = 0.34,
        range(min = 0, max = 1),
        step = 0.02,
        advanced
    )]
    pub shadow: f64,
    #[setting(
        "Press squash",
        default = 0.9,
        range(min = 0.5, max = 1),
        step = 0.01,
        help = "How small the cursor gets while pressing.",
        advanced
    )]
    pub press_squash: f64,
    #[setting("Hide while typing", default = true, advanced)]
    pub hide_while_typing: bool,
}

#[settings]
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
    #[setting(
        "Ring strength",
        default = 47,
        range(min = 0, max = 100),
        step = 1,
        unit = "%",
        advanced
    )]
    pub ring_opacity: u32,
    #[setting(
        "Fill strength",
        default = 9,
        range(min = 0, max = 100),
        step = 1,
        unit = "%",
        advanced
    )]
    pub fill_opacity: u32,
    #[setting(
        "Ripple time",
        default = 350,
        range(min = 50, max = 3000),
        step = 10,
        unit = "ms",
        advanced
    )]
    pub duration_ms: u32,
}

#[settings]
pub struct KeystrokeChips {
    #[setting(
        "Position",
        default = KeystrokePosition::Bottom
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
    #[setting(
        "Shortcut shown for",
        default = 1100,
        range(min = 100, max = 10000),
        step = 50,
        unit = "ms",
        advanced
    )]
    pub chord_hold_ms: u32,
    #[setting(
        "Typed text stays for",
        default = 600,
        range(min = 0, max = 10000),
        step = 50,
        unit = "ms",
        advanced
    )]
    pub typed_hold_ms: u32,
    #[setting(
        "Distance from the edge",
        default = 56,
        range(min = 0, max = 400),
        step = 1,
        unit = "px",
        advanced
    )]
    pub edge: u32,
    #[setting(
        "Keycap corners",
        default = 26,
        range(min = 0, max = 80),
        step = 1,
        unit = "px",
        advanced
    )]
    pub radius: u32,
    #[setting("Key names", advanced)]
    pub key_names: KeyNames,
    #[setting("Font", widget = "font", zero_label = "Project fonts", advanced)]
    pub font: String,
    #[setting(
        "Typed text shadow",
        default = 0.35,
        range(min = 0, max = 1),
        step = 0.01,
        show_if = { "typed": ["true"] },
        advanced
    )]
    pub shadow: f64,
    #[setting("Blinking caret", default = true, show_if = { "typed": ["true"] }, advanced)]
    pub caret: bool,
}

#[derive(Debug)]
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
    pub label_look: Option<LabelLook>,
    pub mark_look: Option<AnnotationLook>,
}

pub trait Annotation: Params {
    const KIND: AnnotationKind;
    const ICON: Icon;
    const COLOR: &'static str;
    const TEXT_COLOR: Option<&'static str> = None;

    fn fields(&self) -> AnnotationFields<'_>;
}

macro_rules! annotation_params {
    ($name:ident [$($options:ident),*], color = $color:literal, radius = $radius:literal, { $($extra:tt)* }) => {
        #[settings($($options),*)]
        pub struct $name {
            #[serde(rename = "match")]
            #[setting(
                "Show on actions named",
                help = "Leave empty to show on every click, typing, hover and drag. Matches part of an action's name."
            )]
            pub match_text: String,
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
                unit = "ms",
                advanced
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
            #[setting("Fade", default = 180, range(min = 0, max = 2000), step = 10, unit = "ms", advanced)]
            pub fade_ms: u32,
            #[setting("Corner radius", default = $radius, range(min = 0, max = 80), unit = "px", advanced)]
            pub corner_radius: u32,
            $($extra)*
        }
    };
}

annotation_params!(Spotlight [eq], color = "rgba(0,0,0,0.55)", radius = 12, {
    #[setting("Soft edge", range(min = 0, max = 200), unit = "px", advanced)]
    pub softness: u32,
});
annotation_params!(HighlightBox [], color = "#a78bfa", radius = 10, {
    #[setting(
        "Look",
        default = HighlightLook::Clean,
        help = "The hand-drawn looks draw themselves in as the action starts."
    )]
    pub look: HighlightLook,
    #[setting(
        "Sketchiness",
        default = 1.0,
        range(min = 0, max = 3),
        step = 0.1,
        advanced,
        show_if = { "look": ["box", "circle", "underline", "marker"] }
    )]
    pub roughness: f64,
    #[setting("Line width", default = 3, range(min = 1, max = 20), unit = "px", advanced)]
    pub line_width: u32,
    #[setting("Fill", default = 12, range(min = 0, max = 100), unit = "%", advanced)]
    pub fill_opacity: u32,
    #[setting(
        "Marker strength",
        default = 50,
        range(min = 0, max = 100),
        unit = "%",
        show_if = { "look": ["marker"] },
        advanced
    )]
    pub marker_opacity: u32,
    #[setting(
        "Draw-in time",
        default = 450,
        range(min = 0, max = 3000),
        step = 10,
        unit = "ms",
        show_if = { "look": ["box", "circle", "underline", "marker"] },
        advanced
    )]
    pub draw_ms: u32,
    #[setting(
        "Strokes",
        default = 2,
        range(min = 1, max = 5),
        show_if = { "look": ["box", "circle", "underline", "marker"] },
        advanced
    )]
    pub strokes: u32,
});

const fn plain_fields<'a>(
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
        label_look: None,
        mark_look: None,
    }
}

impl Annotation for Spotlight {
    const KIND: AnnotationKind = AnnotationKind::Spotlight;
    const ICON: Icon = Icon::Flashlight;
    const COLOR: &'static str = "rgba(0,0,0,0.55)";

    fn fields(&self) -> AnnotationFields<'_> {
        AnnotationFields {
            mark_look: Some(AnnotationLook {
                fade_s: f64::from(self.fade_ms) / 1000.0,
                radius: f64::from(self.corner_radius),
                softness: Some(f64::from(self.softness)),
                line_width: None,
                fill_opacity: None,
                marker_opacity: None,
                draw_s: None,
                strokes: None,
            }),
            ..plain_fields(
                &self.match_text,
                "",
                &self.color,
                (self.padding, self.lead_ms, self.hold_ms),
                (HighlightLook::Clean, 1.0),
            )
        }
    }
}

impl Annotation for HighlightBox {
    const KIND: AnnotationKind = AnnotationKind::Highlight;
    const ICON: Icon = Icon::SquareDashed;
    const COLOR: &'static str = "#a78bfa";

    fn fields(&self) -> AnnotationFields<'_> {
        AnnotationFields {
            mark_look: Some(AnnotationLook {
                fade_s: f64::from(self.fade_ms) / 1000.0,
                radius: f64::from(self.corner_radius),
                softness: None,
                line_width: Some(f64::from(self.line_width)),
                fill_opacity: Some(f64::from(self.fill_opacity) / 100.0),
                marker_opacity: Some(f64::from(self.marker_opacity) / 100.0),
                draw_s: Some(f64::from(self.draw_ms) / 1000.0),
                strokes: Some(f64::from(self.strokes)),
            }),
            ..plain_fields(
                &self.match_text,
                "",
                &self.color,
                (self.padding, self.lead_ms, self.hold_ms),
                (self.look, self.roughness),
            )
        }
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
    let look = (f.look != HighlightLook::Clean).then(|| f.look.label().to_owned());
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
        label_look: f.label_look,
        mark_look: f.mark_look,
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
            pointer: PointerProps {
                shown: p.visible,
                scale: p.size,
                feel: Some(CursorFeel {
                    grow_with_zoom: p.grow_with_zoom,
                    outline_width: p.outline_width,
                    shadow: p.shadow,
                    press_squash: p.press_squash,
                    hide_while_typing: p.hide_while_typing,
                }),
                look: match p.look {
                    CursorShape::Mac => CursorLook::Mac,
                    CursorShape::Arrow => CursorLook::Arrow {
                        fill: color_or(&p.color, DEFAULT_CURSOR_FILL),
                        edge: color_or(&p.outline, DEFAULT_CURSOR_EDGE),
                    },
                },
                ..props.pointer
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
            pointer: PointerProps {
                ripple: p.enabled.then(|| RippleLook {
                    color: color_or(&p.color, DEFAULT_RIPPLE_COLOR),
                    size: p.size,
                    width: p.width,
                    ring_opacity: Some(f64::from(p.ring_opacity) / 100.0),
                    fill_opacity: Some(f64::from(p.fill_opacity) / 100.0),
                    duration_s: Some(f64::from(p.duration_ms) / 1000.0),
                }),
                ..props.pointer
            },
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
        let position = p.position.label();
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
                chord_hold_s: Some(f64::from(p.chord_hold_ms) / 1000.0),
                typed_hold_s: Some(f64::from(p.typed_hold_ms) / 1000.0),
                edge: Some(f64::from(p.edge)),
                radius: Some(f64::from(p.radius)),
                key_names: Some(p.key_names),
                font: font_prop(&p.font),
                shadow: Some(p.shadow),
                caret: Some(p.caret),
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
