use opsis_types::actions::ActionKind;
use opsis_types::timeline::TimelineSpec;
use serde::Serialize;

use crate::time::{chapter_time, frames_to_ms};
use crate::timeline::flatten::flatten_spec;

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FilmChapter {
    pub title: String,
    pub start_ms: f64,
    pub end_ms: f64,
}

const MIN_CHAPTER_MS: f64 = 10_000.0;
pub const MIN_LISTED_CHAPTERS: usize = 3;
const OPENING_TITLE: &str = "Intro";

pub fn film_chapters(spec: &TimelineSpec) -> Vec<FilmChapter> {
    let duration_ms = frames_to_ms(spec.duration_in_frames, spec.fps);
    let mut starts: Vec<(String, f64)> = flatten_spec(spec)
        .marks
        .into_iter()
        .filter(|m| m.kind == ActionKind::Chapter)
        .map(|m| (m.label, frames_to_ms(m.frame, spec.fps)))
        .filter(|(_, start)| *start < duration_ms)
        .collect();
    starts.sort_by(|a, b| a.1.total_cmp(&b.1));
    let Some(first) = starts.first_mut() else {
        return Vec::new();
    };
    if first.1 >= MIN_CHAPTER_MS {
        starts.insert(0, (OPENING_TITLE.to_owned(), 0.0));
    } else {
        first.1 = 0.0;
    }
    let mut kept: Vec<(String, f64)> = Vec::new();
    for c in starts {
        if kept
            .last()
            .is_some_and(|previous| c.1 - previous.1 < MIN_CHAPTER_MS)
        {
            continue;
        }
        kept.push(c);
    }
    let last_long_enough = kept
        .iter()
        .enumerate()
        .rposition(|(i, c)| i == 0 || duration_ms - c.1 >= MIN_CHAPTER_MS)
        .unwrap_or(0);
    kept.truncate(last_long_enough + 1);
    let ends: Vec<f64> = kept
        .iter()
        .skip(1)
        .map(|c| c.1)
        .chain([duration_ms])
        .collect();
    kept.into_iter()
        .zip(ends)
        .map(|((title, start_ms), end_ms)| FilmChapter {
            title,
            start_ms,
            end_ms,
        })
        .collect()
}

pub fn chapter_list(chapters: &[FilmChapter]) -> String {
    let lines: Vec<String> = chapters
        .iter()
        .map(|c| format!("{} {}", chapter_time(c.start_ms), c.title))
        .collect();
    format!("{}\n", lines.join("\n"))
}

fn escape_meta(value: &str) -> String {
    value.chars().fold(String::new(), |mut out, c| {
        if matches!(c, '=' | ';' | '#' | '\\' | '\n') {
            out.push('\\');
        }
        out.push(c);
        out
    })
}

pub fn chapter_metadata(chapters: &[FilmChapter]) -> String {
    let mut lines = vec![";FFMETADATA1".to_owned()];
    for c in chapters {
        lines.push("[CHAPTER]".to_owned());
        lines.push("TIMEBASE=1/1000".to_owned());
        lines.push(format!("START={}", c.start_ms));
        lines.push(format!("END={}", c.end_ms));
        lines.push(format!("title={}", escape_meta(&c.title)));
    }
    lines.push(String::new());
    lines.join("\n")
}
