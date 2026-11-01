use opsis_macros::{choice, settings};
use opsis_types::cards::{CardRow, Icon, Tone};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers;
use opsis_types::timeline::{
    EditHandles, GroupProps, ItemBase, LinkedParam, ParamRef, ParamUnit, Sign, TimelineTrack,
    TransitionEase, TransitionRef,
};
use opsis_types::values::{ClipValue, CompileValue};
use opsis_types::workflow::{PortType, WorkflowSettings};
use schemars::Schema;
use serde_json::{Value, json};

use crate::card_rows::{
    Cover, TAG_TEXT_MAX, cover, cover_row, frame_secs, quote, range_row, transition_length_text,
};
use crate::clip::{
    EMPTY_CLIP_FRAMES, SequenceSlots, empty_clip, group_item, mark_layers, sequence_clips,
    stack_clips, with_handles, wrap_clip,
};
use crate::errors::CompileError;
use crate::limits::{MAX_FRAMES, MAX_REPEATS};
use crate::node::{
    CLIP_PORT, CardCtx, Category, Ctx, Inputs, NodeDef, Outputs, Phase, RowExt, missing, opt,
    outputs, port, row, with_clip,
};
use crate::nodes::transitions::{
    NO_TRANSITION, default_transition_params, transition_kinds, transition_ref, transition_title,
};
use crate::ports::{clip_family, clip_label, connected_clips, in_index, layer_label};
use crate::remap::{Remap, speed_remap};
use crate::text::{format_duration, parse_number_text, plural};
use crate::time::{at_least_one_frame, frames_to_ms, ms_to_frames, ms_to_frames_from};
use crate::vars::Vars;

fn with_connected_clips(
    inputs: &Inputs,
    build: impl FnOnce(Vec<ClipValue>, Vec<String>) -> ClipValue,
) -> Outputs {
    let connected = connected_clips(inputs);
    let clip = if connected.is_empty() {
        empty_clip(EMPTY_CLIP_FRAMES)
    } else {
        let ports = connected.iter().map(|(p, _)| p.clone()).collect();
        build(
            connected.into_iter().map(|(_, c)| c.clone()).collect(),
            ports,
        )
    };
    outputs([(CLIP_PORT, CompileValue::Clip(clip))])
}

fn transition_choices(schema: &mut Schema) {
    let kinds = transition_kinds();
    let cut_kinds: Vec<&String> = kinds.iter().filter(|k| *k != NO_TRANSITION).collect();
    if let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) {
        if let Some(transition) = properties.get_mut("transition") {
            transition["enum"] = json!(kinds);
        }
        if let Some(keep_whole) = properties.get_mut("keepWhole") {
            keep_whole["x-show-if"] = json!({ "transition": cut_kinds });
        }
    }
}

#[settings]
#[schemars(transform = transition_choices)]
pub struct Cut {
    #[setting("Before", default = "in2", widget = "cut")]
    pub before: String,
    #[setting("Transition", default = "fade", widget = "transition")]
    pub transition: String,
    #[setting(
        "Length",
        default = 15,
        range(min = 1, max = 108000),
        step = 1,
        unit = "frames"
    )]
    pub transition_duration: u32,
    #[setting(
        "Keep both clips whole",
        help = "The transition plays over a still of the two clips' last and first frames.",
        advanced
    )]
    pub keep_whole: bool,
    #[setting(
        "Easing",
        default = TransitionEase::Linear,
        advanced
    )]
    pub easing: TransitionEase,
}

