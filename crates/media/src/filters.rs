use opsis_compile::color::css_hex;
use opsis_compile::nodes::media::{Dither, ScaleMode};

use crate::codecs::{edit_video, even_down, serde_name, with_even_frame};
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
