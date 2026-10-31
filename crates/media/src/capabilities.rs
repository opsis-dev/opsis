use std::collections::BTreeSet;
use std::path::Path;

use opsis_engine::tools::{ToolProblem, ffmpeg_version, tool_output, unrecognized};

const VIDEO_TOOL: &str = "video tool";

const REQUIRED_CONFIGURATION: [&str; 2] = ["--enable-gpl", "--enable-version3"];

const REFUSED_CONFIGURATION: [&str; 1] = ["--enable-nonfree"];

pub const REQUIRED_ENCODERS: [&str; 13] = [
    "aac",
    "gif",
    "libmp3lame",
    "libopus",
    "libvpx",
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

pub const REQUIRED_FILTERS: [&str; 87] = [
    "acompressor",
    "acrossfade",
    "acrusher",
    "adeclick",
    "adeclip",
    "adelay",
    "aecho",
    "afade",
    "afftdn",
    "afir",
    "ametadata",
    "aformat",
    "alimiter",
    "aloop",
    "amix",
    "amovie",
    "anull",
    "anullsrc",
    "apad",
    "aresample",
    "areverse",
    "asetpts",
    "asetnsamples",
    "asetrate",
    "asplit",
    "astats",
    "atempo",
    "atrim",
    "bandpass",
    "blend",
    "cas",
    "channelsplit",
    "color",
    "colorbalance",
    "colorchannelmixer",
    "colorize",
    "colortemperature",
    "compand",
    "concat",
    "crop",
    "curves",
    "deband",
    "deesser",
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
    "lowpass",
    "lowshelf",
    "lut3d",
    "monochrome",
    "noise",
    "normalize",
    "null",
    "overlay",
    "pad",
    "palettegen",
    "paletteuse",
    "pan",
    "reverse",
    "rubberband",
    "rgbashift",
    "scale",
    "setpts",
    "setsar",
    "showspectrumpic",
    "showwavespic",
    "silencedetect",
    "sidechaincompress",
    "speechnorm",
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

fn configuration_problem(version_text: &str) -> Option<String> {
    let flags: BTreeSet<&str> = version_text
        .lines()
        .find_map(|line| line.trim().strip_prefix("configuration:"))
        .unwrap_or_default()
        .split_whitespace()
        .collect();
    let lacking: Vec<&str> = REQUIRED_CONFIGURATION
        .into_iter()
        .filter(|flag| !flags.contains(flag))
        .collect();
    let refused: Vec<&str> = REFUSED_CONFIGURATION
        .into_iter()
        .filter(|flag| flags.contains(flag))
        .collect();
    match (lacking.is_empty(), refused.is_empty()) {
        (true, true) => None,
        (false, _) => Some(format!("It isn't built with {}.", lacking.join(" "))),
        (true, false) => Some(format!("It is built with {}.", refused.join(" "))),
    }
}

pub async fn check_ffmpeg(path: &Path) -> Result<String, ToolProblem> {
    let version_text = tool_output(path, &["-version"]).await?;
    let version = ffmpeg_version(&version_text).ok_or_else(|| unrecognized(path, VIDEO_TOOL))?;
    if let Some(problem) = configuration_problem(&version_text) {
        return Err(ToolProblem::Failed(problem));
    }
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
            tool: VIDEO_TOOL.to_owned(),
            missing: lacking.join(", "),
        })
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
    fn accepts_only_a_gpl_version_3_build_without_nonfree_parts() {
        let line = |flags: &str| format!("ffmpeg version 7.0.2-static\n  configuration: {flags}\n");
        assert_eq!(
            configuration_problem(&line("--enable-gpl --enable-version3 --enable-libx264")),
            None
        );
        assert!(configuration_problem(&line("--enable-gpl --enable-libx264")).is_some());
        assert!(configuration_problem(&line("--enable-version3")).is_some());
        assert!(
            configuration_problem(&line("--enable-gpl --enable-version3 --enable-nonfree"))
                .is_some()
        );
        assert!(configuration_problem("ffmpeg version 7.0.2-static\n").is_some());
    }

    #[tokio::test]
    async fn the_bundled_video_tool_has_everything_opsis_uses() {
        let checked = check_ffmpeg(&opsis_golden::ffmpeg()).await;
        assert!(checked.is_ok(), "{checked:?}");
    }
}