#[settings]
#[schemars(transform = transition_choices)]
pub struct Sequence {
    #[setting("Transition", default = "fade", widget = "transition")]
    pub transition: String,
    #[setting(
        "Transition length",
        default = 15,
        range(min = 1, max = 108000),
        step = 1,
        unit = "frames"
    )]
    pub transition_duration: u32,
    #[setting(
        "Keep every clip whole",
        help = "The transition plays over a still of the last and first frames, so no clip loses screen time. The sequence gets longer by each transition.",
        advanced
    )]
    pub keep_whole: bool,
    #[setting(
        "Easing",
        default = TransitionEase::Linear,
        advanced
    )]
    pub easing: TransitionEase,
    #[setting(
        "Different transitions",
        help = "Use another transition before some clips. The others use the transition above.",
        advanced
    )]
    pub cuts: Vec<Cut>,
}

struct CutPlan {
    transition: String,
    transition_duration: u32,
    keep_whole: bool,
    easing: TransitionEase,
    before: String,
    after: String,
    own: bool,
}

fn cuts_for(p: &Sequence, ports: &[String]) -> Vec<CutPlan> {
    ports
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, handle)| {
            let own = p.cuts.iter().rev().find(|c| c.before == *handle);
            let (transition, duration, keep_whole, easing) = match own {
                Some(c) => (
                    c.transition.clone(),
                    c.transition_duration,
                    c.keep_whole,
                    c.easing,
                ),
                None => (
                    p.transition.clone(),
                    p.transition_duration,
                    p.keep_whole,
                    p.easing,
                ),
            };
            CutPlan {
                transition,
                transition_duration: duration,
                keep_whole,
                easing,
                before: handle.clone(),
                after: ports[i - 1].clone(),
                own: own.is_some(),
            }
        })
        .collect()
}

fn worst_cover(
    cuts: &[CutPlan],
    frames: &indexmap::IndexMap<String, u32>,
    fps: u32,
) -> Option<CardRow> {
    let worst = cuts
        .iter()
        .filter(|c| c.transition != NO_TRANSITION && !c.keep_whole)
        .filter_map(|c| {
            cover(
                c.transition_duration,
                &[
                    frames.get(&c.after).copied(),
                    frames.get(&c.before).copied(),
                ],
            )
        })
        .fold(None::<Cover>, |w, c| match w {
            Some(w) if c.ratio <= w.ratio => Some(w),
            _ => Some(c),
        });
    cover_row(worst, fps)
}

fn connected_ports(ctx_connected: &[String]) -> Vec<String> {
    let mut ports: Vec<(usize, &String)> = ctx_connected
        .iter()
        .filter_map(|c| Some((in_index(c)?, c)))
        .collect();
    ports.sort_by_key(|(i, _)| *i);
    ports.dedup_by_key(|(i, _)| *i);
    ports.into_iter().map(|(_, c)| c.clone()).collect()
}

fn sequence() -> NodeDef {
    NodeDef::of::<Sequence>(
        "timeline.sequence",
        "Sequence",
        Category::Timeline,
        Phase::Compile,
    )
    .describe("Plays clips one after another, with a transition between each.")
    .family(clip_family(clip_label).with_refs(&[("cuts", "before")]))
    .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
    .card(|ctx: &CardCtx<'_, Sequence>| {
        let p = &ctx.params;
        let fps = ctx.fps();
        let ports = connected_ports(&ctx.inputs.connected);
        let cuts = cuts_for(p, &ports);
        let own = cuts.iter().filter(|c| c.own).count();
        let length = ctx
            .inputs
            .clip_frames
            .filter(|f| *f != 0)
            .map(|f| format!(" · {}", frame_secs(f, fps)))
            .unwrap_or_default();
        vec![
            Some(
                row(
                    Icon::ListVideo,
                    format!("{}{length}", plural(numbers::float(ports.len()), "clip")),
                )
                .warn_if(ports.is_empty()),
            ),
            (p.transition != NO_TRANSITION).then(|| {
                row(
                    Icon::ArrowRightLeft,
                    format!(
                        "{} · {}",
                        transition_title(&p.transition),
                        transition_length_text(p.transition_duration, fps, p.keep_whole)
                    ),
                )
            }),
            (own > 0).then(|| {
                row(
                    Icon::ArrowRightLeft,
                    format!(
                        "{} {} own transition",
                        plural(numbers::float(own), "cut"),
                        if own == 1 { "uses its" } else { "use their" }
                    ),
                )
            }),
            worst_cover(&cuts, &ctx.inputs.frames, fps),
        ]
    })
    .compiles(|ctx: &Ctx<'_, Sequence>| {
        with_connected_clips(ctx.inputs, |clips, ports| {
            let cuts = cuts_for(&ctx.params, &ports);
            let transitions: Vec<Option<TransitionRef>> = (0..clips.len())
                .map(|i| {
                    let cut = if i > 0 { cuts.get(i - 1) } else { None }?;
                    (cut.transition != NO_TRANSITION).then(|| {
                        transition_ref(
                            &cut.transition,
                            cut.transition_duration,
                            cut.keep_whole,
                            cut.easing,
                            default_transition_params(&cut.transition),
                        )
                    })
                })
                .collect();
            sequence_clips(
                ctx.node_id,
                &clips,
                &transitions,
                SequenceSlots {
                    ports: Some(&ports),
                    slot_node: None,
                },
            )
        })
    })
}

