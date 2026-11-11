use opsis_compile::nodes::media::{
    Codec, ConvertCodec, Encoding, ProresProfile, SoundFormat, SoundQuality,
};
use opsis_compile::nodes::output::{PixelFormat, RenderProres};
use opsis_types::settings::EncodePreset;
use serde::Serialize;

pub const FASTSTART: [&str; 2] = ["-movflags", "+faststart"];
pub const EVEN_FRAME: &str = "crop=trunc(iw/2)*2:trunc(ih/2)*2";
pub const AAC: [&str; 2] = ["-c:a", "aac"];
pub const PCM_16: [&str; 2] = ["-c:a", "pcm_s16le"];
const AAC_EXPORT: [&str; 4] = ["-c:a", "aac", "-b:a", "320k"];
const OPUS_EXPORT: [&str; 4] = ["-c:a", "libopus", "-b:a", "192k"];
const ALAC_EXPORT: [&str; 2] = ["-c:a", "alac"];
const FLAC_EXPORT: [&str; 2] = ["-c:a", "flac"];
const EDIT_CRF: f64 = 18.0;

pub fn serde_name<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

pub fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|&a| a.to_owned()).collect()
}

pub fn even_down(size: f64) -> f64 {
    size - size % 2.0
}

pub fn with_even_frame(filter: &str) -> String {
    format!("{filter},{EVEN_FRAME}")
}

#[derive(Debug, Default)]
pub struct H264<'a> {
    pub crf: f64,
    pub preset: Option<&'a str>,
    pub keyframe_every: Option<f64>,
    pub faststart: bool,
}

pub fn h264_args(opts: &H264<'_>) -> Vec<String> {
    let mut args = strings(&["-c:v", "libx264"]);
    if let Some(preset) = opts.preset.filter(|p| !p.is_empty()) {
        args.extend(strings(&["-preset", preset]));
    }
    args.extend(["-crf".to_owned(), opts.crf.to_string()]);
    if let Some(every) = opts.keyframe_every.filter(|k| *k != 0.0) {
        args.extend(["-g".to_owned(), every.to_string()]);
    }
    args.extend(strings(&["-pix_fmt", "yuv420p"]));
    if opts.faststart {
        args.extend(strings(&FASTSTART));
    }
    args
}

pub fn edit_video(preset: EncodePreset) -> Vec<String> {
    let name = serde_name(&preset);
    h264_args(&H264 {
        crf: EDIT_CRF,
        preset: (preset != EncodePreset::Medium).then_some(name.as_str()),
        ..H264::default()
    })
}

pub fn edit_audio() -> Vec<String> {
    strings(&AAC)
}

#[derive(Debug)]
pub struct CodecInfo {
    pub ext: &'static str,
    pub audio: Option<&'static [&'static str]>,
    pub faststart: bool,
    pub crf: bool,
}

pub const fn codec_info(codec: Codec) -> CodecInfo {
    let (audio, faststart, crf): (Option<&'static [&'static str]>, bool, bool) = match codec {
        Codec::H264 | Codec::H265 => (Some(&AAC_EXPORT), true, true),
        Codec::Vp8 | Codec::Vp9 => (Some(&OPUS_EXPORT), false, true),
        Codec::Prores => (Some(&PCM_16), false, false),
        Codec::H264Lossless => (Some(&ALAC_EXPORT), true, false),
        Codec::Ffv1 => (Some(&FLAC_EXPORT), false, false),
        Codec::Gif => (None, false, false),
    };
    CodecInfo {
        ext: codec.extension(),
        audio,
        faststart,
        crf,
    }
}

