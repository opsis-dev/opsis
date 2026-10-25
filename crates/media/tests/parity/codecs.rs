use crate::common::{args, fixture, list, number, text, typed};
use opsis_compile::nodes::media::{Codec, ConvertCodec, EncodePreset, SoundFormat};
use opsis_media::codecs::{
    EditedFile, FrameGrab, H264, audio_format_args, audio_format_for, codec_info, codec_video,
    convert_args, edited_file, ffmpeg_seconds, finish_args, frame_grab_args, grade_sound_args,
    grade_video_args, h264_args, mux_args, sound_file,
};
use pretty_assertions::assert_eq;
use serde_json::Value;

const PRESETS: [EncodePreset; 5] = [
    EncodePreset::Ultrafast,
    EncodePreset::Fast,
    EncodePreset::Medium,
    EncodePreset::Slow,
    EncodePreset::Veryslow,
];

#[test]
fn the_codec_table_matches() {
    let f = fixture("media/codecs.json");
    for (name, row) in f["table"].as_object().unwrap() {
        let codec: Codec = typed(&Value::String(name.clone()));
        let info = codec_info(codec);
        assert_eq!(info.ext, text(&row["ext"]), "{name}");
        assert_eq!(
            info.audio.map(opsis_media::codecs::strings),
            typed::<Option<Vec<String>>>(&row["audio"]),
            "{name}"
        );
        assert_eq!(
            (info.faststart, info.crf),
            (row["faststart"] == true, row["crf"] == true),
            "{name}"
        );
        let video: Option<Vec<Vec<String>>> = PRESETS
            .iter()
            .map(|p| codec_video(codec, 23.0, *p))
            .collect::<Option<Vec<_>>>();
        assert_eq!(
            video,
            typed::<Option<Vec<Vec<String>>>>(&row["video"]),
            "{name}"
        );
    }
}

#[test]
fn convert_and_grade_arguments_match() {
    let f = fixture("media/codecs.json");
    for case in list(&f["convert"]) {
        let codec: ConvertCodec = typed(&case["codec"]);
        let preset: EncodePreset = typed(&case["preset"]);
        let crf = number(&case["crf"]);
        assert_eq!(
            convert_args(codec, crf, preset),
            args(&case["convert"]),
            "{case}"
        );
        assert_eq!(
            grade_video_args(codec, crf, preset),
            args(&case["gradeVideo"]),
            "{case}"
        );
    }
    for case in list(&f["gradeSound"]) {
        let codec: ConvertCodec = typed(&case["codec"]);
        assert_eq!(
            grade_sound_args(codec, case["sound"].as_str()),
            args(&case["args"]),
            "{case}"
        );
    }
}

#[test]
fn encoder_and_file_arguments_match() {
    let f = fixture("media/codecs.json");
    for case in list(&f["h264"]) {
        let o = &case["opts"];
        let opts = H264 {
            crf: number(&o["crf"]),
            preset: o["preset"].as_str(),
            keyframe_every: o["keyframeEvery"].as_f64(),
            faststart: o["faststart"] == true,
        };
        assert_eq!(h264_args(&opts), args(&case["args"]), "{case}");
    }
    for case in list(&f["edited"]) {
        let s = &case["streams"];
        let out = edited_file(text(&case["ext"]), s["video"] == true, s["audio"] == true);
        assert_eq!(out, typed::<Option<EditedFile>>(&case["out"]), "{case}");
    }
    for case in list(&f["soundFiles"]) {
        assert_eq!(
            sound_file(text(&case["ext"])),
            typed::<EditedFile>(&case["out"]),
            "{case}"
        );
    }
    for (name, expected) in f["audioFormats"].as_object().unwrap() {
        let format: SoundFormat = typed(&Value::String(name.clone()));
        assert_eq!(audio_format_args(format), args(expected), "{name}");
    }
    for case in list(&f["audioFormatFor"]) {
        assert_eq!(
            audio_format_for(text(&case["ext"])),
            typed::<SoundFormat>(&case["format"]),
            "{case}"
        );
    }
    for case in list(&f["seconds"]) {
        assert_eq!(
            ffmpeg_seconds(number(&case["ms"])),
            text(&case["text"]),
            "{case}"
        );
    }
    for case in list(&f["frameGrabs"]) {
        let o = &case["opts"];
        let grab = FrameGrab {
            at_ms: number(&o["atMs"]),
            width: number(&o["width"]),
            even_height: o["evenHeight"] == true,
            jpeg_quality: o["jpegQuality"].as_f64(),
        };
        assert_eq!(
            frame_grab_args("in.mp4", "out.jpg", &grab),
            args(&case["args"]),
            "{case}"
        );
    }
    for case in list(&f["mux"]) {
        let codec: Codec = typed(&case["codec"]);
        assert_eq!(
            mux_args(codec, "p.mp4", "m.wav", "o.mp4"),
            args(&case["args"]),
            "{case}"
        );
    }
    for case in list(&f["finish"]) {
        let o = &case["opts"];
        assert_eq!(
            finish_args(
                "a.mp4",
                "b.mp4",
                o["faststart"] == true,
                o["chapters"].as_str()
            ),
            args(&case["args"]),
            "{case}"
        );
    }
}