#[choice]
pub enum LengthSource {
    #[label("Longest")]
    Longest,
    #[label("Layer 1")]
    First,
    #[label("Front layer")]
    Last,
}

impl LengthSource {
    fn row(self) -> &'static str {
        match self {
            Self::Longest => "Length follows the longest",
            Self::First => "Length follows Layer 1",
            Self::Last => "Length follows the front layer",
        }
    }

    fn pick(self, clips: &[ClipValue]) -> Option<&ClipValue> {
        match self {
            Self::Longest => None,
            Self::First => clips.first(),
            Self::Last => clips.last(),
        }
    }
}

#[settings]
pub struct Layers {
    #[setting(
        "Length follows",
        default = LengthSource::Longest
    )]
    pub duration: LengthSource,
    #[setting("Start each layer at", widget = "offsets")]
    pub offsets_ms: String,
    #[setting(
        "Keep this layout in other shapes",
        help = "In 9:16, 1:1 and 4:5 the layers stay arranged as they are here, scaled to fit, so marks drawn over a picture stay on it.",
        advanced
    )]
    pub keep_layout: bool,
}

pub fn parse_number_list(text: &str) -> Vec<f64> {
    text.split(',')
        .map(|part| parse_number_text(part).unwrap_or(0.0))
        .collect()
}

fn keep_layout(node_id: &str, clip: ClipValue, settings: &WorkflowSettings) -> ClipValue {
    let group = group_item(
        ItemBase::new(format!("{node_id}:design:g"), 0, clip.duration_in_frames),
        GroupProps {
            tracks: clip.tracks.clone(),
            design: Some(opsis_types::actions::Viewport {
                width: settings.width,
                height: settings.height,
            }),
            ..GroupProps::default()
        },
    );
    ClipValue {
        tracks: vec![TimelineTrack {
            id: format!("{node_id}:design"),
            items: vec![group],
            layer: None,
        }],
        ..clip
    }
}

