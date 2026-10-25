use opsis_compile::envelope::sample_points;
use opsis_compile::mix::plan::{BusDuck, BusPlan, ChannelPlan, SoundPlacement};
use opsis_compile::time::clamp;
use opsis_types::mix::{Comp, Eq, EqType, Highpass, LoudnessGoal, Master, SAMPLE_RATE, db_to_gain};
use opsis_types::timeline::Keyframe;
use serde::{Deserialize, Serialize};

use crate::filters::{atempo_chain, loudnorm_filter};

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
    pub fps: f64,
    pub duration_in_frames: f64,
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
    if sound.offset > 0.0 {
        steps.extend([
            format!("atrim=start={}", secs(sound.offset, fps)),
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
        format!("atrim=end={}", secs(sound.length, fps)),
        "asetpts=PTS-STARTPTS".to_owned(),
    ]);
    if sound.volume != 1.0 {
        steps.push(format!("volume={}", n6(sound.volume)));
    }
    if sound.fade_in > 0.0 {
        steps.push(format!(
            "afade=t=in:st=0:d={}",
            secs(f64::min(sound.fade_in, sound.length), fps)
        ));
    }
    if sound.fade_out > 0.0 {
        let d = f64::min(sound.fade_out, sound.length);
        steps.push(format!(
            "afade=t=out:st={}:d={}",
            secs(sound.length - d, fps),
            secs(d, fps)
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

fn sound_chain(sound: &SoundPlacement, input: usize, label: &str, fps: f64) -> String {
    let steps = placement_steps(sound, fps);
    let delay = (sound.at * 1000.0 / fps).round();
    let after: Vec<String> = if delay > 0.0 {
        vec![format!("adelay=delays={}:all=1", delay.to_string())]
    } else {
        Vec::new()
    };
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
            secs(plan.duration_in_frames, plan.fps)
        ),
        output: OUT.to_owned(),
        taps: None,
    }
}

fn fit(plan: Plan) -> String {
    format!(
        "apad,atrim=end={},asetpts=PTS-STARTPTS",
        secs(plan.duration_in_frames, plan.fps)
    )
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
    let pads: String = labels.iter().map(|l| format!("[{l}]")).collect();
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

pub fn channel_job(plan: Plan, channel: &ChannelPlan, inputs: Vec<String>) -> Job {
    if channel.sounds.is_empty() {
        return silence_job(plan);
    }
    let labels: Vec<String> = (0..channel.sounds.len()).map(|i| format!("s{i}")).collect();
    let chains = channel
        .sounds
        .iter()
        .enumerate()
        .map(|(i, s)| sound_chain(s, i, &labels[i], plan.fps))
        .collect();
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
    use opsis_types::mix::{Master, Strip};
    use serde_json::{Value, json};

    use super::*;

    const FILM: Plan = Plan {
        fps: 30.0,
        duration_in_frames: 300.0,
    };
    const PADDED: &str = "[s0]anull,apad,atrim=end=10,asetpts=PTS-STARTPTS[out]";

    fn resolve(r: &str) -> String {
        format!("/files/{}", r.split(':').nth(1).unwrap_or(r))
    }

    fn sound_job(placement: Value, strip: Value) -> Job {
        let mut sound = json!({ "src": "project:a.wav", "at": 0, "offset": 0, "length": 90, "rate": 1, "loop": false, "volume": 1, "fadeIn": 0, "fadeOut": 0 });
        sound
            .as_object_mut()
            .unwrap()
            .extend(placement.as_object().unwrap().clone());
        let channel: ChannelPlan = serde_json::from_value(json!({
            "id": "a", "kind": "effect", "bus": "effects", "audible": true,
            "strip": serde_json::from_value::<Strip>(strip).unwrap(), "sounds": [sound]
        }))
        .unwrap();
        let inputs = channel.sounds.iter().map(|s| resolve(&s.src)).collect();
        channel_job(FILM, &channel, inputs)
    }

    #[test]
    fn places_a_sound_in_time_with_fades_and_volume() {
        let job = sound_job(
            json!({ "at": 30, "offset": 15, "length": 60, "volume": 0.5, "fadeIn": 6, "fadeOut": 12 }),
            json!({}),
        );
        assert_eq!(job.inputs, ["/files/a.wav"]);
        assert_eq!(
            job.filter,
            format!(
                "[0:a]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,atrim=start=0.5,asetpts=PTS-STARTPTS,atrim=end=2,asetpts=PTS-STARTPTS,volume=0.5,afade=t=in:st=0:d=0.2,afade=t=out:st=1.6:d=0.4,adelay=delays=1000:all=1[s0];{PADDED}"
            )
        );
    }

    #[test]
    fn loops_pans_and_fades_volume_with_keyframes() {
        let pan = json!([{ "frame": 0, "value": -1, "ease": "linear" }, { "frame": 30, "value": 1, "ease": "linear" }]);
        let filter = sound_job(
            json!({ "loop": true, "envelope": { "pan": pan } }),
            json!({}),
        )
        .filter;
        for part in [
            "aloop=loop=-1:size=2147483647",
            "channelsplit=channel_layout=stereo[s0l][s0r]",
            "[s0L][s0R]join=inputs=2:channel_layout=stereo[s0]",
            "min(1,1-(if(lt(t,0),-1,if(lt(t,1),-1+(2)*(t-0)/1,1))))",
        ] {
            assert!(filter.contains(part), "{part}");
        }
        let volume = json!([{ "frame": 0, "value": 1, "ease": "linear" }, { "frame": 30, "value": 0, "ease": "linear" }, { "frame": 60, "value": 0, "ease": "linear" }]);
        let filter = sound_job(json!({ "envelope": { "volume": volume } }), json!({})).filter;
        assert!(filter.contains("volume=eval=frame:volume='if(lt(t,0),1,if(lt(t,1),1+(-1)*(t-0)/1,if(lt(t,2),0,0)))'"), "{filter}");
    }

    #[test]
    fn runs_a_channel_strip_in_signal_order() {
        let strip = json!({
            "trim": 3, "highpass": { "on": true, "freq": 80 },
            "eq": { "on": true, "bands": [
                { "type": "lowshelf", "freq": 120, "gain": -2, "q": 0.7 }, { "type": "peak", "freq": 500, "gain": 0, "q": 1 },
                { "type": "peak", "freq": 2500, "gain": 3, "q": 1.4 }, { "type": "highshelf", "freq": 8000, "gain": 1.5, "q": 0.7 }
            ] },
            "comp": { "on": true, "threshold": -20, "ratio": 4, "attack": 5, "release": 100, "makeup": 6 },
            "pan": -0.5, "fader": -3
        });
        let filter = sound_job(json!({}), strip).filter;
        let pinned = "aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo";
        assert_eq!(
            filter.split(';').skip(1).collect::<Vec<_>>(),
            [
                "[s0]anull,apad,atrim=end=10,asetpts=PTS-STARTPTS,volume=3dB,highpass=f=80,lowshelf=f=120:t=q:w=0.7:g=-2,equalizer=f=2500:t=q:w=1.4:g=3,highshelf=f=8000:t=q:w=0.7:g=1.5[sum]".to_owned(),
                format!("[sum]{pinned},asplit=2[pre][c]"),
                format!("[c]acompressor=threshold=0.1:ratio=4:attack=5:release=100:makeup=1.995262,{pinned},asplit=2[post][m]"),
                "[m]pan=stereo|c0=1*c0|c1=0.5*c1,volume=-3dB[out]".to_owned(),
            ]
        );
    }

    #[test]
    fn writes_stem_arguments_with_a_length_limit() {
        let job = sound_job(json!({}), json!({}));
        let args = job_args(&job, "/out.wav", PcmCodec::F32, None, Some(11.0));
        let expected = [
            "-i",
            "/files/a.wav",
            "-filter_complex",
            &job.filter,
            "-map",
            "[out]",
            "-t",
            "11",
            "-ac",
            "2",
            "-ar",
            "48000",
            "-c:a",
            "pcm_f32le",
            "/out.wav",
        ];
        assert_eq!(args, expected);
    }

    fn bus(channels: &[&str], duck: Option<Value>, fader: f64, processing: Value) -> BusPlan {
        serde_json::from_value(json!({ "id": "music", "name": "Music", "fader": fader, "channels": channels, "duck": duck, "processing": processing })).unwrap()
    }

    fn no_processing() -> Value {
        json!({ "highpass": { "on": false, "freq": 80 }, "eq": { "on": false, "bands": [] }, "comp": { "on": false } })
    }

    #[test]
    fn ducks_a_bus_with_a_sidechain_keyed_from_other_channels() {
        let duck = json!({ "keys": ["fx", "clip"], "amount": 14, "attack": 50, "release": 400 });
        let job = bus_job(
            FILM,
            &bus(&["music"], Some(duck), -2.0, no_processing()),
            &|id| format!("/stems/{id}.wav"),
        );
        assert_eq!(
            job.inputs,
            ["/stems/music.wav", "/stems/fx.wav", "/stems/clip.wav"]
        );
        assert_eq!(
            job.filter,
            "[0:a]anull,apad,atrim=end=10,asetpts=PTS-STARTPTS,volume=-2dB[main];[1:a][2:a]amix=inputs=2:normalize=0:duration=longest:dropout_transition=0,apad,atrim=end=10,asetpts=PTS-STARTPTS[key];[main][key]sidechaincompress=threshold=0.01:ratio=2:attack=50:release=400:makeup=1,apad,atrim=end=10,asetpts=PTS-STARTPTS[out]"
        );
    }

    #[test]
    fn measures_then_normalises_loudness_on_the_master_limiting_last() {
        let master: Master = serde_json::from_value(
            json!({ "loudness": "web", "limiter": { "on": true, "ceiling": -1 } }),
        )
        .unwrap();
        assert!(
            loudness_measure_job(FILM, &master, &["/b1.wav".into()])
                .unwrap()
                .filter
                .contains("loudnorm=I=-14:TP=-1:LRA=11:print_format=json")
        );
        let measured = parse_loudnorm("junk\n{\n\t\"input_i\" : \"-20.1\",\n\t\"input_tp\" : \"-3.2\",\n\t\"input_lra\" : \"4.0\",\n\t\"input_thresh\" : \"-30.5\",\n\t\"output_i\" : \"-14.0\",\n\t\"target_offset\" : \"0.1\"\n}\n").unwrap();
        let fields = [
            &measured.input_i,
            &measured.input_tp,
            &measured.input_lra,
            &measured.input_thresh,
            &measured.target_offset,
        ];
        assert_eq!(fields, ["-20.1", "-3.2", "4.0", "-30.5", "0.1"]);
        assert_eq!(
            parse_loudnorm("{ \"input_i\" : \"-20.1\", \"input_tp\" : \"-3.2\" }"),
            None
        );
        assert_eq!(parse_loudnorm("{ \"input_i\" : \"-inf\" }"), None);
        let master: Master = serde_json::from_value(json!({ "loudness": "web", "limiter": { "on": true, "ceiling": -1 }, "comp": { "on": false } })).unwrap();
        let job = master_job(
            FILM,
            &master,
            &["/b1.wav".into(), "/b2.wav".into()],
            Some(&measured),
        );
        assert_eq!(
            job.filter,
            "[0:a][1:a]amix=inputs=2:normalize=0:duration=longest:dropout_transition=0,apad,atrim=end=10,asetpts=PTS-STARTPTS,loudnorm=I=-14:TP=-1:LRA=11:measured_I=-20.1:measured_TP=-3.2:measured_LRA=4.0:measured_thresh=-30.5:offset=0.1:linear=true:print_format=summary,aresample=48000,aresample=192000,alimiter=limit=0.891251:level=false,aresample=48000[out]"
        );
    }

    #[test]
    fn uses_silence_for_an_empty_bus_and_runs_bus_processing_before_its_fader() {
        let empty = bus_job(
            Plan {
                fps: 30.0,
                duration_in_frames: 60.0,
            },
            &bus(&[], None, 0.0, no_processing()),
            &|_| String::new(),
        );
        assert_eq!(
            empty,
            Job {
                inputs: Vec::new(),
                filter: "anullsrc=r=48000:cl=stereo,atrim=end=2[out]".into(),
                output: "out".into(),
                taps: None
            }
        );
        let processing = json!({
            "highpass": { "on": true, "freq": 60 },
            "eq": { "on": true, "bands": [
                { "type": "lowshelf", "freq": 100, "gain": 2, "q": 0.7 }, { "type": "peak", "freq": 600, "gain": 0, "q": 1 },
                { "type": "peak", "freq": 4000, "gain": 0, "q": 1 }, { "type": "highshelf", "freq": 9000, "gain": 0, "q": 0.7 }
            ] },
            "comp": { "on": true, "threshold": -16, "ratio": 4, "attack": 3, "release": 80, "makeup": 0 }
        });
        let job = bus_job(FILM, &bus(&["a"], None, -3.0, processing), &|id| {
            format!("{id}.wav")
        });
        assert!(job.filter.contains("highpass=f=60,lowshelf=f=100"));
        assert!(job.filter.find("acompressor").unwrap() < job.filter.find("volume=-3dB").unwrap());
    }
}
