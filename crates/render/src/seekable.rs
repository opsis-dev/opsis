use std::path::{Path, PathBuf};
use std::sync::Mutex;

use opsis_engine::files::{atomic_write, file_identity, write_via_temp};
use opsis_engine::hash::versioned_hash;
use opsis_engine::versions::PREPARED_VERSION;
use opsis_media::ffmpeg::{Ffmpeg, RunOptions};
use opsis_probe::Probe;
use opsis_runner::context::{ExecError, ExecutorContext};
use opsis_types::numbers;

pub const MAX_KEYFRAME_GAP_US: i64 = 1_100_000;
const KEY_LENGTH: usize = 32;
const PREPARED_FOLDER: &str = ".opsis/cache/prepared";
const US_PER_SECOND: f64 = 1_000_000.0;
const US_PER_MS: i64 = 1000;
const FALLBACK_FPS: f64 = 30.0;
const KEPT_FORMATS: [&str; 6] = [
    "yuv420p", "yuvj420p", "yuv422p", "yuvj422p", "yuv444p", "yuvj444p",
];
const WIDE_FORMAT: &str = "yuv444p";
const PREPARING: &str = "Preparing footage";

pub fn longest_keyframe_gap_us(keyframes_us: &[i64], end_us: i64) -> i64 {
    let mut times = keyframes_us.to_vec();
    times.sort_unstable();
    let Some(&last) = times.last() else {
        return i64::MAX;
    };
    times
        .into_iter()
        .chain(std::iter::once(end_us.max(last)))
        .scan(0, |previous, time| {
            let gap = time - *previous;
            *previous = time;
            Some(gap)
        })
        .max()
        .unwrap_or(i64::MAX)
}

pub fn keyframe_us(line: &str) -> Option<i64> {
    let (_, rest) = line.split_once("pts_time:")?;
    let seconds: f64 = rest.split_whitespace().next()?.parse().ok()?;
    Some(numbers::truncate((seconds * US_PER_SECOND).round()))
}

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|&a| a.to_owned()).collect()
}

fn read_error(file: &Path, error: impl std::fmt::Display) -> ExecError {
    ExecError::new(format!("Couldn't read {}: {error}", file.display()))
}

async fn keyframes(
    tools: &Ffmpeg,
    ctx: &ExecutorContext,
    file: &Path,
) -> Result<Vec<i64>, ExecError> {
    let found = Mutex::new(Vec::new());
    let log = |line: &str| {
        if let Some(time) = keyframe_us(line)
            && let Ok(mut found) = found.lock()
        {
            found.push(time);
        }
    };
    let mut args = strings(&["-nostdin", "-skip_frame", "nokey", "-i"]);
    args.push(file.to_string_lossy().into_owned());
    args.extend(strings(&[
        "-map", "0:v:0", "-an", "-vf", "showinfo", "-f", "null", "-",
    ]));
    let opts = RunOptions {
        cancel: Some(&ctx.cancel),
        on_log: Some(&log),
        ..RunOptions::default()
    };
    tools
        .run(&args, opts)
        .await
        .map_err(|e| read_error(file, e))?;
    Ok(found.into_inner().unwrap_or_default())
}

fn copy_args(file: &Path, probe: &Probe, output: &Path) -> Vec<String> {
    let video = probe.video.as_ref();
    let every = numbers::whole(video.and_then(|v| v.fps).unwrap_or(FALLBACK_FPS)).max(1);
    let format = video
        .map(|v| v.pix_fmt.as_str())
        .filter(|f| KEPT_FORMATS.contains(f))
        .unwrap_or(WIDE_FORMAT);
    let mut args = strings(&["-nostdin", "-i"]);
    args.push(file.to_string_lossy().into_owned());
    args.extend(strings(&[
        "-map",
        "0:v:0",
        "-an",
        "-c:v",
        "libx264",
        "-preset",
        "veryfast",
        "-qp",
        "0",
        "-g",
        &every.to_string(),
        "-fps_mode",
        "passthrough",
        "-pix_fmt",
        format,
    ]));
    args.push(output.to_string_lossy().into_owned());
    args
}

pub async fn seekable(
    ctx: &ExecutorContext,
    file: &Path,
    probe: &Probe,
) -> Result<PathBuf, ExecError> {
    let identity = file_identity(file).map_err(|e| read_error(file, e))?;
    let key = versioned_hash(
        PREPARED_VERSION,
        &("seekable", file.to_string_lossy(), identity),
        KEY_LENGTH,
    );
    let folder = ctx.project_dir.join(PREPARED_FOLDER);
    let marker = folder.join(format!("{key}.seekable"));
    let copy = folder.join(format!("{key}.mp4"));
    if marker.exists() {
        return Ok(file.to_owned());
    }
    if copy.exists() {
        return Ok(copy);
    }
    ctx.progress(0.0, Some(PREPARING));
    let tools = Ffmpeg::new(&ctx.env.engine).with_temp(&ctx.env.temp_dir);
    let found = keyframes(&tools, ctx, file).await?;
    let end_us = probe.duration_ms.map_or(0, |ms| i64::from(ms) * US_PER_MS);
    std::fs::create_dir_all(&folder)?;
    if longest_keyframe_gap_us(&found, end_us) <= MAX_KEYFRAME_GAP_US {
        if let Err(error) = atomic_write(&marker, b"") {
            log::warn!("keeping a footage check: {error}");
        }
        return Ok(file.to_owned());
    }
    let opts = RunOptions {
        cancel: Some(&ctx.cancel),
        ..RunOptions::default()
    };
    write_via_temp(&copy, |partial| async move {
        tools
            .run(&copy_args(file, probe, &partial), opts)
            .await
            .map_err(|e| read_error(file, e))
    })
    .await?;
    Ok(copy)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn keyframes_every_step_never_leave_a_longer_gap(step in 1i64..5_000_000, count in 1usize..200, tail in 0i64..5_000_000) {
            let keys: Vec<i64> = (0..count).map(|i| i64::try_from(i).unwrap() * step).collect();
            let end = keys.last().copied().unwrap() + tail.min(step);
            prop_assert!(longest_keyframe_gap_us(&keys, end) <= step);
        }

        #[test]
        fn another_keyframe_never_lengthens_the_longest_gap(mut keys in proptest::collection::vec(0i64..60_000_000, 1..50), extra in 0i64..60_000_000) {
            let end = 60_000_000;
            let before = longest_keyframe_gap_us(&keys, end);
            keys.push(extra);
            prop_assert!(longest_keyframe_gap_us(&keys, end) <= before);
        }

        #[test]
        fn a_keyframe_line_reads_back_its_time(micros in 0i64..10_000_000_000) {
            let seconds = numbers::float_wide(u64::try_from(micros).unwrap()) / US_PER_SECOND;
            let line = format!("[Parsed_showinfo_0 @ 0x1] n:   3 pts:  128000 pts_time:{seconds:.6}   duration: 512");
            prop_assert_eq!(keyframe_us(&line), Some(micros));
        }
    }

    #[test]
    fn no_keyframes_is_never_seekable() {
        assert_eq!(longest_keyframe_gap_us(&[], 1_000), i64::MAX);
    }
}