fn layers() -> NodeDef {
    NodeDef::of::<Layers>(
        "timeline.stack",
        "Layers",
        Category::Timeline,
        Phase::Compile,
    )
    .describe("Stacks clips on top of each other: Layer 1 at the back, the last at the front.")
    .family(clip_family(layer_label))
    .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
    .card(|ctx: &CardCtx<'_, Layers>| {
        let layers = connected_ports(&ctx.inputs.connected).len();
        vec![
            Some(row(Icon::Layers, plural(numbers::float(layers), "layer")).warn_if(layers == 0)),
            Some(row(Icon::Clock, ctx.params.duration.row())),
            ctx.inputs
                .clip_frames
                .filter(|f| *f != 0)
                .map(|f| row(Icon::Timer, frame_secs(f, ctx.fps()))),
        ]
    })
    .compiles(|ctx: &Ctx<'_, Layers>| {
        let p = &ctx.params;
        with_connected_clips(ctx.inputs, |clips, ports| {
            let fps = ctx.settings.fps;
            let offsets: Vec<u32> = parse_number_list(&p.offsets_ms)
                .into_iter()
                .map(|n| ms_to_frames_from(n, fps, 0))
                .collect();
            let node_id = ctx.node_id;
            let move_for = |index: usize| {
                Some(ParamRef {
                    index: u32::try_from(index).ok(),
                    ..ParamRef::new(node_id, "offsetsMs", ParamUnit::Ms)
                })
            };
            let stacked = mark_layers(stack_clips(
                node_id,
                &clips,
                &offsets,
                Some(&ports),
                Some(&move_for),
            ));
            let clip = match p.duration.pick(&clips) {
                Some(pick) => ClipValue {
                    duration_in_frames: pick.duration_in_frames,
                    ..stacked
                },
                None => stacked,
            };
            if p.keep_layout {
                keep_layout(node_id, clip, ctx.settings)
            } else {
                clip
            }
        })
    })
}

#[settings]
pub struct Trim {
    #[setting("Start", range(min = 0, max = 600000), step = 50, unit = "ms")]
    pub start_ms: u32,
    #[setting(
        "Keep",
        range(min = 0, max = 600000),
        step = 50,
        unit = "ms",
        zero_label = "To the end"
    )]
    pub duration_ms: u32,
}

fn trim() -> NodeDef {
    NodeDef::clip_modifier::<Trim>("timeline.trim", "Trim", Category::Timeline)
        .describe("Keeps only part of a clip.")
        .card(|ctx: &CardCtx<'_, Trim>| {
            let p = &ctx.params;
            let clip_ms = ctx
                .inputs
                .frames
                .get(CLIP_PORT)
                .filter(|f| **f != 0)
                .map(|f| frames_to_ms(*f, ctx.fps()));
            let end = (p.duration_ms != 0).then(|| f64::from(p.start_ms + p.duration_ms));
            vec![
                Some(range_row(f64::from(p.start_ms), end)),
                clip_ms
                    .filter(|ms| *ms != 0.0 && end.unwrap_or(f64::from(p.start_ms)) > *ms)
                    .map(|ms| {
                        row(
                            Icon::TriangleAlert,
                            format!("The clip ends at {}", format_duration(ms)),
                        )
                        .tone(Tone::Warn)
                    }),
            ]
        })
        .compile(|ctx: &Ctx<'_, Trim>| {
            let p = &ctx.params;
            let fps = ctx.settings.fps;
            let Some(clip) = ctx.input(CLIP_PORT).and_then(CompileValue::as_clip) else {
                return Ok(Some(Outputs::new()));
            };
            let start = ms_to_frames_from(f64::from(p.start_ms), fps, 0);
            if start >= clip.duration_in_frames {
                return Err(CompileError::StartAfterEnd {
                    clip: format_duration(frames_to_ms(clip.duration_in_frames, fps)),
                });
            }
            let rest = clip.duration_in_frames - start;
            let duration = if p.duration_ms != 0 {
                rest.min(ms_to_frames(f64::from(p.duration_ms), fps))
            } else {
                rest
            };
            let mut trim_start = ParamRef::new(ctx.node_id, "startMs", ParamUnit::Ms);
            trim_start.linked = Some(vec![LinkedParam {
                param: "durationMs".into(),
                sign: Sign::Opposite,
                if_non_zero: Some(true),
            }]);
            let handles = EditHandles {
                trim_start: Some(trim_start),
                trim_end: Some(ParamRef::new(ctx.node_id, "durationMs", ParamUnit::Ms)),
                ..EditHandles::default()
            };
            let wrapped = wrap_clip(
                clip,
                ctx.node_id,
                GroupProps {
                    start_from: Some(start),
                    ..GroupProps::default()
                },
                Some(duration),
            );
            Ok(Some(outputs([(
                CLIP_PORT,
                CompileValue::Clip(with_handles(wrapped, handles, None, None)),
            )])))
        })
}

