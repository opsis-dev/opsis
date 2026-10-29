use opsis_compile::color::css_hex;
use opsis_compile::nodes::image::PlaceAlign;
use opsis_compile::nodes::media::{Dither, PaletteFrom, ScaleMode};

use crate::codecs::{edit_video, even_down, ffmpeg_seconds, serde_name, with_even_frame};
use opsis_compile::text::round_to;
use opsis_types::numbers;

const JOIN_AUDIO: &str = "aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo";

pub fn ffmpeg_color(color: &str) -> String {
    css_hex(color.trim()).unwrap_or_else(|| "black".to_owned())
}

fn placed(free: &str, align: PlaceAlign) -> String {
    match align {
        PlaceAlign::Start => "0".to_owned(),
        PlaceAlign::Center => format!("{free}/2"),
        PlaceAlign::End => free.to_owned(),
    }
}

fn pad_to(w: f64, h: f64, color: Option<&str>, align: PlaceAlign) -> String {
    let color = color.map(|c| format!(":color={c}")).unwrap_or_default();
    let (x, y) = (placed("(ow-iw)", align), placed("(oh-ih)", align));
    let (w, h) = (w.to_string(), h.to_string());
    format!(
        "scale={w}:{h}:force_original_aspect_ratio=decrease,pad={w}:{h}:{x}:{y}{color},setsar=1"
    )
}

pub fn fit_filter(
    mode: ScaleMode,
    width: f64,
    height: f64,
    background: &str,
    align: PlaceAlign,
) -> String {
    match mode {
        ScaleMode::Pad => {
            let color = (!background.is_empty()).then(|| ffmpeg_color(background));
            pad_to(width, height, color.as_deref(), align)
        }
        ScaleMode::Crop => {
            let (w, h) = (width.to_string(), height.to_string());
            let at = match align {
                PlaceAlign::Center => String::new(),
                PlaceAlign::Start => ":0:0".to_owned(),
                PlaceAlign::End => ":iw-ow:ih-oh".to_owned(),
            };
            format!("scale={w}:{h}:force_original_aspect_ratio=increase,crop={w}:{h}{at},setsar=1")
        }
        ScaleMode::Stretch => format!("scale={}:{},setsar=1", width, height),
    }
}

pub struct GifLook {
    pub dither: Dither,
    pub colours: u32,
    pub palette_from: PaletteFrom,
}

