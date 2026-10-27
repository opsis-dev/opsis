use std::collections::BTreeSet;
use std::path::Path;

use opsis_engine::tools::{ToolProblem, ffmpeg_version, tool_output, unrecognized};
use opsis_types::engine::EngineTool;

pub const REQUIRED_ENCODERS: [&str; 12] = [
    "aac",
    "gif",
    "libmp3lame",
    "libopus",
    "libvpx-vp9",
    "libx264",
    "libx265",
    "mjpeg",
    "pcm_f32le",
    "pcm_s16le",
    "png",
    "prores_ks",
];

pub const REQUIRED_DECODERS: [&str; 4] = ["aac", "h264", "mjpeg", "png"];

pub const REQUIRED_FORMATS: [&str; 10] = [
    "concat",
    "f32le",
    "gif",
    "image2",
    "image2pipe",
    "mov",
    "mp4",
    "null",
    "wav",
    "webm",
];

pub const REQUIRED_FILTERS: [&str; 68] = [
    "acompressor",
    "adeclick",
    "adeclip",
    "adelay",
    "afade",
    "aformat",
    "alimiter",
    "aloop",
    "amix",
    "anull",
    "anullsrc",
    "apad",
    "aresample",
    "areverse",
    "asetpts",
    "asetrate",
    "asplit",
    "atempo",
    "atrim",
    "blend",
    "cas",
    "channelsplit",
    "color",
    "colorbalance",
    "colorchannelmixer",
    "colorize",
    "colortemperature",
    "concat",
    "crop",
    "curves",
    "deband",
    "eq",
    "equalizer",
    "exposure",
    "format",
    "fps",
    "gblur",
    "grayworld",
    "haldclut",
    "highpass",
    "highshelf",
    "hue",
    "join",
    "lagfun",
    "lenscorrection",
    "loudnorm",
    "lowshelf",
    "lut3d",
    "monochrome",
    "noise",
    "normalize",
    "null",
    "pad",
    "palettegen",
    "paletteuse",
    "pan",
    "reverse",
    "rgbashift",
    "scale",
    "setpts",
    "setsar",
    "sidechaincompress",
    "split",
    "tile",
    "trim",
    "vibrance",
    "vignette",
    "volume",
];

fn listed(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let flags = words.next()?;
            let flag_like = flags.len() <= 6
                && flags
                    .chars()
                    .all(|c| c == '.' || c.is_ascii_uppercase() || c == 'd');
            flag_like.then(|| words.next()).flatten()
        })
        .flat_map(|names| names.split(',').map(str::to_owned).collect::<Vec<_>>())
        .collect()
}

fn missing(required: &[&str], present: &BTreeSet<String>) -> Vec<String> {
    required
        .iter()
        .filter(|name| !present.contains(**name))
        .map(|name| (*name).to_owned())
        .collect()
}

async fn listing(path: &Path, flag: &str) -> Result<BTreeSet<String>, ToolProblem> {
    Ok(listed(&tool_output(path, &["-hide_banner", flag]).await?))
}

pub async fn check_ffmpeg(path: &Path) -> Result<String, ToolProblem> {
    let version = ffmpeg_version(&tool_output(path, &["-version"]).await?, "ffmpeg")
        .ok_or_else(|| unrecognized(path, EngineTool::Ffmpeg))?;
    let mut lacking = Vec::new();
    for (flag, required) in [
        ("-encoders", &REQUIRED_ENCODERS[..]),
        ("-decoders", &REQUIRED_DECODERS[..]),
        ("-formats", &REQUIRED_FORMATS[..]),
        ("-filters", &REQUIRED_FILTERS[..]),
    ] {
        lacking.extend(missing(required, &listing(path, flag).await?));
    }
    lacking.dedup();
    if lacking.is_empty() {
        Ok(version)
    } else {
        Err(ToolProblem::Missing {
            tool: EngineTool::Ffmpeg.label().to_owned(),
            missing: lacking.join(", "),
        })
    }
}

fn silent_wav() -> Vec<u8> {
    const RATE: u32 = 8000;
    const SAMPLES: u32 = 800;
    let data = SAMPLES * 2;
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&RATE.to_le_bytes());
    wav.extend_from_slice(&(RATE * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data.to_le_bytes());
    wav.resize(wav.len() + usize::try_from(data).unwrap_or_default(), 0);
    wav
}

