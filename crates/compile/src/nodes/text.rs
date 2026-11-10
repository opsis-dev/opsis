use std::sync::LazyLock;

use icu_segmenter::WordSegmenter;
use icu_segmenter::options::WordBreakInvariantOptions;
use opsis_macros::settings;
use opsis_types::cards::{Icon, Tone};
use opsis_types::looks::NodeLook;
use opsis_types::recording::{AnnotationKind, HighlightLook, LabelLook};
use opsis_types::values::{Caption, CaptionsValue, CompileValue};
use opsis_types::workflow::PortType;
use regex::Regex;
use serde::Serialize;
use serde_json::Value;

use crate::card_rows::{quoted_row, truncate_card};
use crate::clip::{EMPTY_CLIP_FRAMES, ItemExtra, empty_clip, item_clip, stack_clips};
use crate::elements::overlay::captions::{CAPTIONS_ELEMENT, Captions};
use crate::fonts::font_prop;
use crate::node::{
    CLIP_PORT, CardCtx, Category, Ctx, NodeDef, Phase, RowExt, opt, outputs, port, row,
};
use crate::nodes::cursor::{Annotation, AnnotationFields, annotation};
use crate::params::to_values;
use crate::text::{format_duration, humanize};
use crate::time::{frames_to_ms, ms_to_frames};

const MS_PER_MINUTE: f64 = 60_000.0;
const EMPTY_CAPTIONS_MS: f64 = 1000.0;
const CAPTION_TAIL_MS: f64 = 500.0;
pub const CAPTIONS_PORT: &str = "captions";
pub const CAPTIONS_FROM_SCRIPT_TYPE: &str = "text.captionsFromText";
pub const SHOW_CAPTIONS_TYPE: &str = "text.captions";

static ENDS_SENTENCE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[.!?…。！？]$").expect("pattern"));
static CLOSING_MARKS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"["'”’)\]}»」』]+$"#).expect("pattern"));
static ABBREVIATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:\p{L}\.){2,}$|^(?:mr|mrs|ms|dr|st|vs|etc|inc|ltd|no|fig|approx)\.$")
        .expect("pattern")
});
static UNSPACED_SCRIPT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\p{Han}\p{Hiragana}\p{Katakana}\p{Thai}]").expect("pattern"));
static PARAGRAPH_BREAK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n\s*\n").expect("pattern"));

fn words_of(chunk: &str) -> Vec<String> {
    if !UNSPACED_SCRIPT.is_match(chunk) {
        return vec![chunk.to_owned()];
    }
    let segmenter = WordSegmenter::new_auto(WordBreakInvariantOptions::default());
    let mut words: Vec<String> = Vec::new();
    let mut start = 0;
    for (end, kind) in segmenter.segment_str(chunk).iter_with_word_type() {
        if end == 0 {
            continue;
        }
        let segment = &chunk[start..end];
        start = end;
        match words.last_mut() {
            Some(last) if !kind.is_word_like() => last.push_str(segment),
            _ => words.push(segment.to_owned()),
        }
    }
    words
}