pub fn gif_filter(fps: f64, width: f64, look: &GifLook) -> String {
    let colours = if look.colours >= 256 {
        String::new()
    } else {
        format!(":max_colors={}", look.colours.max(2))
    };
    let stats = match look.palette_from {
        PaletteFrom::Moving => "diff",
        PaletteFrom::Whole => "full",
    };
    format!(
        "fps={},scale={}:-1:flags=lanczos,split[a][b];[a]palettegen=stats_mode={stats}{colours}[pal];[b][pal]paletteuse=dither={}",
        fps,
        width,
        serde_name(&look.dither)
    )
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinPart {
    pub video: bool,
    pub audio: bool,
    pub duration_ms: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
pub struct JoinSize {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Join {
    pub filter: String,
    pub video: bool,
    pub audio: bool,
}

pub fn join_filter(parts: &[JoinPart], to: JoinSize, background: &str) -> Join {
    let background = ffmpeg_color(background);
    let rate = (round_to(to.fps, 3)).to_string();
    let (width, height) = (
        even_down(f64::from(to.width)),
        even_down(f64::from(to.height)),
    );
    let video = parts.iter().any(|p| p.video);
    let audio = parts.iter().any(|p| p.audio);
    let mut chains = Vec::new();
    let mut pads = String::new();
    for (i, p) in parts.iter().enumerate() {
        let seconds = (round_to(p.duration_ms / 1000.0, 3)).to_string();
        if video {
            chains.push(if p.video {
                format!(
                    "[{i}:v]{},fps={rate},format=yuv420p[v{i}]",
                    pad_to(width, height, Some(&background), PlaceAlign::Center)
                )
            } else {
                format!(
                    "color=c={background}:s={}x{}:r={rate}:d={seconds},format=yuv420p[v{i}]",
                    width, height
                )
            });
            pads.push_str(&format!("[v{i}]"));
        }
        if audio {
            chains.push(if p.audio {
                format!("[{i}:a]{JOIN_AUDIO}[a{i}]")
            } else {
                format!("anullsrc=r=48000:cl=stereo,atrim=duration={seconds}[a{i}]")
            });
            pads.push_str(&format!("[a{i}]"));
        }
    }
    let out = format!(
        "{}{}",
        if video { "[v]" } else { "" },
        if audio { "[a]" } else { "" }
    );
    let join = format!(
        "{pads}concat=n={}:v={}:a={}{out}",
        parts.len(),
        u8::from(video),
        u8::from(audio)
    );
    Join {
        filter: format!("{};{join}", chains.join(";")),
        video,
        audio,
    }
}

pub fn atempo_chain(rate: f64) -> String {
    let mut parts = Vec::new();
    let mut r = rate;
    while r > 2.0 {
        parts.push(2.0);
        r /= 2.0;
    }
    while r < 0.5 {
        parts.push(0.5);
        r /= 0.5;
    }
    parts.push(r);
    parts
        .iter()
        .map(|x| format!("atempo={x:.4}"))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn retime_sound(rate: f64, keep_pitch: bool) -> String {
    if keep_pitch {
        atempo_chain(rate)
    } else {
        format!("rubberband=tempo={rate:.4}:pitch={rate:.4}")
    }
}

pub fn speed_args(
    input: &str,
    output: &str,
    rate: f64,
    fps: f64,
    sound: Option<bool>,
) -> Vec<String> {
    let mut args = vec![
        "-i".to_owned(),
        input.to_owned(),
        "-vf".to_owned(),
        with_even_frame(&format!("setpts=PTS/{}", rate)),
        "-r".to_owned(),
        (round_to(fps, 3)).to_string(),
    ];
    if let Some(keep_pitch) = sound {
        args.extend(["-af".to_owned(), retime_sound(rate, keep_pitch)]);
    } else {
        args.push("-an".to_owned());
    }
    args.extend(edit_video());
    args.push(output.to_owned());
    args
}

pub fn speed_sound_args(
    input: &str,
    output: &str,
    rate: f64,
    keep_pitch: bool,
    codecs: &[String],
) -> Vec<String> {
    let mut args = vec![
        "-i".to_owned(),
        input.to_owned(),
        "-af".to_owned(),
        retime_sound(rate, keep_pitch),
    ];
    args.extend(codecs.iter().cloned());
    args.push(output.to_owned());
    args
}

pub fn loudnorm_filter(lufs: f64, true_peak: f64, range: f64) -> String {
    format!("loudnorm=I={}:TP={}:LRA={}", lufs, true_peak, range)
}

pub fn repair_filter(clicks: bool, clipping: bool, strength: f64) -> Option<String> {
    let s = strength.clamp(0.0, 1.0);
    let mut chain = Vec::new();
    if clicks {
        chain.push(format!("adeclick=t={:.2}", 4.0 - 3.0 * s));
    }
    if clipping {
        chain.push(format!("adeclip=t={:.2}", 15.0 - 10.0 * s));
    }
    (!chain.is_empty()).then(|| chain.join(","))
}

pub const HUM_LISTEN_WIDTH_HZ: f64 = 4.0;

const SPEECH_NOISE_REDUCTION_DB: f64 = 40.0;
const SPEECH_NOISE_FLOOR_DB: f64 = -50.0;
const FLOOR_ABOVE_NOISE_DB: f64 = 3.0;
const QUIET_SHARE: f64 = 0.1;
const PITCH_TAIL_S: u32 = 1;
fn speech_level(lift_db: f64) -> String {
    let expansion = round_to(10f64.powf(lift_db / 20.0), 2).max(1.0);
    format!("speechnorm=e={expansion}:r=0.00001:l=1:t=0.01")
}

pub fn pitch_filter(
    p: &opsis_compile::nodes::media::ChangePitch,
    length_ms: Option<u32>,
) -> String {
    let formant = if p.keep_character {
        "preserved"
    } else {
        "shifted"
    };
    let pitch = format!(
        "rubberband=pitch={}:formant={formant}:pitchq=quality:transients=mixed",
        round_to(opsis_compile::nodes::media::pitch_ratio(p.semitones), 6)
    );
    match length_ms {
        Some(ms) => format!(
            "apad=pad_dur={PITCH_TAIL_S},{pitch},atrim=end={}",
            ffmpeg_seconds(f64::from(ms))
        ),
        None => pitch,
    }
}

fn telephone(grit: f64) -> String {
    format!(
        "highpass=f=300,highpass=f=300,lowpass=f=3400,lowpass=f=3400,acrusher=bits=8:mode=log:aa=1:mix={},compand=attacks=0.01:decays=0.2:points=-80/-80|-30/-15|0/-3",
        round_to((0.3 * grit).clamp(0.0, 1.0), 2)
    )
}

fn radio(grit: f64) -> String {
    format!(
        "highpass=f=200,lowpass=f=5000,acrusher=bits=10:mode=log:aa=1:mix={},compand=attacks=0.01:decays=0.3:points=-80/-80|-35/-20|0/-4",
        round_to((0.2 * grit).clamp(0.0, 1.0), 2)
    )
}

fn echo(p: &opsis_compile::nodes::media::VoiceEffects) -> String {
    let repeats = 1..=p.repeats.max(1);
    let delays: Vec<String> = repeats
        .clone()
        .map(|n| (p.delay_ms * n).to_string())
        .collect();
    let decays: Vec<String> = repeats
        .map(|n| round_to(p.decay.powi(numbers::signed(n)), 2).to_string())
        .collect();
    format!(
        "aecho=in_gain=1:out_gain=1:delays={}:decays={}",
        delays.join("|"),
        decays.join("|")
    )
}

fn mixed_with_original(chain: &str, mix: f64) -> String {
    let wet = round_to(mix.clamp(0.0, 1.0), 2);
    if wet >= 1.0 {
        return chain.to_owned();
    }
    if wet <= 0.0 {
        return "anull".to_owned();
    }
    format!(
        "[in]asplit[dry][wet];[wet]{chain}[effect];[dry][effect]amix=inputs=2:weights='{} {wet}':normalize=0:duration=first[out]",
        round_to(1.0 - wet, 2)
    )
}

fn in_room(room: &str, mix: f64) -> String {
    let wet = round_to(mix.clamp(0.0, 1.0), 2);
    let source = format!("amovie=filename={room}[room]");
    if wet <= 0.0 {
        return "anull".to_owned();
    }
    if wet >= 1.0 {
        return format!("{source};[in][room]afir=gtype=none:irnorm=-1[out]");
    }
    format!(
        "{source};[in]asplit[dry][wet];[wet][room]afir=gtype=none:irnorm=-1[effect];[dry][effect]amix=inputs=2:weights='{} {wet}':normalize=0:duration=first[out]",
        round_to(1.0 - wet, 2)
    )
}

pub fn voice_filter(p: &opsis_compile::nodes::media::VoiceEffects, room: Option<&str>) -> String {
    use opsis_compile::nodes::media::VoiceEffect;
    match (p.effect, room) {
        (VoiceEffect::Telephone, _) => mixed_with_original(&telephone(p.grit), p.mix),
        (VoiceEffect::Radio, _) => mixed_with_original(&radio(p.grit), p.mix),
        (VoiceEffect::Echo, _) => mixed_with_original(&echo(p), p.mix),
        (VoiceEffect::SmallRoom | VoiceEffect::Hall, Some(path)) => in_room(path, p.mix),
        (VoiceEffect::SmallRoom | VoiceEffect::Hall, None) => "anull".to_owned(),
    }
}

pub fn fit_inputs(plan: opsis_compile::nodes::media::FitPlan) -> u32 {
    match plan {
        opsis_compile::nodes::media::FitPlan::Loop { repeats, .. } => repeats + 1,
        _ => 1,
    }
}

pub fn fit_graph(
    plan: opsis_compile::nodes::media::FitPlan,
    target_ms: f64,
    p: &opsis_compile::nodes::media::FitMusic,
) -> String {
    use opsis_compile::nodes::media::FitPlan;
    let end = ffmpeg_seconds(target_ms);
    let fade = f64::from(p.fade_out_ms).min(target_ms);
    let ending = if fade > 0.0 {
        format!(
            ",afade=t=out:st={}:d={}{}",
            ffmpeg_seconds(target_ms - fade),
            ffmpeg_seconds(fade),
            match p.fade_curve {
                opsis_compile::nodes::media::FadeCurve::Tri => String::new(),
                curve => format!(":curve={}", serde_name(&curve)),
            }
        )
    } else {
        String::new()
    };
    match plan {
        FitPlan::Stretch { tempo } => format!(
            "[0:a]apad=pad_dur={PITCH_TAIL_S},rubberband=tempo={}:pitchq=quality:transients=mixed,atrim=end={end}{ending}[out]",
            round_to(tempo, 6)
        ),
        FitPlan::Cut => format!("[0:a]atrim=end={end}{ending}[out]"),
        FitPlan::Loop {
            from_ms,
            to_ms,
            repeats,
        } => {
            let pieces = repeats + 1;
            let mut parts = vec![format!(
                "[0:a]atrim=end={},asetpts=PTS-STARTPTS[p0]",
                ffmpeg_seconds(to_ms)
            )];
            for i in 1..pieces {
                parts.push(format!(
                    "[{i}:a]atrim=start={}:end={},asetpts=PTS-STARTPTS[p{i}]",
                    ffmpeg_seconds(from_ms),
                    ffmpeg_seconds(to_ms)
                ));
            }
            let mut last = "p0".to_owned();
            for i in 1..pieces {
                parts.push(format!(
                    "[{last}][p{i}]acrossfade=d={}:c1=qsin:c2=qsin[x{i}]",
                    ffmpeg_seconds(f64::from(p.crossfade_ms))
                ));
                last = format!("x{i}");
            }
            parts.push(format!("[{last}]atrim=end={end}{ending}[out]"));
            parts.join(";")
        }
    }
}

pub fn hum_filter(hertz: f64, harmonics: u32, width: f64, depth: f64) -> String {
    (1..=harmonics + 1)
        .map(|n| {
            format!(
                "equalizer=f={}:t=h:w={width}:g={depth}",
                hertz * f64::from(n)
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub fn hum_listen_filter(hertz: f64) -> String {
    format!(
        "bandpass=f={hertz}:t=h:w={HUM_LISTEN_WIDTH_HZ},astats=measure_perchannel=none:measure_overall=RMS_level"
    )
}

pub fn rms_level(lines: &[String]) -> Option<f64> {
    lines
        .iter()
        .rev()
        .find_map(|line| line.split("RMS level dB:").nth(1))
        .and_then(|value| value.trim().parse().ok())
}

pub fn spectral_floor(noise_level_db: Option<f64>) -> f64 {
    noise_level_db.map_or(SPEECH_NOISE_FLOOR_DB, |level| {
        round_to(level + FLOOR_ABOVE_NOISE_DB, 1).clamp(-80.0, -20.0)
    })
}

pub fn speech_filter(
    p: &opsis_compile::nodes::media::CleanSpeech,
    spectral_floor_db: Option<f64>,
) -> String {
    let mut steps = Vec::new();
    if let Some(floor) = spectral_floor_db.filter(|_| p.noise > 0.0) {
        steps.push(format!(
            "afftdn=nr={}:nf={floor}",
            round_to(p.noise * SPEECH_NOISE_REDUCTION_DB, 1)
        ));
    }
    if p.rumble_hz > 0 {
        steps.push(format!("highpass=f={}", p.rumble_hz));
    }
    if p.soften_s > 0.0 {
        steps.push(format!("deesser=i={}", round_to(p.soften_s, 2)));
    }
    if p.even_level {
        steps.push(speech_level(p.even_lift));
    }
    steps.join(",")
}

pub fn noise_level_listen_filter() -> &'static str {
    "aresample=48000,asetnsamples=n=960,astats=metadata=1:reset=1:measure_perchannel=none:measure_overall=RMS_level,ametadata=mode=print:key=lavfi.astats.Overall.RMS_level"
}

pub fn noise_level(lines: &[String]) -> Option<f64> {
    let mut levels: Vec<f64> = lines
        .iter()
        .filter_map(|line| line.split("lavfi.astats.Overall.RMS_level=").nth(1))
        .filter_map(|value| value.trim().parse::<f64>().ok())
        .filter(|db| db.is_finite())
        .collect();
    levels.sort_by(f64::total_cmp);
    let at = numbers::truncate::<usize>(numbers::float(levels.len()) * QUIET_SHARE);
    levels.get(at).map(|db| round_to(*db, 1))
}

pub fn sound_picture_graph(p: &opsis_compile::nodes::media::SoundPicture) -> String {
    use opsis_compile::nodes::media::SoundPictureKind;
    let size = format!("{}x{}", p.width, p.height);
    let drawn = match p.kind {
        SoundPictureKind::Waveform => format!(
            "showwavespic=s={size}:split_channels={}:colors={}{}",
            u8::from(p.split_channels),
            ffmpeg_color(&p.color),
            match p.wave_scale {
                opsis_compile::nodes::media::WaveScale::Lin => String::new(),
                scale => format!(":scale={}", serde_name(&scale)),
            }
        ),
        SoundPictureKind::Spectrum => format!(
            "showspectrumpic=s={size}:legend=0:color={}{},scale={size}",
            p.colours.id(),
            match p.pitch_scale {
                opsis_compile::nodes::media::PitchScale::Lin => String::new(),
                scale => format!(":fscale={}", serde_name(&scale)),
            }
        ),
    };
    format!(
        "[0:a]{drawn},format=rgba[drawn];color=c={}:s={size},format=rgba[ground];[ground][drawn]overlay=format=auto,format=rgba[out]",
        ffmpeg_color(&p.background)
    )
}

pub const PAUSE_CROSSFADE_S: f64 = 0.01;

pub fn silence_listen_filter(threshold_db: f64, longer_than_ms: u32) -> String {
    format!(
        "silencedetect=n={threshold_db}dB:d={}",
        f64::from(longer_than_ms) / 1000.0
    )
}

pub fn silences(lines: &[String]) -> Vec<opsis_compile::nodes::media::Pause> {
    let number = |line: &str, key: &str| -> Option<f64> {
        line.split(key)
            .nth(1)?
            .split_whitespace()
            .next()?
            .trim_end_matches('|')
            .parse()
            .ok()
    };
    let mut found = Vec::new();
    let mut start = None;
    for line in lines {
        if let Some(at) = number(line, "silence_start:") {
            start = Some(at.max(0.0));
        } else if let (Some(from), Some(to)) = (start, number(line, "silence_end:")) {
            found.push(opsis_compile::nodes::media::Pause {
                start_ms: from * 1000.0,
                end_ms: to * 1000.0,
            });
            start = None;
        }
    }
    found
}

pub fn joined_parts_graph(parts: &[(f64, f64)]) -> String {
    let mut chains: Vec<String> = parts
        .iter()
        .enumerate()
        .map(|(i, (from, to))| {
            format!(
                "[0:a]atrim=start={:.3}:end={:.3},asetpts=PTS-STARTPTS[p{i}]",
                from / 1000.0,
                to / 1000.0
            )
        })
        .collect();
    let mut last = "p0".to_owned();
    for i in 1..parts.len() {
        let next = format!("j{i}");
        chains.push(format!(
            "[{last}][p{i}]acrossfade=d={PAUSE_CROSSFADE_S}[{next}]"
        ));
        last = next;
    }
    chains.push(format!("[{last}]anull[out]"));
    chains.join(";")
}