pub fn codec_video(codec: Codec, crf: f64, preset: EncodePreset) -> Option<Vec<String>> {
    let crf_text = crf.to_string();
    let preset_name = serde_name(&preset);
    Some(match codec {
        Codec::H264 => {
            let mut args = strings(&["-vf", EVEN_FRAME]);
            args.extend(h264_args(&H264 {
                crf,
                preset: Some(&preset_name),
                ..H264::default()
            }));
            args
        }
        Codec::H265 => vec![
            "-vf".into(),
            EVEN_FRAME.into(),
            "-c:v".into(),
            "libx265".into(),
            "-crf".into(),
            crf_text,
            "-preset".into(),
            preset_name,
            "-tag:v".into(),
            "hvc1".into(),
            "-pix_fmt".into(),
            "yuv420p".into(),
        ],
        Codec::Vp9 => vec![
            "-c:v".into(),
            "libvpx-vp9".into(),
            "-crf".into(),
            crf_text,
            "-b:v".into(),
            "0".into(),
        ],
        Codec::Prores => prores_args(ProresProfile::Hq),
        Codec::Vp8 | Codec::Gif | Codec::H264Lossless | Codec::Ffv1 => return None,
    })
}

#[derive(Debug, Clone)]
pub struct FilmEncoding {
    pub codec: Codec,
    pub fps: f64,
    pub png_frames: bool,
    pub crf: Option<f64>,
    pub x264_preset: Option<EncodePreset>,
    pub pixel_format: Option<PixelFormat>,
    pub prores: Option<RenderProres>,
    pub gif_loops: Option<u32>,
}

const fn film_encoder(codec: Codec) -> &'static str {
    match codec {
        Codec::H264 => "libx264",
        Codec::H265 => "libx265",
        Codec::Vp8 => "libvpx",
        Codec::Vp9 => "libvpx-vp9",
        Codec::Prores => "prores_ks",
        Codec::H264Lossless => "libx264rgb",
        Codec::Ffv1 => "ffv1",
        Codec::Gif => "gif",
    }
}

pub const fn is_lossless(codec: Codec) -> bool {
    matches!(codec, Codec::H264Lossless | Codec::Ffv1)
}

fn film_pixel_format(e: &FilmEncoding) -> String {
    match e.codec {
        Codec::Prores => "yuv422p10le".to_owned(),
        Codec::H264Lossless => "bgr24".to_owned(),
        Codec::Ffv1 => "bgr0".to_owned(),
        _ => e
            .pixel_format
            .map_or_else(|| "yuv420p".to_owned(), |f| serde_name(&f)),
    }
}

const fn default_film_crf(codec: Codec) -> Option<f64> {
    match codec {
        Codec::H264 => Some(18.0),
        Codec::H265 => Some(23.0),
        Codec::Vp8 => Some(9.0),
        Codec::Vp9 => Some(28.0),
        Codec::Prores | Codec::Gif | Codec::H264Lossless | Codec::Ffv1 => None,
    }
}

const fn prores_profile_number(profile: Option<RenderProres>) -> &'static str {
    match profile {
        Some(RenderProres::Proxy) => "0",
        Some(RenderProres::Light) => "1",
        Some(RenderProres::Standard) => "2",
        None => "3",
    }
}

fn gif_loops(loops: u32) -> String {
    if loops == 0 {
        "-1".to_owned()
    } else {
        loops.to_string()
    }
}

pub fn film_encode_args(e: &FilmEncoding, output: &str) -> Vec<String> {
    let mut args = strings(&["-f", "image2pipe", "-framerate"]);
    args.push(e.fps.to_string());
    args.extend(strings(&[
        "-c:v",
        if e.png_frames { "png" } else { "mjpeg" },
        "-i",
        "-",
    ]));
    let gif = e.codec == Codec::Gif;
    if gif {
        args.extend(strings(&[
            "-filter_complex",
            "split[v],palettegen,[v]paletteuse",
        ]));
    }
    args.push("-an".to_owned());
    if let Some(loops) = e.gif_loops.filter(|_| gif) {
        args.extend(["-loop".to_owned(), gif_loops(loops)]);
    }
    args.extend(["-c:v".to_owned(), film_encoder(e.codec).to_owned()]);
    if e.codec == Codec::Prores {
        args.extend([
            "-profile:v".to_owned(),
            prores_profile_number(e.prores).to_owned(),
        ]);
    }
    if !gif {
        args.extend(strings(&["-vf", EVEN_FRAME, "-pix_fmt"]));
        args.push(film_pixel_format(e));
    }
    if e.codec == Codec::H264Lossless {
        args.extend(strings(&["-qp", "0"]));
    }
    if let Some(preset) = e.x264_preset.filter(|_| e.codec == Codec::H264) {
        args.extend(["-preset".to_owned(), serde_name(&preset)]);
    }
    if e.codec == Codec::H265 {
        args.extend(strings(&["-tag:v", "hvc1"]));
    }
    if !gif && e.codec != Codec::Ffv1 {
        args.extend(strings(&["-video_track_timescale", "90000"]));
    }
    if let Some(crf) = default_film_crf(e.codec).map(|fallback| e.crf.unwrap_or(fallback)) {
        args.extend(["-crf".to_owned(), crf.to_string()]);
    }
    args.extend(strings(&["-map_metadata", "-1", output]));
    args
}