#[settings]
pub struct Repeat {
    #[setting("Times", default = 2, range(min = 1, max = MAX_REPEATS), unit = "×")]
    pub times: u32,
}

fn repeat() -> NodeDef {
    NodeDef::clip_modifier::<Repeat>("timeline.loop", "Repeat", Category::Timeline)
        .describe("Plays a clip several times.")
        .card(|ctx: &CardCtx<'_, Repeat>| {
            vec![Some(row(Icon::Repeat, format!("{}×", ctx.params.times)))]
        })
        .compile(|ctx: &Ctx<'_, Repeat>| {
            let p = &ctx.params;
            let fps = ctx.settings.fps;
            let Some(clip) = ctx.input(CLIP_PORT).and_then(CompileValue::as_clip) else {
                return Ok(Some(Outputs::new()));
            };
            let Some(total) = clip
                .duration_in_frames
                .checked_mul(p.times)
                .filter(|total| *total <= MAX_FRAMES)
            else {
                let total = f64::from(clip.duration_in_frames) * f64::from(p.times);
                return Err(CompileError::TooLong {
                    times: p.times,
                    length: format_duration(frames_to_ms(total, fps)),
                    limit: format_duration(frames_to_ms(MAX_FRAMES, fps)),
                });
            };
            let props = GroupProps {
                loop_every: Some(clip.duration_in_frames),
                ..GroupProps::default()
            };
            Ok(Some(outputs([(
                CLIP_PORT,
                CompileValue::Clip(wrap_clip(clip, ctx.node_id, props, Some(total))),
            )])))
        })
}

#[settings]
pub struct Delay {
    #[setting(
        "Delay",
        default = 500,
        range(min = 0, max = 600000),
        step = 50,
        unit = "ms"
    )]
    pub ms: u32,
}

fn delay() -> NodeDef {
    NodeDef::clip_modifier::<Delay>("timeline.delay", "Delay", Category::Timeline)
        .describe("Starts a clip later.")
        .card(|ctx: &CardCtx<'_, Delay>| {
            vec![Some(row(
                Icon::Clock,
                format!("+{}", format_duration(f64::from(ctx.params.ms))),
            ))]
        })
        .compiles(|ctx: &Ctx<'_, Delay>| {
            with_clip(ctx, |clip| {
                let offset = ms_to_frames_from(f64::from(ctx.params.ms), ctx.settings.fps, 0);
                let node_id = ctx.node_id;
                let move_for = |_: usize| Some(ParamRef::new(node_id, "ms", ParamUnit::Ms));
                stack_clips(
                    node_id,
                    std::slice::from_ref(clip),
                    &[offset],
                    None,
                    Some(&move_for),
                )
            })
        })
}

#[settings]
pub struct Speed {
    #[setting(
        "Speed",
        default = 1.5,
        range(min = 0.1, max = 64),
        step = 0.05,
        unit = "×"
    )]
    pub rate: f64,
    #[setting("Keep the pitch", default = true, advanced)]
    pub keep_pitch: bool,
}

fn speed() -> NodeDef {
    NodeDef::clip_modifier::<Speed>("timeline.speed", "Speed", Category::Timeline)
        .describe("Plays a clip faster or slower.")
        .card(|ctx: &CardCtx<'_, Speed>| {
            let rate = ctx.params.rate;
            vec![Some(row(Icon::Gauge, format!("{}×", rate)))]
        })
        .compiles(|ctx: &Ctx<'_, Speed>| {
            with_clip(ctx, |clip| {
                let out = at_least_one_frame(f64::from(clip.duration_in_frames) / ctx.params.rate);
                let props = GroupProps {
                    remap: Some(speed_remap(clip.duration_in_frames, out)),
                    shift_pitch: (!ctx.params.keep_pitch).then_some(true),
                    ..GroupProps::default()
                };
                let handles = EditHandles {
                    speed: Some(ParamRef::new(ctx.node_id, "rate", ParamUnit::Rate)),
                    ..EditHandles::default()
                };
                with_handles(
                    wrap_clip(clip, ctx.node_id, props, Some(out)),
                    handles,
                    None,
                    None,
                )
            })
        })
}