pub fn ends_sentence(word: &str) -> bool {
    let bare = CLOSING_MARKS.replace(word, "");
    ENDS_SENTENCE.is_match(&bare) && !ABBREVIATION.is_match(&bare)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScriptPace {
    pub words_per_minute: f64,
    pub start_ms: f64,
    pub line_pause_ms: f64,
    pub paragraph_pause_ms: f64,
}

pub fn script_to_captions(script: &str, pace: ScriptPace) -> Vec<Caption> {
    let per_word = MS_PER_MINUTE / pace.words_per_minute;
    let mut captions = Vec::new();
    let mut t = pace.start_ms;
    for paragraph in PARAGRAPH_BREAK.split(script) {
        let words: Vec<String> = paragraph.split_whitespace().flat_map(words_of).collect();
        let count = words.len();
        for (i, word) in words.into_iter().enumerate() {
            let pause = if i + 1 == count {
                pace.paragraph_pause_ms
            } else if ends_sentence(&word) {
                pace.line_pause_ms
            } else {
                0.0
            };
            captions.push(Caption {
                text: word,
                start_ms: t,
                end_ms: t + per_word,
            });
            t += per_word + pause;
        }
    }
    captions
}

macro_rules! labelled_annotation {
    ($name:ident, look = $look:ty, text = $text:literal, color = $color:literal, font_size = $size:literal $(, text_color = $text_color:literal)?) => {
        #[settings]
        pub struct $name {
            #[serde(rename = "match")]
            #[setting(
                "Show on actions named",
                help = "Leave empty to show on every click, typing, hover and drag. Matches part of an action's name."
            )]
            pub match_text: String,
            #[setting(
                "Text",
                default = $text,
                check = { "rule": "not-blank", "message": "Add the text to show" }
            )]
            pub text: String,
            #[setting(
                "Colour",
                default = $color,
                widget = "color",
                zero_label = "Default",
                check = colour_or_empty
            )]
            pub color: String,
            #[setting("Space around", default = 10, range(min = 0, max = 80), unit = "px", advanced)]
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
            $(
                #[setting(
                    "Text colour",
                    default = $text_color,
                    widget = "color",
                    zero_label = "Default",
                    check = colour_or_empty
                )]
                pub text_color: String,
            )?
            #[setting("Font size", default = $size, range(min = 8, max = 160), unit = "px")]
            pub font_size: u32,
            #[setting(
                "Font",
                widget = "font",
                help = "Project font follows the fonts set for the whole project.",
                advanced
            )]
            pub font: String,
            #[setting(
                "Look",
                default = HighlightLook::Clean,
                help = "The hand-drawn looks draw themselves in as the action starts.",
                hidden
            )]
            pub look: HighlightLook,
            #[setting(
                "Sketchiness",
                default = 1.0,
                range(min = 0, max = 3),
                step = 0.1,
                advanced,
                hidden,
                show_if = { "look": ["box", "circle", "underline", "marker"] }
            )]
            pub roughness: f64,
            #[serde(flatten)]
            pub style: $look,
        }
    };
}

#[settings(eq)]
pub struct TooltipLook {
    #[setting(
        "Font weight",
        default = 600,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub font_weight: u32,
    #[setting(
        "Space inside",
        default = 14,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub box_padding: u32,
    #[setting(
        "Rounded corners",
        default = 10,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub radius: u32,
    #[setting(
        "Pointer size",
        default = 8,
        range(min = 0, max = 30),
        step = 1,
        unit = "px",
        advanced
    )]
    pub pointer_size: u32,
    #[setting("Shadow", default = true, advanced)]
    pub shadow: bool,
}

#[settings]
pub struct CalloutLook {
    #[setting(
        "Font weight",
        default = 800,
        range(min = 100, max = 900),
        step = 100,
        advanced
    )]
    pub font_weight: u32,
    #[setting(
        "Arrow size",
        default = 1.0,
        range(min = 0.3, max = 3),
        step = 0.05,
        unit = "×",
        advanced
    )]
    pub arrow_size: f64,
    #[setting(
        "Line width",
        default = 5,
        range(min = 1, max = 20),
        step = 1,
        unit = "px",
        advanced
    )]
    pub line_width: u32,
    #[setting("Tilt", default = -4.0, range(min = -30, max = 30), step = 0.5, unit = "°", advanced)]
    pub tilt: f64,
}

impl TooltipLook {
    fn label(&self) -> LabelLook {
        let callout = CalloutLook::default();
        LabelLook {
            font_weight: f64::from(self.font_weight),
            box_padding: f64::from(self.box_padding),
            radius: f64::from(self.radius),
            pointer_size: f64::from(self.pointer_size),
            shadow: self.shadow,
            arrow_size: callout.arrow_size,
            line_width: f64::from(callout.line_width),
            tilt: callout.tilt,
        }
    }
}

impl CalloutLook {
    fn label(&self) -> LabelLook {
        let tooltip = TooltipLook::default();
        LabelLook {
            font_weight: f64::from(self.font_weight),
            box_padding: f64::from(tooltip.box_padding),
            radius: f64::from(tooltip.radius),
            pointer_size: f64::from(tooltip.pointer_size),
            shadow: false,
            arrow_size: self.arrow_size,
            line_width: f64::from(self.line_width),
            tilt: self.tilt,
        }
    }
}