pub const fn convert_codec(codec: ConvertCodec) -> Codec {
    match codec {
        ConvertCodec::H264 => Codec::H264,
        ConvertCodec::H265 => Codec::H265,
        ConvertCodec::Vp9 => Codec::Vp9,
        ConvertCodec::Prores => Codec::Prores,
    }
}

pub fn prores_args(profile: ProresProfile) -> Vec<String> {
    vec![
        "-c:v".into(),
        "prores_ks".into(),
        "-profile:v".into(),
        profile.number().to_string(),
        "-pix_fmt".into(),
        "yuv422p10le".into(),
    ]
}

pub fn sound_args(codec: ConvertCodec, quality: SoundQuality) -> Vec<String> {
    let rate = |aac: &str, opus: &str| match codec {
        ConvertCodec::Vp9 => opus.to_owned(),
        _ => aac.to_owned(),
    };
    let bit_rate = match quality {
        SoundQuality::Low => rate("128k", "96k"),
        SoundQuality::Medium => rate("192k", "128k"),
        SoundQuality::High => rate("320k", "192k"),
    };
    match codec {
        ConvertCodec::Prores => strings(&PCM_16),
        ConvertCodec::Vp9 => vec!["-c:a".into(), "libopus".into(), "-b:a".into(), bit_rate],
        ConvertCodec::H264 | ConvertCodec::H265 => {
            vec!["-c:a".into(), "aac".into(), "-b:a".into(), bit_rate]
        }
    }
}

fn convert_video(p: &Encoding, preset: EncodePreset) -> Vec<String> {
    if p.codec == ConvertCodec::Prores {
        return prores_args(p.prores_profile);
    }
    codec_video(convert_codec(p.codec), f64::from(p.crf), preset).unwrap_or_default()
}

pub const fn has_sound_track(codec: Codec) -> bool {
    codec_info(codec).audio.is_some()
}

pub fn is_image_codec(codec: Codec) -> bool {
    codec == Codec::Gif
}

pub fn convert_args(p: &Encoding, preset: EncodePreset) -> Vec<String> {
    let info = codec_info(convert_codec(p.codec));
    let mut args = convert_video(p, preset);
    if info.faststart {
        args.extend(strings(&FASTSTART));
    }
    args.extend(sound_args(p.codec, p.sound_quality));
    args
}

const MP4_SOUNDS: [&str; 7] = ["aac", "mp3", "ac3", "eac3", "opus", "flac", "alac"];

pub fn graded_sound_args(sound: Option<&str>) -> Vec<String> {
    match sound.filter(|s| !s.is_empty()) {
        None => Vec::new(),
        Some(sound) if MP4_SOUNDS.contains(&sound) => strings(&["-c:a", "copy"]),
        Some(_) => edit_audio(),
    }
}

pub fn audio_format_args(format: SoundFormat) -> Vec<String> {
    match format {
        SoundFormat::M4a => strings(&["-c:a", "aac", "-b:a", "192k"]),
        SoundFormat::Mp3 => strings(&["-c:a", "libmp3lame", "-q:a", "2"]),
        SoundFormat::Wav => strings(&PCM_16),
    }
}