#[choice]
pub enum FitMode {
    #[label("Change speed")]
    Speed,
    #[label("Hold the end")]
    Hold,
    #[label("Cut")]
    Cut,
}

impl FitMode {
    fn row(self) -> &'static str {
        match self {
            Self::Speed => "By speed",
            Self::Hold => "By holding the end",
            Self::Cut => "By cutting",
        }
    }

    fn remap(self, length: u32, target: u32) -> Option<Remap> {
        let (end, last) = (f64::from(target), f64::from(length) - 1.0);
        match self {
            Self::Speed => Some(speed_remap(length, target)),
            Self::Hold if target <= length => None,
            Self::Hold if length <= 1 => Some(vec![(0.0, 0.0), (end, 0.0)]),
            Self::Hold => Some(vec![(0.0, 0.0), (last, last), (end, last)]),
            Self::Cut => None,
        }
    }

    fn warning(self, input: u32, output: u32, fps: u32) -> Option<CardRow> {
        match self {
            Self::Speed => {
                let speed = f64::from(input) / f64::from(output);
                ((speed - 1.0).abs() > 0.02).then(|| {
                    row(Icon::FastForward, format!("Plays at {speed:.2}×"))
                        .warn_if(!(0.67..=1.5).contains(&speed))
                })
            }
            Self::Cut if input < output => Some(
                row(
                    Icon::TriangleAlert,
                    format!(
                        "Ends {} early, then shows nothing",
                        frame_secs(output - input, fps)
                    ),
                )
                .tone(Tone::Warn),
            ),
            Self::Cut if input > output => Some(
                row(
                    Icon::Scissors,
                    format!("Cuts {} off the end", frame_secs(input - output, fps)),
                )
                .warn_if(input - output > fps / 2),
            ),
            Self::Hold | Self::Cut => None,
        }
    }
}

#[settings]
pub struct FitLength {
    #[setting(
        "Length",
        default = 10000,
        range(min = 0, max = 600000),
        step = 50,
        unit = "ms",
        help = "Ignored when a sound is connected."
    )]
    pub target_ms: u32,
    #[setting(
        "How",
        default = FitMode::Speed
    )]
    pub mode: FitMode,
}

fn fit_length() -> NodeDef {
    NodeDef::of::<FitLength>(
        "timeline.durationFit",
        "Fit length",
        Category::Timeline,
        Phase::Compile,
    )
    .describe(
        "Makes a clip a set length, or as long as a sound, by speeding up, holding or cutting.",
    )
    .inputs(vec![
        (CLIP_PORT, port(PortType::Clip, "Clip")),
        ("audio", opt(PortType::Audio, "Fit to this sound")),
    ])
    .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
    .card(|ctx: &CardCtx<'_, FitLength>| {
        let p = &ctx.params;
        let input = ctx
            .inputs
            .frames
            .get(CLIP_PORT)
            .copied()
            .filter(|f| *f != 0);
        let output = ctx.inputs.clip_frames.filter(|f| *f != 0);
        vec![
            Some(row(
                Icon::Ruler,
                if ctx.connected("audio") {
                    "Fits the sound".to_owned()
                } else {
                    format_duration(f64::from(p.target_ms))
                },
            )),
            Some(row(Icon::Settings2, p.mode.row())),
            input
                .zip(output)
                .and_then(|(i, o)| p.mode.warning(i, o, ctx.fps())),
        ]
    })
    .compiles(|ctx: &Ctx<'_, FitLength>| {
        let p = &ctx.params;
        with_clip(ctx, |clip| {
            let sound = ctx
                .input("audio")
                .and_then(CompileValue::as_audio)
                .and_then(|a| a.duration_in_frames)
                .filter(|f| *f > 0);
            let target = sound
                .unwrap_or_else(|| ms_to_frames(f64::from(p.target_ms), ctx.settings.fps))
                .max(1);
            let props = GroupProps {
                remap: p.mode.remap(clip.duration_in_frames, target),
                ..GroupProps::default()
            };
            let fitted = wrap_clip(clip, ctx.node_id, props, Some(target));
            match sound {
                Some(_) => fitted,
                None => with_handles(
                    fitted,
                    EditHandles {
                        trim_end: Some(ParamRef::new(ctx.node_id, "targetMs", ParamUnit::Ms)),
                        ..EditHandles::default()
                    },
                    None,
                    None,
                ),
            }
        })
    })
}