labelled_annotation!(
    Tooltip,
    look = TooltipLook,
    text = "Click to continue",
    color = "#18181b",
    font_size = 18,
    text_color = "#fafafa"
);
labelled_annotation!(
    CalloutArrow,
    look = CalloutLook,
    text = "New!",
    color = "#f43f5e",
    font_size = 30
);

impl Annotation for Tooltip {
    const KIND: AnnotationKind = AnnotationKind::Tooltip;
    const ICON: Icon = Icon::MessageSquare;
    const COLOR: &'static str = "#18181b";
    const TEXT_COLOR: Option<&'static str> = Some("#fafafa");

    fn fields(&self) -> AnnotationFields<'_> {
        AnnotationFields {
            match_text: &self.match_text,
            text: &self.text,
            color: &self.color,
            padding: self.padding,
            lead_ms: self.lead_ms,
            hold_ms: self.hold_ms,
            look: self.look,
            roughness: self.roughness,
            text_color: Some(&self.text_color),
            font_size: Some(self.font_size),
            font: Some(&self.font),
            label_look: Some(self.style.label()),
            mark_look: None,
        }
    }
}

impl Annotation for CalloutArrow {
    const KIND: AnnotationKind = AnnotationKind::Callout;
    const ICON: Icon = Icon::MoveUpRight;
    const COLOR: &'static str = "#f43f5e";

    fn fields(&self) -> AnnotationFields<'_> {
        AnnotationFields {
            match_text: &self.match_text,
            text: &self.text,
            color: &self.color,
            padding: self.padding,
            lead_ms: self.lead_ms,
            hold_ms: self.hold_ms,
            look: self.look,
            roughness: self.roughness,
            text_color: None,
            font_size: Some(self.font_size),
            font: Some(&self.font),
            label_look: Some(self.style.label()),
            mark_look: None,
        }
    }
}

#[settings(eq)]
pub struct CaptionsFromScript {
    #[setting(
        "Script",
        default = "Record a real flow. Turn it into a cinematic demo.",
        widget = "textarea",
        check = { "rule": "not-blank", "message": "Write a script" }
    )]
    pub script: String,
    #[setting(
        "Speaking pace",
        default = 165,
        range(min = 60, max = 400),
        unit = "words/min"
    )]
    pub words_per_minute: u32,
    #[setting("Start at", range(min = 0, max = 600_000), step = 50, unit = "ms")]
    pub start_ms: u32,
    #[setting(
        "Pause between sentences",
        default = 350,
        range(min = 0, max = 5000),
        step = 50,
        unit = "ms",
        advanced
    )]
    pub line_pause_ms: u32,
    #[setting(
        "Pause between paragraphs",
        default = 700,
        range(min = 0, max = 10000),
        step = 1,
        unit = "ms",
        advanced
    )]
    pub paragraph_pause_ms: u32,
}

fn captions_from_script() -> NodeDef {
    NodeDef::of::<CaptionsFromScript>(
        CAPTIONS_FROM_SCRIPT_TYPE,
        "Captions from script",
        Category::TextAndOverlays,
        Phase::Compile,
    )
    .describe("Turns your script into timed captions at a natural speaking pace.")
    .outputs(vec![(CAPTIONS_PORT, port(PortType::Captions, "Captions"))])
    .card(|ctx: &CardCtx<'_, CaptionsFromScript>| {
        let p = &ctx.params;
        vec![
            Some(quoted_row(Icon::Captions, &p.script, "Write a script")),
            Some(row(
                Icon::Gauge,
                format!("{} words/min", p.words_per_minute),
            )),
        ]
    })
    .compiles(|ctx: &Ctx<'_, CaptionsFromScript>| {
        let p = &ctx.params;
        let pace = ScriptPace {
            words_per_minute: f64::from(p.words_per_minute),
            start_ms: f64::from(p.start_ms),
            line_pause_ms: f64::from(p.line_pause_ms),
            paragraph_pause_ms: f64::from(p.paragraph_pause_ms),
        };
        outputs([(
            CAPTIONS_PORT,
            CompileValue::Captions(CaptionsValue {
                captions: script_to_captions(&p.script, pace),
            }),
        )])
    })
}

