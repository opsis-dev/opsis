use opsis_compile::color::css_hex;
use opsis_compile::nodes::media::{Dither, ScaleMode};

use crate::codecs::{edit_video, even_down, ffmpeg_seconds, serde_name, with_even_frame};
use opsis_compile::text::round_to;

const JOIN_AUDIO: &str = "aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo";

pub fn ffmpeg_color(color: &str) -> String {
    css_hex(color.trim()).unwrap_or_else(|| "black".to_owned())
}

fn pad_to(w: f64, h: f64, color: Option<&str>) -> String {
    let color = color.map(|c| format!(":color={c}")).unwrap_or_default();
    let (w, h) = (w.to_string(), h.to_string());
    format!(
        "scale={w}:{h}:force_original_aspect_ratio=decrease,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2{color},setsar=1"
    )
}

pub fn fit_filter(mode: ScaleMode, width: f64, height: f64, background: &str) -> String {
    match mode {
        ScaleMode::Pad => {
            let color = (!background.is_empty()).then(|| ffmpeg_color(background));
            pad_to(width, height, color.as_deref())
        }
        ScaleMode::Crop => {
            let (w, h) = (width.to_string(), height.to_string());
            format!("scale={w}:{h}:force_original_aspect_ratio=increase,crop={w}:{h},setsar=1")
        }
        ScaleMode::Stretch => format!("scale={}:{},setsar=1", width, height),
    }
}

pub fn gif_filter(fps: f64, width: f64, dither: Dither) -> String {
    format!(
        "fps={},scale={}:-1:flags=lanczos,split[a][b];[a]palettegen=stats_mode=diff[pal];[b][pal]paletteuse=dither={}",
        fps,
        width,
        serde_name(&dither)
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

pub fn join_filter(parts: &[JoinPart], to: JoinSize) -> Join {
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
                    pad_to(width, height, None)
                )
            } else {
                format!(
                    "color=c=black:s={}x{}:r={rate}:d={seconds},format=yuv420p[v{i}]",
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

pub fn speed_args(input: &str, output: &str, rate: f64, fps: f64, sound: bool) -> Vec<String> {
    let mut args = vec![
        "-i".to_owned(),
        input.to_owned(),
        "-vf".to_owned(),
        with_even_frame(&format!("setpts=PTS/{}", rate)),
        "-r".to_owned(),
        (round_to(fps, 3)).to_string(),
    ];
    if sound {
        args.extend(["-af".to_owned(), atempo_chain(rate)]);
    } else {
        args.push("-an".to_owned());
    }
    args.extend(edit_video());
    args.push(output.to_owned());
    args
}

pub fn speed_sound_args(input: &str, output: &str, rate: f64, codecs: &[String]) -> Vec<String> {
    let mut args = vec![
        "-i".to_owned(),
        input.to_owned(),
        "-af".to_owned(),
        atempo_chain(rate),
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

pub const HUM_DEPTH_DB: f64 = -40.0;
pub const HUM_LISTEN_WIDTH_HZ: f64 = 4.0;

const RUMBLE_HZ: u32 = 80;
const SPEECH_NOISE_REDUCTION_DB: f64 = 40.0;
const SPEECH_NOISE_FLOOR_DB: f64 = -50.0;
const FLOOR_ABOVE_NOISE_DB: f64 = 3.0;
const QUIET_SHARE: f64 = 0.1;
const PITCH_TAIL_S: u32 = 1;
const SPEECH_LEVEL: &str = "speechnorm=e=6.25:r=0.00001:l=1:t=0.01";

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

pub fn hum_filter(hertz: f64, harmonics: u32, width: f64) -> String {
    (1..=harmonics + 1)
        .map(|n| {
            format!(
                "equalizer=f={}:t=h:w={width}:g={HUM_DEPTH_DB}",
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
    steps.push(format!("highpass=f={RUMBLE_HZ}"));
    if p.soften_s > 0.0 {
        steps.push(format!("deesser=i={}", round_to(p.soften_s, 2)));
    }
    if p.even_level {
        steps.push(SPEECH_LEVEL.to_owned());
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
    let at = (levels.len() as f64 * QUIET_SHARE) as usize;
    levels.get(at).map(|db| round_to(*db, 1))
}

pub fn sound_picture_graph(p: &opsis_compile::nodes::media::SoundPicture) -> String {
    use opsis_compile::nodes::media::SoundPictureKind;
    let size = format!("{}x{}", p.width, p.height);
    let drawn = match p.kind {
        SoundPictureKind::Waveform => format!(
            "showwavespic=s={size}:split_channels=0:colors={}",
            ffmpeg_color(&p.color)
        ),
        SoundPictureKind::Spectrum => format!(
            "showspectrumpic=s={size}:legend=0:color={},scale={size}",
            p.colours.id()
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