pub async fn check_ffprobe(path: &Path, temp_dir: &Path) -> Result<String, ToolProblem> {
    let version = ffmpeg_version(&tool_output(path, &["-version"]).await?, "ffprobe")
        .ok_or_else(|| unrecognized(path, EngineTool::Ffprobe))?;
    let sample = tempfile::Builder::new()
        .prefix("opsis-probe-")
        .suffix(".wav")
        .tempfile_in(temp_dir)
        .map_err(|e| ToolProblem::Failed(e.to_string()))?;
    std::fs::write(sample.path(), silent_wav()).map_err(|e| ToolProblem::Failed(e.to_string()))?;
    let sample_path = sample.path().to_string_lossy().into_owned();
    let probed = tool_output(
        path,
        &[
            "-v",
            "error",
            "-show_entries",
            "format=format_name",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            &sample_path,
        ],
    )
    .await?;
    if probed.trim() == "wav" {
        Ok(version)
    } else {
        Err(unrecognized(path, EngineTool::Ffprobe))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn strings(value: &Value, out: &mut Vec<String>) {
        match value {
            Value::String(s) => out.push(s.clone()),
            Value::Array(items) => items.iter().for_each(|v| strings(v, out)),
            Value::Object(map) => map.values().for_each(|v| strings(v, out)),
            _ => {}
        }
    }

    fn split_top(text: &str, separator: char) -> Vec<String> {
        let (mut parts, mut current, mut depth, mut quoted) =
            (Vec::new(), String::new(), 0i32, false);
        for c in text.chars() {
            match c {
                '\'' => quoted = !quoted,
                '(' | '[' if !quoted => depth += 1,
                ')' | ']' if !quoted => depth -= 1,
                _ => {}
            }
            if c == separator && depth == 0 && !quoted {
                parts.push(std::mem::take(&mut current));
            } else {
                current.push(c);
            }
        }
        parts.push(current);
        parts
    }

    fn filter_name(part: &str) -> Option<String> {
        let mut rest = part.trim();
        while let Some(after) = rest.strip_prefix('[') {
            rest = after.split_once(']')?.1.trim_start();
        }
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')
            .collect();
        let next = rest[name.len()..].chars().next();
        let ends = matches!(next, None | Some('=' | '['));
        (!name.is_empty() && name.starts_with(|c: char| c.is_ascii_lowercase()) && ends)
            .then_some(name)
    }

    fn looks_like_graph(text: &str) -> bool {
        text.contains("[0:")
            || text.contains("[out")
            || text.contains("]")
            || text
                .split(',')
                .next()
                .is_some_and(|f| filter_name(f).is_some() && f.contains('='))
    }

    fn graph_filters() -> BTreeSet<String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
        let mut found = BTreeSet::new();
        for folder in ["media", "render"] {
            for entry in std::fs::read_dir(root.join(folder)).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().is_none_or(|e| e != "json") {
                    continue;
                }
                let value: Value =
                    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
                let mut texts = Vec::new();
                strings(&value, &mut texts);
                for text in texts.iter().filter(|t| looks_like_graph(t)) {
                    for chain in split_top(text, ';') {
                        found.extend(split_top(&chain, ',').iter().filter_map(|p| filter_name(p)));
                    }
                }
            }
        }
        found
    }

    #[test]
    fn requires_every_filter_the_golden_graphs_use() {
        let required: BTreeSet<&str> = REQUIRED_FILTERS.into_iter().collect();
        let used = graph_filters();
        assert!(used.len() > 40, "only found {used:?}");
        let unknown: Vec<&String> = used
            .iter()
            .filter(|name| !required.contains(name.as_str()))
            .collect();
        assert!(
            unknown.is_empty(),
            "add these to REQUIRED_FILTERS: {unknown:?}"
        );
    }

    #[test]
    fn reads_names_from_ffmpeg_listings() {
        let encoders = " V....D libx264              libx264 H.264\n A....D aac                  AAC\n V..... = Video\n";
        let names = listed(encoders);
        assert!(names.contains("libx264") && names.contains("aac"));
        let formats = " D   concat          Virtual concatenation script\n DE  mov,mp4,m4a  QuickTime\n  E  null            raw null video\n";
        let names = listed(formats);
        assert!(
            ["concat", "mov", "mp4", "null"]
                .iter()
                .all(|n| names.contains(*n))
        );
        let filters =
            " TSC aap               AA->A      Apply\n ... abench            A->A       Bench\n";
        assert!(listed(filters).contains("abench"));
        assert_eq!(missing(&["aac", "gif"], &names), ["aac", "gif"]);
    }

    #[test]
    fn writes_a_probeable_wav() {
        let wav = silent_wav();
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(wav.len(), 44 + 1600);
    }

    #[tokio::test]
    #[ignore = "needs the dev engine in ~/.config/Opsis/engine"]
    async fn accepts_the_pinned_video_tools() {
        let dir = dirs_engine();
        assert!(
            check_ffmpeg(&dir.join("ffmpeg").join(exe("ffmpeg")))
                .await
                .is_ok()
        );
        let temp = tempfile::tempdir().unwrap();
        assert!(
            check_ffprobe(&dir.join("ffmpeg").join(exe("ffprobe")), temp.path())
                .await
                .is_ok()
        );
    }

    fn exe(name: &str) -> String {
        format!("{name}{}", std::env::consts::EXE_SUFFIX)
    }

    fn dirs_engine() -> std::path::PathBuf {
        std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".config/Opsis/engine")
    }
}
