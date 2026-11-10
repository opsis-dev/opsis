use opsis_compile::nodes::media::{Codec, Encoding, SoundFormat};
use opsis_media::codecs::{
    FilmEncoding, FrameGrab, H264, audio_format_args, audio_format_for, codec_info, codec_video,
    convert_args, edited_file, ffmpeg_seconds, film_encode_args, finish_args, frame_grab_args,
    graded_sound_args, h264_args, mux_args, sound_file, strings,
};
use opsis_types::numbers;
use opsis_types::settings::EncodePreset;
use serde_json::{Map, Value};

use crate::common::{cases, json, number, text, typed, with};

const CODECS: &str = "media/codecs.json";
const FILM_ENCODING: &str = "media/film-encoding.json";

const PRESETS: [EncodePreset; 5] = [
    EncodePreset::Ultrafast,
    EncodePreset::Fast,
    EncodePreset::Medium,
    EncodePreset::Slow,
    EncodePreset::Veryslow,
];

fn table(stored: &Value) -> Value {
    let rows: Map<String, Value> = stored
        .as_object()
        .expect("a codec table")
        .keys()
        .map(|name| {
            let codec: Codec = typed(&Value::String(name.clone()));
            let info = codec_info(codec);
            let video: Option<Vec<Vec<String>>> = PRESETS
                .iter()
                .map(|p| codec_video(codec, 23.0, *p))
                .collect();
            let row = serde_json::json!({
                "ext": info.ext,
                "audio": info.audio.map(strings),
                "faststart": info.faststart,
                "crf": info.crf,
                "video": video,
            });
            (name.clone(), row)
        })
        .collect();
    Value::Object(rows)
}

fn audio_formats(stored: &Value) -> Value {
    let formats: Map<String, Value> = stored
        .as_object()
        .expect("audio formats")
        .keys()
        .map(|name| {
            let format: SoundFormat = typed(&Value::String(name.clone()));
            (name.clone(), json(audio_format_args(format)))
        })
        .collect();
    Value::Object(formats)
}

#[test]
fn codec_and_file_arguments_are_as_recorded() {
    let f = opsis_golden::read(CODECS);
    let convert = cases(&f["convert"], |case| {
        let p: Encoding = typed(&serde_json::json!({
            "codec": case["codec"],
            "crf": numbers::whole(number(&case["crf"])),
        }));
        let preset: EncodePreset = typed(&case["preset"]);
        with(case, &[("convert", json(convert_args(&p, preset)))])
    });
    let grade_sound = cases(&f["gradeSound"], |case| {
        let made = graded_sound_args(case["sound"].as_str());
        with(case, &[("args", json(made))])
    });
    let h264 = cases(&f["h264"], |case| {
        let o = &case["opts"];
        let opts = H264 {
            crf: number(&o["crf"]),
            preset: o["preset"].as_str(),
            keyframe_every: o["keyframeEvery"].as_f64(),
            faststart: o["faststart"] == true,
        };
        with(case, &[("args", json(h264_args(&opts)))])
    });
    let edited = cases(&f["edited"], |case| {
        let s = &case["streams"];
        let out = edited_file(
            text(&case["ext"]),
            s["video"] == true,
            s["audio"] == true,
            EncodePreset::Medium,
        );
        with(case, &[("out", json(out))])
    });
    let sound_files = cases(&f["soundFiles"], |case| {
        with(case, &[("out", json(sound_file(text(&case["ext"]))))])
    });
    let format_for = cases(&f["audioFormatFor"], |case| {
        with(
            case,
            &[("format", json(audio_format_for(text(&case["ext"]))))],
        )
    });
    let seconds = cases(&f["seconds"], |case| {
        with(case, &[("text", json(ffmpeg_seconds(number(&case["ms"]))))])
    });
    let frame_grabs = cases(&f["frameGrabs"], |case| {
        let o = &case["opts"];
        let grab = FrameGrab {
            at_ms: number(&o["atMs"]),
            width: number(&o["width"]),
            even_height: o["evenHeight"] == true,
            jpeg_quality: o["jpegQuality"].as_f64(),
        };
        let made = frame_grab_args("in.mp4", "out.jpg", &grab);
        with(case, &[("args", json(made))])
    });
    let mux = cases(&f["mux"], |case| {
        let codec: Codec = typed(&case["codec"]);
        let made = mux_args(codec, "p.mp4", "m.wav", "o.mp4", None);
        with(case, &[("args", json(made))])
    });
    let finish = cases(&f["finish"], |case| {
        let o = &case["opts"];
        let made = finish_args(
            "a.mp4",
            "b.mp4",
            o["faststart"] == true,
            o["chapters"].as_str(),
        );
        with(case, &[("args", json(made))])
    });
    let made = serde_json::json!({
        "table": table(&f["table"]),
        "convert": convert,
        "gradeSound": grade_sound,
        "h264": h264,
        "edited": edited,
        "soundFiles": sound_files,
        "audioFormats": audio_formats(&f["audioFormats"]),
        "audioFormatFor": format_for,
        "seconds": seconds,
        "frameGrabs": frame_grabs,
        "mux": mux,
        "finish": finish,
    });
    opsis_golden::check(CODECS, &made);
}

fn film_encoding(e: &Value) -> FilmEncoding {
    FilmEncoding {
        codec: typed(&e["codec"]),
        fps: number(&e["fps"]),
        png_frames: e["pngFrames"] == true,
        crf: e["crf"].as_f64(),
        x264_preset: typed(&e["x264Preset"]),
        pixel_format: typed(&e["pixelFormat"]),
        prores: typed(&e["prores"]),
        gif_loops: e["gifLoops"].as_u64().and_then(|n| u32::try_from(n).ok()),
    }
}

#[test]
fn film_encoding_arguments_are_as_recorded() {
    let f = opsis_golden::read(FILM_ENCODING);
    let made = cases(&f["cases"], |case| {
        let args = film_encode_args(&film_encoding(&case["encoding"]), "film.out");
        with(case, &[("args", json(args))])
    });
    opsis_golden::check(FILM_ENCODING, &serde_json::json!({ "cases": made }));
}