pub fn extract_args(format: SoundFormat, kbps: u32) -> Vec<String> {
    match format {
        SoundFormat::M4a => vec![
            "-c:a".into(),
            "aac".into(),
            "-b:a".into(),
            format!("{kbps}k"),
        ],
        other => audio_format_args(other),
    }
}

pub fn audio_format_for(ext: &str) -> SoundFormat {
    serde_json::from_value(serde_json::Value::String(ext.to_owned())).unwrap_or(SoundFormat::M4a)
}

pub fn ffmpeg_seconds(ms: f64) -> String {
    format!("{:.3}", ms / 1000.0)
}

#[derive(Debug, Default)]
pub struct FrameGrab {
    pub at_ms: f64,
    pub width: f64,
    pub even_height: bool,
    pub jpeg_quality: Option<f64>,
}

pub fn frame_grab_args(input: &str, output: &str, grab: &FrameGrab) -> Vec<String> {
    let mut args = vec![
        "-ss".to_owned(),
        ffmpeg_seconds(grab.at_ms),
        "-i".to_owned(),
        input.to_owned(),
        "-frames:v".to_owned(),
        "1".to_owned(),
        "-vf".to_owned(),
        format!(
            "scale={}:{}",
            if grab.width > 0.0 {
                grab.width.to_string()
            } else {
                "iw".to_owned()
            },
            if grab.even_height { -2 } else { -1 }
        ),
    ];
    if let Some(quality) = grab.jpeg_quality.filter(|q| *q != 0.0) {
        args.extend(["-q:v".to_owned(), quality.to_string()]);
    }
    args.push(output.to_owned());
    args
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct EditedFile {
    pub ext: String,
    pub codecs: Vec<String>,
}

pub fn sound_file(ext: &str) -> EditedFile {
    let format = match ext {
        ".mp3" => Some(SoundFormat::Mp3),
        ".wav" => Some(SoundFormat::Wav),
        ".m4a" => Some(SoundFormat::M4a),
        _ => None,
    };
    let mut codecs = vec!["-vn".to_owned()];
    codecs.extend(audio_format_args(format.unwrap_or(SoundFormat::M4a)));
    EditedFile {
        ext: if format.is_some() {
            ext.to_owned()
        } else {
            ".m4a".to_owned()
        },
        codecs,
    }
}

pub fn edited_file(
    ext: &str,
    video: bool,
    audio: bool,
    preset: EncodePreset,
) -> Option<EditedFile> {
    if video {
        let mut codecs = strings(&["-vf", EVEN_FRAME]);
        codecs.extend(edit_video(preset));
        codecs.extend(if audio {
            edit_audio()
        } else {
            strings(&["-an"])
        });
        return Some(EditedFile {
            ext: ".mp4".to_owned(),
            codecs,
        });
    }
    audio.then(|| sound_file(ext))
}

pub fn mux_args(
    codec: Codec,
    picture: &str,
    sound: &str,
    out: &str,
    kbps: Option<u32>,
) -> Vec<String> {
    let mut args = strings(&[
        "-y", "-i", picture, "-i", sound, "-map", "0:v:0", "-map", "1:a:0", "-c:v", "copy",
    ]);
    let mut audio = strings(codec_info(codec).audio.unwrap_or_default());
    if let (Some(kbps), Some(at)) = (kbps, audio.iter().position(|a| a == "-b:a")) {
        audio[at + 1] = format!("{kbps}k");
    }
    args.extend(audio);
    args.extend(strings(&["-shortest", out]));
    args
}

pub fn finish_args(input: &str, out: &str, faststart: bool, chapters: Option<&str>) -> Vec<String> {
    let mut args = strings(&["-y", "-i", input]);
    if let Some(chapters) = chapters {
        args.extend(strings(&[
            "-i",
            chapters,
            "-map_metadata",
            "1",
            "-map_chapters",
            "1",
        ]));
    }
    args.extend(strings(&["-map", "0", "-c", "copy"]));
    if faststart {
        args.extend(strings(&FASTSTART));
    }
    args.push(out.to_owned());
    args
}