fn humanized(value: &impl Serialize) -> Option<String> {
    serde_json::to_value(value)
        .ok()?
        .as_str()
        .filter(|s| !s.is_empty())
        .map(humanize)
}

fn show_captions() -> NodeDef {
    NodeDef::of::<Captions>(
        SHOW_CAPTIONS_TYPE,
        "Show captions",
        Category::TextAndOverlays,
        Phase::Compile,
    )
    .describe("Displays captions over the video, word by word, karaoke or line by line.")
    .inputs(vec![
        (CAPTIONS_PORT, port(PortType::Captions, "Captions")),
        (CLIP_PORT, opt(PortType::Clip, "Over clip")),
    ])
    .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
    .card(|ctx: &CardCtx<'_, Captions>| {
        let p = &ctx.params;
        let look: Vec<String> = [humanized(&p.style), humanized(&p.position)]
            .into_iter()
            .flatten()
            .collect();
        let spoken = ctx.inputs.captions_ms.get(CAPTIONS_PORT).copied();
        let clip_ms = ctx
            .inputs
            .frames
            .get(CLIP_PORT)
            .filter(|f| **f != 0)
            .map(|f| frames_to_ms(*f, ctx.fps()));
        vec![
            Some(row(Icon::Captions, truncate_card(&look.join(" · ")))),
            match (spoken, clip_ms) {
                (Some(spoken), Some(clip_ms)) if clip_ms != 0.0 && spoken > clip_ms => Some(
                    row(
                        Icon::TriangleAlert,
                        format!(
                            "Runs {} past the clip, which cuts it off",
                            format_duration(spoken - clip_ms)
                        ),
                    )
                    .tone(Tone::Warn),
                ),
                _ => None,
            },
        ]
    })
    .compiles(|ctx: &Ctx<'_, Captions>| {
        let under = ctx.input(CLIP_PORT).and_then(CompileValue::as_clip);
        let Some(caps) = ctx.input(CAPTIONS_PORT).and_then(CompileValue::as_captions) else {
            let clip = under
                .cloned()
                .unwrap_or_else(|| empty_clip(EMPTY_CLIP_FRAMES));
            return outputs([(CLIP_PORT, CompileValue::Clip(clip))]);
        };
        let end = caps.captions.last().map_or(EMPTY_CAPTIONS_MS, |c| c.end_ms);
        let mut props = to_values(&ctx.params);
        match font_prop(&ctx.params.font_family) {
            Some(stack) => props.insert("fontFamily".into(), Value::from(stack)),
            None => props.remove("fontFamily"),
        };
        props.insert(
            "captions".into(),
            serde_json::to_value(&caps.captions).expect("captions serialize"),
        );
        let layer = item_clip(
            ctx.node_id,
            CAPTIONS_ELEMENT,
            props,
            ms_to_frames(end + CAPTION_TAIL_MS, ctx.settings.fps),
            ItemExtra::default(),
        );
        let clip = match under {
            Some(under) => {
                let stacked = stack_clips(ctx.node_id, &[under.clone(), layer], &[], None, None);
                opsis_types::values::ClipValue {
                    duration_in_frames: under.duration_in_frames,
                    enter: under.enter.clone(),
                    ..stacked
                }
            }
            None => layer,
        };
        outputs([(CLIP_PORT, CompileValue::Clip(clip))])
    })
}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        annotation::<Tooltip>(
            "text.tooltip",
            "Tooltip",
            Category::TextAndOverlays,
            "A tooltip that stays attached to an element as the camera moves.",
        )
        .look(NodeLook::of("lucide:message-square")),
        annotation::<CalloutArrow>(
            "text.calloutArrow",
            "Callout arrow",
            Category::TextAndOverlays,
            "A hand-drawn arrow pointing at an element, with a label.",
        )
        .look(NodeLook::of("lucide:move-up-right")),
        captions_from_script().look(NodeLook::of("lucide:scroll-text").morphs("lucide:captions")),
        show_captions().look(NodeLook::of("lucide:captions")),
    ]
}
