use opsis_compile::envelope::sample_points;
use opsis_compile::mix::plan::{BusDuck, BusPlan, ChannelPlan, SoundPlacement};
use opsis_compile::time::clamp;
use opsis_types::mix::{Comp, Eq, EqType, Highpass, LoudnessGoal, Master, SAMPLE_RATE, db_to_gain};
use opsis_types::numbers::whole;
use opsis_types::timeline::{FadeCurve, Keyframe};
use serde::{Deserialize, Serialize};

use crate::codecs::serde_name;
use crate::filters::{atempo_chain, loudnorm_filter};

const LANE_AFTER: usize = 16;
const MIN_COMP_THRESHOLD: f64 = 0.000_976_563;
const MAX_COMP_MAKEUP: f64 = 64.0;
const MIN_LIMITER_CEILING: f64 = 0.0625;
const LIMITER_OVERSAMPLING: u32 = 4;
const LOUDNESS_RANGE: f64 = 11.0;
const ENDLESS_LOOP: i64 = 2_147_483_647;
const MIN_LEVEL_DB: f64 = 0.01;
const MIN_EQ_GAIN_DB: f64 = 0.05;
const MIN_PAN: f64 = 0.005;
const DUCK_THRESHOLD_DB: f64 = -40.0;
const DUCK_REFERENCE_DB: f64 = -12.0;
const DUCK_RANGE_DB: f64 = DUCK_REFERENCE_DB - DUCK_THRESHOLD_DB;
const MIN_DUCK_AMOUNT_DB: f64 = 0.5;
const MAX_DUCK_AMOUNT_DB: f64 = DUCK_RANGE_DB - 1.0;
const MAX_DUCK_RATIO: f64 = 20.0;
const DEFAULT_DIGITS: usize = 6;
const GAIN_DB_DIGITS: usize = 2;
const FREQ_DIGITS: usize = 1;
const Q_DIGITS: usize = 3;
const PAN_DIGITS: usize = 4;
const RATIO_DIGITS: usize = 3;
const COMP_TIME_DIGITS: usize = 3;
const DUCK_TIME_DIGITS: usize = 2;
const SECONDS_DIGITS: usize = 3;
const STEM_PRE: &str = "pre";
const STEM_POST: &str = "post";
const OUT: &str = "out";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Taps {
    pub pre: String,
    pub post: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    pub inputs: Vec<String>,
    pub filter: String,
    pub output: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taps: Option<Taps>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoudnessMeasure {
    pub input_i: String,
    pub input_tp: String,
    pub input_lra: String,
    pub input_thresh: String,
    pub target_offset: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Plan {
    pub fps: u32,
    pub duration_in_frames: u32,
}

impl Plan {
    fn length(self) -> String {
        secs(f64::from(self.duration_in_frames), f64::from(self.fps))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcmCodec {
    F32,
    S16,
}

impl PcmCodec {
    fn name(self) -> &'static str {
        match self {
            Self::F32 => "pcm_f32le",
            Self::S16 => "pcm_s16le",
        }
    }
}

pub fn num(n: f64, digits: usize) -> String {
    let fixed = format!("{:.*}", digits, n);
    let trimmed = if fixed.contains('.') {
        fixed.trim_end_matches('0').trim_end_matches('.')
    } else {
        fixed.as_str()
    };
    if trimmed == "-0" {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn n6(n: f64) -> String {
    num(n, DEFAULT_DIGITS)
}

fn secs(frames: f64, fps: f64) -> String {
    n6(frames / fps)
}

fn db_step(db: f64) -> Vec<String> {
    if db.abs() >= MIN_LEVEL_DB {
        vec![format!("volume={}dB", num(db, GAIN_DB_DIGITS))]
    } else {
        Vec::new()
    }
}

fn envelope_expr(keys: &[Keyframe], fps: f64) -> String {
    let points: Vec<(f64, f64)> = sample_points(keys)
        .into_iter()
        .map(|(frame, value)| (frame / fps, value))
        .collect();
    let Some(&(_, last)) = points.last() else {
        return "1".to_owned();
    };
    if points.len() == 1 {
        return n6(last);
    }
    let mut expr = n6(last);
    for i in (0..points.len() - 1).rev() {
        let (at, av) = points[i];
        let (bt, bv) = points[i + 1];
        let span = bt - at;
        let seg = if span <= 0.0 || av == bv {
            n6(av)
        } else {
            format!("{}+({})*(t-{})/{}", n6(av), n6(bv - av), n6(at), n6(span))
        };
        expr = format!("if(lt(t,{}),{seg},{expr})", n6(bt));
    }
    format!("if(lt(t,{}),{},{expr})", n6(points[0].0), n6(points[0].1))
}

fn side_gain(left: bool, pan: f64) -> String {
    let side = if left { -1.0 } else { 1.0 };
    num(f64::min(1.0, 1.0 + side * pan), PAN_DIGITS)
}

fn side_gain_expr(left: bool, pan: &str) -> String {
    format!("min(1,1{}({pan}))", if left { "-" } else { "+" })
}

fn pan_chain(label: &str, steps: &[String], pan: &str, after: &[String]) -> String {
    let (left, right, left_out, right_out) = (
        format!("{label}l"),
        format!("{label}r"),
        format!("{label}L"),
        format!("{label}R"),
    );
    let mut joined = vec!["join=inputs=2:channel_layout=stereo".to_owned()];
    joined.extend(after.iter().cloned());
    [
        format!(
            "{},channelsplit=channel_layout=stereo[{left}][{right}]",
            steps.join(",")
        ),
        format!(
            "[{left}]volume=eval=frame:volume='{}'[{left_out}]",
            side_gain_expr(true, pan)
        ),
        format!(
            "[{right}]volume=eval=frame:volume='{}'[{right_out}]",
            side_gain_expr(false, pan)
        ),
        format!("[{left_out}][{right_out}]{}[{label}]", joined.join(",")),
    ]
    .join(";")
}

fn placement_steps(sound: &SoundPlacement, fps: f64) -> Vec<String> {
    let mut steps = vec![
        format!("aresample={SAMPLE_RATE}"),
        "aformat=sample_fmts=fltp:channel_layouts=stereo".to_owned(),
    ];
    if let Some(pitch) = sound.pitch.filter(|p| *p != 1.0) {
        steps.extend([
            format!("asetrate={}", whole(f64::from(SAMPLE_RATE) * pitch)),
            format!("aresample={SAMPLE_RATE}"),
        ]);
    }
    if sound.offset > 0 {
        steps.extend([
            format!("atrim=start={}", secs(f64::from(sound.offset), fps)),
            "asetpts=PTS-STARTPTS".to_owned(),
        ]);
    }
    if sound.looped {
        steps.push(format!("aloop=loop=-1:size={ENDLESS_LOOP}"));
    }
    if sound.rate != 1.0 {
        steps.extend([atempo_chain(sound.rate), "asetpts=PTS-STARTPTS".to_owned()]);
    }
    steps.extend([
        format!("atrim=end={}", secs(f64::from(sound.length), fps)),
        "asetpts=PTS-STARTPTS".to_owned(),
    ]);
    if sound.volume != 1.0 {
        steps.push(format!("volume={}", n6(sound.volume)));
    }
    let curve = match sound.fade_curve {
        None | Some(FadeCurve::Tri) => String::new(),
        Some(curve) => format!(":curve={}", serde_name(&curve)),
    };
    if sound.fade_in > 0 {
        steps.push(format!(
            "afade=t=in:st=0:d={}{curve}",
            secs(f64::from(sound.fade_in.min(sound.length)), fps)
        ));
    }
    if sound.fade_out > 0 {
        let d = sound.fade_out.min(sound.length);
        steps.push(format!(
            "afade=t=out:st={}:d={}{curve}",
            secs(f64::from(sound.length - d), fps),
            secs(f64::from(d), fps)
        ));
    }
    if let Some(keys) = sound
        .envelope
        .as_ref()
        .and_then(|e| e.volume.as_deref())
        .filter(|k| !k.is_empty())
    {
        steps.push(format!(
            "volume=eval=frame:volume='{}'",
            envelope_expr(keys, fps)
        ));
    }
    steps
}

fn delayed(sound: &SoundPlacement, fps: f64) -> Vec<String> {
    let delay = (f64::from(sound.at) * 1000.0 / fps).round();
    if delay > 0.0 {
        vec![format!("adelay=delays={}:all=1", delay.to_string())]
    } else {
        Vec::new()
    }
}

fn sound_chain(
    sound: &SoundPlacement,
    input: usize,
    label: &str,
    fps: f64,
    after: Vec<String>,
) -> String {
    let steps = placement_steps(sound, fps);
    let chain = match sound
        .envelope
        .as_ref()
        .and_then(|e| e.pan.as_deref())
        .filter(|k| !k.is_empty())
    {
        Some(pan) => pan_chain(label, &steps, &envelope_expr(pan, fps), &after),
        None => format!("{}[{label}]", [steps, after].concat().join(",")),
    };
    format!("[{input}:a]{chain}")
}

fn eq_steps(eq: &Eq) -> Vec<String> {
    if !eq.on {
        return Vec::new();
    }
    eq.bands
        .iter()
        .filter(|b| b.gain.abs() >= MIN_EQ_GAIN_DB)
        .map(|b| {
            let name = match b.kind {
                EqType::Peak => "equalizer",
                EqType::Lowshelf => "lowshelf",
                EqType::Highshelf => "highshelf",
            };
            format!(
                "{name}=f={}:t=q:w={}:g={}",
                num(b.freq, FREQ_DIGITS),
                num(b.q, Q_DIGITS),
                num(b.gain, GAIN_DB_DIGITS)
            )
        })
        .collect()
}

fn comp_steps(comp: &Comp) -> Vec<String> {
    if !comp.on {
        return Vec::new();
    }
    let threshold = clamp(db_to_gain(comp.threshold), MIN_COMP_THRESHOLD, 1.0);
    let makeup = clamp(db_to_gain(comp.makeup), 1.0, MAX_COMP_MAKEUP);
    let options = [
        format!("threshold={}", n6(threshold)),
        format!("ratio={}", num(comp.ratio, RATIO_DIGITS)),
        format!("attack={}", num(comp.attack, COMP_TIME_DIGITS)),
        format!("release={}", num(comp.release, COMP_TIME_DIGITS)),
        format!("makeup={}", n6(makeup)),
    ];
    vec![format!("acompressor={}", options.join(":"))]
}

fn pan_step(pan: f64) -> Vec<String> {
    if pan.abs() < MIN_PAN {
        return Vec::new();
    }
    vec![format!(
        "pan=stereo|c0={}*c0|c1={}*c1",
        side_gain(true, pan),
        side_gain(false, pan)
    )]
}

fn highpass_step(highpass: &Highpass) -> Vec<String> {
    if highpass.on {
        vec![format!("highpass=f={}", num(highpass.freq, FREQ_DIGITS))]
    } else {
        Vec::new()
    }
}

struct Stages {
    pre: Vec<String>,
    comp: Vec<String>,
    post: Vec<String>,
}

fn strip_stages(channel: &ChannelPlan) -> Stages {
    let s = &channel.strip;
    Stages {
        pre: [db_step(s.trim), highpass_step(&s.highpass), eq_steps(&s.eq)].concat(),
        comp: comp_steps(&s.comp),
        post: [pan_step(s.pan), db_step(s.fader)].concat(),
    }
}

fn bus_steps(bus: &BusPlan) -> Vec<String> {
    [
        highpass_step(&bus.processing.highpass),
        eq_steps(&bus.processing.eq),
        comp_steps(&bus.processing.comp),
    ]
    .concat()
}

fn loudnorm_base(target: LoudnessGoal) -> String {
    loudnorm_filter(target.lufs, target.true_peak, LOUDNESS_RANGE)
}

fn master_tail(master: &Master, measured: Option<&LoudnessMeasure>) -> Vec<String> {
    let mut steps = db_step(master.fader);
    if let (Some(target), Some(m)) = (master.loudness.target(), measured) {
        let options = [
            loudnorm_base(target),
            format!("measured_I={}", m.input_i),
            format!("measured_TP={}", m.input_tp),
            format!("measured_LRA={}", m.input_lra),
            format!("measured_thresh={}", m.input_thresh),
            format!("offset={}", m.target_offset),
            "linear=true".to_owned(),
            "print_format=summary".to_owned(),
        ];
        steps.extend([options.join(":"), format!("aresample={SAMPLE_RATE}")]);
    }
    if master.limiter.on {
        steps.extend([
            format!("aresample={}", SAMPLE_RATE * LIMITER_OVERSAMPLING),
            format!(
                "alimiter=limit={}:level=false",
                n6(f64::max(
                    MIN_LIMITER_CEILING,
                    db_to_gain(master.limiter.ceiling)
                ))
            ),
            format!("aresample={SAMPLE_RATE}"),
        ]);
    }
    steps
}

fn master_stages(master: &Master, measured: Option<&LoudnessMeasure>) -> Stages {
    Stages {
        pre: eq_steps(&master.eq),
        comp: comp_steps(&master.comp),
        post: master_tail(master, measured),
    }
}

fn pinned() -> String {
    format!("aformat=sample_fmts=fltp:sample_rates={SAMPLE_RATE}:channel_layouts=stereo")
}

fn tapped(sum: &str, comp: &[String], post: &[String]) -> String {
    let tail = if post.is_empty() {
        "anull".to_owned()
    } else {
        post.join(",")
    };
    [
        format!("[{sum}]{},asplit=2[{STEM_PRE}][c]", pinned()),
        format!(
            "[c]{},{},asplit=2[{STEM_POST}][m]",
            comp.join(","),
            pinned()
        ),
        format!("[m]{tail}[{OUT}]"),
    ]
    .join(";")
}

fn silence_job(plan: Plan) -> Job {
    Job {
        inputs: Vec::new(),
        filter: format!(
            "anullsrc=r={SAMPLE_RATE}:cl=stereo,atrim=end={}[{OUT}]",
            plan.length()
        ),
        output: OUT.to_owned(),
        taps: None,
    }
}

fn fit(plan: Plan) -> String {
    format!("apad,atrim=end={},asetpts=PTS-STARTPTS", plan.length())
}

fn mix_of(labels: &[String], out: &str, then: &[String]) -> String {
    let tail = if then.is_empty() {
        String::new()
    } else {
        format!(",{}", then.join(","))
    };
    if labels.len() == 1 {
        return format!("[{}]anull{tail}[{out}]", labels[0]);
    }
    let amix = format!(
        "amix=inputs={}:normalize=0:duration=longest:dropout_transition=0",
        labels.len()
    );
    let pads: String = labels.iter().flat_map(|l| ["[", l.as_str(), "]"]).collect();
    format!("{pads}{amix}{tail}[{out}]")
}

fn stream_labels(count: usize, first: usize) -> Vec<String> {
    (0..count).map(|i| format!("{}:a", first + i)).collect()
}

fn strip_job(
    plan: Plan,
    inputs: Vec<String>,
    chains: Vec<String>,
    labels: &[String],
    stages: Stages,
) -> Job {
    let mut first = vec![fit(plan)];
    first.extend(stages.pre);
    if stages.comp.is_empty() {
        first.extend(stages.post);
        let filter = [chains, vec![mix_of(labels, OUT, &first)]]
            .concat()
            .join(";");
        return Job {
            inputs,
            filter,
            output: OUT.to_owned(),
            taps: None,
        };
    }
    let mixed = mix_of(labels, "sum", &first);
    let filter = [
        chains,
        vec![mixed, tapped("sum", &stages.comp, &stages.post)],
    ]
    .concat()
    .join(";");
    Job {
        inputs,
        filter,
        output: OUT.to_owned(),
        taps: Some(Taps {
            pre: STEM_PRE.to_owned(),
            post: STEM_POST.to_owned(),
        }),
    }
}

fn samples(frames: u32, fps: u32) -> u64 {
    u64::from(frames) * u64::from(SAMPLE_RATE) / u64::from(fps)
}

fn pack_lanes(sounds: &[SoundPlacement], shots: &[usize]) -> Vec<Vec<usize>> {
    let mut order = shots.to_vec();
    order.sort_by_key(|&i| sounds[i].at);
    let mut lanes: Vec<(u32, Vec<usize>)> = Vec::new();
    for i in order {
        let s = &sounds[i];
        match lanes.iter_mut().find(|(end, _)| *end <= s.at) {
            Some((end, lane)) => {
                *end = s.at + s.length;
                lane.push(i);
            }
            None => lanes.push((s.at + s.length, vec![i])),
        }
    }
    lanes.into_iter().map(|(_, lane)| lane).collect()
}

fn lane_chains(plan: Plan, sounds: &[SoundPlacement], lane: &[usize], name: &str) -> Vec<String> {
    let fps = f64::from(plan.fps);
    let mut chains = Vec::new();
    let mut parts = Vec::new();
    let first = sounds[lane[0]].at;
    if first > 0 {
        chains.push(format!(
            "anullsrc=r={SAMPLE_RATE}:cl=stereo,atrim=end_sample={}[{name}q]",
            samples(first, plan.fps)
        ));
        parts.push(format!("[{name}q]"));
    }
    for (k, &i) in lane.iter().enumerate() {
        let until = lane
            .get(k + 1)
            .map_or(plan.duration_in_frames, |&next| sounds[next].at);
        let n = samples(until.saturating_sub(sounds[i].at), plan.fps);
        let label = format!("s{i}");
        let fill = vec![
            format!("apad=whole_len={n}"),
            format!("atrim=end_sample={n}"),
        ];
        chains.push(sound_chain(&sounds[i], i, &label, fps, fill));
        parts.push(format!("[{label}]"));
    }
    chains.push(format!(
        "{}concat=n={}:v=0:a=1[{name}]",
        parts.concat(),
        parts.len()
    ));
    chains
}

pub fn channel_job(plan: Plan, channel: &ChannelPlan, inputs: Vec<String>) -> Job {
    if channel.sounds.is_empty() {
        return silence_job(plan);
    }
    let fps = f64::from(plan.fps);
    let sounds = &channel.sounds;
    let shots: Vec<usize> = (0..sounds.len())
        .filter(|&i| !sounds[i].looped && sounds[i].at < plan.duration_in_frames)
        .collect();
    if shots.len() <= LANE_AFTER {
        let labels: Vec<String> = (0..sounds.len()).map(|i| format!("s{i}")).collect();
        let chains = sounds
            .iter()
            .enumerate()
            .map(|(i, s)| sound_chain(s, i, &labels[i], fps, delayed(s, fps)))
            .collect();
        return strip_job(plan, inputs, chains, &labels, strip_stages(channel));
    }
    let mut chains = Vec::new();
    let mut labels = Vec::new();
    for (k, lane) in pack_lanes(sounds, &shots).iter().enumerate() {
        let name = format!("l{k}");
        chains.extend(lane_chains(plan, sounds, lane, &name));
        labels.push(name);
    }
    for (i, s) in sounds
        .iter()
        .enumerate()
        .filter(|(i, _)| !shots.contains(i))
    {
        let label = format!("s{i}");
        chains.push(sound_chain(s, i, &label, fps, delayed(s, fps)));
        labels.push(label);
    }
    strip_job(plan, inputs, chains, &labels, strip_stages(channel))
}

fn duck_ratio(amount: f64) -> f64 {
    let a = clamp(amount, MIN_DUCK_AMOUNT_DB, MAX_DUCK_AMOUNT_DB);
    f64::min(MAX_DUCK_RATIO, 1.0 / (1.0 - a / DUCK_RANGE_DB))
}

fn sidechain(duck: &BusDuck) -> String {
    let options = [
        format!("threshold={}", n6(db_to_gain(DUCK_THRESHOLD_DB))),
        format!("ratio={}", num(duck_ratio(duck.amount), RATIO_DIGITS)),
        format!("attack={}", num(duck.attack, DUCK_TIME_DIGITS)),
        format!("release={}", num(duck.release, DUCK_TIME_DIGITS)),
        "makeup=1".to_owned(),
    ];
    format!("sidechaincompress={}", options.join(":"))
}

pub fn bus_job(plan: Plan, bus: &BusPlan, stem: &dyn Fn(&str) -> String) -> Job {
    if bus.channels.is_empty() {
        return silence_job(plan);
    }
    let inputs: Vec<String> = bus.channels.iter().map(|c| stem(c)).collect();
    let main: Vec<String> = [vec![fit(plan)], bus_steps(bus), db_step(bus.fader)].concat();
    let Some(duck) = &bus.duck else {
        let filter = mix_of(&stream_labels(inputs.len(), 0), OUT, &main);
        return Job {
            inputs,
            filter,
            output: OUT.to_owned(),
            taps: None,
        };
    };
    let key_inputs: Vec<String> = duck.keys.iter().map(|c| stem(c)).collect();
    let filter = [
        mix_of(&stream_labels(inputs.len(), 0), "main", &main),
        mix_of(
            &stream_labels(key_inputs.len(), inputs.len()),
            "key",
            &[fit(plan)],
        ),
        format!("[main][key]{},{}[{OUT}]", sidechain(duck), fit(plan)),
    ]
    .join(";");
    Job {
        inputs: [inputs, key_inputs].concat(),
        filter,
        output: OUT.to_owned(),
        taps: None,
    }
}

pub fn master_job(
    plan: Plan,
    master: &Master,
    buses: &[String],
    measured: Option<&LoudnessMeasure>,
) -> Job {
    if buses.is_empty() {
        return silence_job(plan);
    }
    strip_job(
        plan,
        buses.to_vec(),
        Vec::new(),
        &stream_labels(buses.len(), 0),
        master_stages(master, measured),
    )
}

pub fn loudness_measure_job(plan: Plan, master: &Master, buses: &[String]) -> Option<Job> {
    let target = master.loudness.target()?;
    if buses.is_empty() {
        return None;
    }
    let unlimited = Master {
        loudness: opsis_types::mix::LoudnessTarget::Off,
        limiter: opsis_types::mix::Limiter {
            on: false,
            ceiling: 0.0,
        },
        ..master.clone()
    };
    let stages = master_stages(&unlimited, None);
    let then: Vec<String> = [
        vec![fit(plan)],
        stages.pre,
        stages.comp,
        stages.post,
        vec![format!("{}:print_format=json", loudnorm_base(target))],
    ]
    .concat();
    Some(Job {
        inputs: buses.to_vec(),
        filter: mix_of(&stream_labels(buses.len(), 0), OUT, &then),
        output: OUT.to_owned(),
        taps: None,
    })
}

fn input_args(job: &Job) -> Vec<String> {
    let mut args: Vec<String> = job
        .inputs
        .iter()
        .flat_map(|f| ["-i".to_owned(), f.clone()])
        .collect();
    args.extend(["-filter_complex".to_owned(), job.filter.clone()]);
    args
}

fn stem_args(label: &str, limit: &[String], codec: PcmCodec, file: &str) -> Vec<String> {
    let mut args = vec!["-map".to_owned(), format!("[{label}]")];
    args.extend(limit.iter().cloned());
    args.extend(["-ac", "2", "-ar"].map(str::to_owned));
    args.extend([
        SAMPLE_RATE.to_string(),
        "-c:a".to_owned(),
        codec.name().to_owned(),
        file.to_owned(),
    ]);
    args
}

pub fn job_args(
    job: &Job,
    output: &str,
    codec: PcmCodec,
    taps: Option<&Taps>,
    max_seconds: Option<f64>,
) -> Vec<String> {
    let limit: Vec<String> = max_seconds
        .filter(|s| *s != 0.0)
        .map(|s| vec!["-t".to_owned(), num(s, SECONDS_DIGITS)])
        .unwrap_or_default();
    let mut args = input_args(job);
    args.extend(stem_args(&job.output, &limit, codec, output));
    if let (Some(files), Some(labels)) = (taps, &job.taps) {
        args.extend(stem_args(&labels.pre, &limit, PcmCodec::F32, &files.pre));
        args.extend(stem_args(&labels.post, &limit, PcmCodec::F32, &files.post));
    }
    args
}

pub fn measure_args(job: &Job) -> Vec<String> {
    let mut args = input_args(job);
    args.extend([
        "-map".to_owned(),
        format!("[{}]", job.output),
        "-f".to_owned(),
        "null".to_owned(),
        "-".to_owned(),
    ]);
    args
}

pub fn parse_loudnorm(stderr: &str) -> Option<LoudnessMeasure> {
    let start = stderr.rfind('{')?;
    let end = stderr.rfind('}').filter(|end| *end >= start)?;
    let measured: LoudnessMeasure = serde_json::from_str(&stderr[start..=end]).ok()?;
    (!measured.input_i.is_empty() && measured.input_i != "-inf").then_some(measured)
}

#[cfg(test)]
mod tests {
    use opsis_compile::mix::plan::SoundPlacement;
    use proptest::prelude::*;

    use super::pack_lanes;

    fn placed(at: u32, length: u32) -> SoundPlacement {
        SoundPlacement {
            src: "app:sfx/key.wav".into(),
            at,
            offset: 0,
            length,
            looped: false,
            rate: 1.0,
            pitch: None,
            volume: 1.0,
            fade_in: 0,
            fade_out: 0,
            fade_curve: None,
            envelope: None,
        }
    }

    proptest! {
        #[test]
        fn packs_every_sound_once_on_as_few_lanes_as_ever_play_at_once(
            spans in prop::collection::vec((0u32..600, 1u32..40), 0..80),
        ) {
            let sounds: Vec<SoundPlacement> = spans.iter().map(|&(at, length)| placed(at, length)).collect();
            let shots: Vec<usize> = (0..sounds.len()).collect();
            let lanes = pack_lanes(&sounds, &shots);
            let mut placed: Vec<usize> = lanes.iter().flatten().copied().collect();
            placed.sort_unstable();
            prop_assert_eq!(&placed, &shots);
            for lane in &lanes {
                for pair in lane.windows(2) {
                    let (before, after) = (&sounds[pair[0]], &sounds[pair[1]]);
                    prop_assert!(before.at + before.length <= after.at);
                }
            }
            let busiest = sounds
                .iter()
                .map(|s| sounds.iter().filter(|o| o.at <= s.at && s.at < o.at + o.length).count())
                .max()
                .unwrap_or(0);
            prop_assert_eq!(lanes.len(), busiest);
        }
    }
}