#[settings]
pub struct Branch {
    #[setting("Variable", default = "Version", widget = "variable")]
    pub variable: String,
    #[setting(
        "Equals",
        default = "A",
        help = "Leave empty to use the first clip whenever the variable has a value.",
        zero_label = "Any value"
    )]
    pub equals: String,
}

pub fn branch_port(value: Option<&str>, equals: &str) -> &'static str {
    let has = value.is_some_and(|v| !v.trim().is_empty());
    if equals.is_empty() {
        return if has { "a" } else { "b" };
    }
    if has && value.map(str::trim) == Some(equals.trim()) {
        "a"
    } else {
        "b"
    }
}

fn chosen(vars: &Vars, p: &Branch) -> &'static str {
    branch_port(vars.get(p.variable.trim()).map(String::as_str), &p.equals)
}

fn branch() -> NodeDef {
    NodeDef::of::<Branch>("timeline.branch", "Choose by variable", Category::Timeline, Phase::Compile)
        .describe(
            "Uses the first clip when a variable has a value, or the value you give, otherwise the second. Handy for A/B versions.",
        )
        .inputs(vec![("a", opt(PortType::Clip, "First")), ("b", opt(PortType::Clip, "Otherwise"))])
        .outputs(vec![(CLIP_PORT, port(PortType::Clip, "Clip"))])
        .card(|ctx: &CardCtx<'_, Branch>| {
            let p = &ctx.params;
            let variable = p.variable.trim();
            vec![Some(if variable.is_empty() {
                missing(Icon::GitBranch, "Choose a variable")
            } else if p.equals.is_empty() {
                row(Icon::GitBranch, format!("{variable} has a value"))
            } else {
                row(Icon::GitBranch, format!("{variable} = {}", quote(&p.equals, TAG_TEXT_MAX)))
            })]
        })
        .compile(|ctx: &Ctx<'_, Branch>| {
            let port = chosen(ctx.vars, &ctx.params);
            match ctx.input(port) {
                Some(clip) => Ok(Some(outputs([(CLIP_PORT, clip.clone())]))),
                None => Err(CompileError::MissingClip(if port == "a" { "First" } else { "Otherwise" })),
            }
        })
}

pub fn nodes() -> Vec<NodeDef> {
    vec![
        sequence().look(NodeLook::of("lucide:list-video")),
        layers().look(NodeLook::of("lucide:layers")),
        trim().look(NodeLook::of("lucide:scissors")),
        repeat().look(NodeLook::of("lucide:repeat").moves(NodeMotion::Turn)),
        delay().look(NodeLook::of("lucide:timer")),
        speed().look(NodeLook::of("lucide:rabbit").morphs("lucide:turtle")),
        fit_length().look(NodeLook::of("lucide:ruler-dimension-line").moves(NodeMotion::Stretch)),
        branch().look(NodeLook::of("lucide:split")),
    ]
}
