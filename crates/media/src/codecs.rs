use opsis_compile::nodes::media::{Codec, ConvertCodec, EncodePreset, SoundFormat};
use serde::Serialize;

pub const FASTSTART: [&str; 2] = ["-movflags", "+faststart"];
pub const EVEN_FRAME: &str = "crop=trunc(iw/2)*2:trunc(ih/2)*2";
pub const AAC: [&str; 2] = ["-c:a", "aac"];
pub const PCM_16: [&str; 2] = ["-c:a", "pcm_s16le"];
const AAC_EXPORT: [&str; 4] = ["-c:a", "aac", "-b:a", "320k"];
const OPUS_EXPORT: [&str; 4] = ["-c:a", "libopus", "-b:a", "192k"];
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

#[derive(Default)]
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

pub fn edit_video() -> Vec<String> {
    h264_args(&H264 {
        crf: EDIT_CRF,
        ..H264::default()
    })
}

pub fn edit_audio() -> Vec<String> {
    strings(&AAC)
}

pub struct CodecInfo {
    pub ext: &'static str,
    pub audio: Option<&'static [&'static str]>,
    pub faststart: bool,
    pub crf: bool,
}

pub fn codec_info(codec: Codec) -> CodecInfo {
    let (audio, faststart, crf): (Option<&'static [&'static str]>, bool, bool) = match codec {
        Codec::H264 | Codec::H265 => (Some(&AAC_EXPORT), true, true),
        Codec::Vp8 | Codec::Vp9 => (Some(&OPUS_EXPORT), false, true),
        Codec::Prores => (Some(&PCM_16), false, false),
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
        Codec::Prores => strings(&[
            "-c:v",
            "prores_ks",
            "-profile:v",
            "3",
            "-pix_fmt",
            "yuv422p10le",
        ]),
        Codec::Vp8 | Codec::Gif => return None,
    })
}

pub fn convert_codec(codec: ConvertCodec) -> Codec {
    match codec {
        ConvertCodec::H264 => Codec::H264,
        ConvertCodec::H265 => Codec::H265,
        ConvertCodec::Vp9 => Codec::Vp9,
        ConvertCodec::Prores => Codec::Prores,
    }
}

fn convert_video(codec: ConvertCodec, crf: f64, preset: EncodePreset) -> Vec<String> {
    codec_video(convert_codec(codec), crf, preset).unwrap_or_default()
}

pub fn has_sound_track(codec: Codec) -> bool {
    codec_info(codec).audio.is_some()
}

pub fn is_image_codec(codec: Codec) -> bool {
    codec == Codec::Gif
}

pub fn convert_args(codec: ConvertCodec, crf: f64, preset: EncodePreset) -> Vec<String> {
    let info = codec_info(convert_codec(codec));
    let mut args = convert_video(codec, crf, preset);
    if info.faststart {
        args.extend(strings(&FASTSTART));
    }
    args.extend(strings(info.audio.unwrap_or_default()));
    args
}

pub fn grade_video_args(codec: ConvertCodec, crf: f64, preset: EncodePreset) -> Vec<String> {
    let mut args = convert_video(codec, crf, preset);
    if let Some(at) = args.iter().position(|a| a == "-vf") {
        args.drain(at..at + 2);
    }
    if codec == ConvertCodec::Vp9 {
        args.extend(strings(&["-pix_fmt", "yuv420p"]));
    }
    args
}

const MP4_SOUNDS: [&str; 7] = ["aac", "mp3", "ac3", "eac3", "opus", "flac", "alac"];
const WEBM_SOUNDS: [&str; 2] = ["opus", "vorbis"];

pub fn grade_sound_args(codec: ConvertCodec, sound: Option<&str>) -> Vec<String> {
    let Some(sound) = sound.filter(|s| !s.is_empty()) else {
        return Vec::new();
    };
    let info = codec_info(convert_codec(codec));
    let keeps = match info.ext {
        "webm" => WEBM_SOUNDS.contains(&sound),
        "mov" => true,
        _ => MP4_SOUNDS.contains(&sound),
    };
    if keeps {
        strings(&["-c:a", "copy"])
    } else {
        strings(info.audio.unwrap_or_default())
    }
}

pub fn audio_format_args(format: SoundFormat) -> Vec<String> {
    match format {
        SoundFormat::M4a => strings(&["-c:a", "aac", "-b:a", "192k"]),
        SoundFormat::Mp3 => strings(&["-c:a", "libmp3lame", "-q:a", "2"]),
        SoundFormat::Wav => strings(&PCM_16),
    }
}

pub fn audio_format_for(ext: &str) -> SoundFormat {
    serde_json::from_value(serde_json::Value::String(ext.to_owned())).unwrap_or(SoundFormat::M4a)
}

pub fn ffmpeg_seconds(ms: f64) -> String {
    format!("{:.3}", ms / 1000.0)
}

#[derive(Default)]
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
            grab.width.to_string(),
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

pub fn edited_file(ext: &str, video: bool, audio: bool) -> Option<EditedFile> {
    if video {
        let mut codecs = strings(&["-vf", EVEN_FRAME]);
        codecs.extend(edit_video());
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

pub fn mux_args(codec: Codec, picture: &str, sound: &str, out: &str) -> Vec<String> {
    let mut args = strings(&[
        "-y", "-i", picture, "-i", sound, "-map", "0:v:0", "-map", "1:a:0", "-c:v", "copy",
    ]);
    args.extend(strings(codec_info(codec).audio.unwrap_or_default()));
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
