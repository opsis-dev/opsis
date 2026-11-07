use std::collections::HashMap;

use opsis_compile::text::{format_duration, format_precise_duration};
use opsis_qt::{
    ReplayActionView, ReplayView, TraceActionView, TraceConsoleView, TraceErrorView,
    TraceFrameView, TraceRequestView, TraceView,
};
use opsis_types::trace::{ConsoleLevel, TraceAction, TraceFrame, TraceModel};
use opsis_types::values::{NodeResult, TakeSource};

use crate::ui::plain;

pub struct Sources<'a> {
    pub node_id: &'a str,
    pub title: String,
    pub capture: bool,
    pub result: Option<&'a NodeResult>,
    pub error: Option<&'a str>,
    pub trace: Option<&'a str>,
    pub tab: &'a str,
    pub resolve: &'a dyn Fn(&str) -> String,
}

pub fn depths(actions: &[TraceAction]) -> Vec<u32> {
    let parents: HashMap<&str, Option<&str>> = actions
        .iter()
        .map(|a| (a.id.as_str(), a.parent_id.as_deref()))
        .collect();
    let mut known: HashMap<&str, u32> = HashMap::new();
    actions
        .iter()
        .map(|action| {
            let mut chain = Vec::new();
            let mut at = Some(action.id.as_str());
            let mut base = 0;
            while let Some(id) = at {
                if let Some(&depth) = known.get(id) {
                    base = depth + 1;
                    break;
                }
                if chain.contains(&id) || !parents.contains_key(id) {
                    break;
                }
                chain.push(id);
                at = parents.get(id).copied().flatten();
            }
            for id in chain.iter().rev() {
                known.insert(id, base);
                base += 1;
            }
            known.get(action.id.as_str()).copied().unwrap_or(0)
        })
        .collect()
}

pub fn frame_at(frames: &[TraceFrame], time: f64) -> Option<usize> {
    if frames.is_empty() {
        return None;
    }
    let after = frames.partition_point(|f| f.time <= time);
    Some(after.saturating_sub(1))
}

pub fn picked(actions: &[TraceAction]) -> Option<usize> {
    actions
        .iter()
        .position(|a| a.error.is_some())
        .or_else(|| actions.iter().rposition(|a| a.group))
        .or_else(|| actions.len().checked_sub(1))
}

fn index(found: Option<usize>) -> i32 {
    found.map_or(-1, |i| i32::try_from(i).unwrap_or(i32::MAX))
}

fn at_text(start: f64, time: f64) -> String {
    format!("+{}", format_precise_duration(time - start))
}

const fn level_word(level: ConsoleLevel) -> &'static str {
    match level {
        ConsoleLevel::Log => "log",
        ConsoleLevel::Info => "info",
        ConsoleLevel::Warning => "warning",
        ConsoleLevel::Error => "error",
        ConsoleLevel::Debug => "debug",
    }
}

pub const fn hidden() -> ReplayView {
    ReplayView {
        shown: false,
        node_id: String::new(),
        title: String::new(),
        description: String::new(),
        tab: String::new(),
        alert: String::new(),
        video_file: String::new(),
        image_file: String::new(),
        reveal: String::new(),
        trace_id: String::new(),
        can_replay: false,
        actions: Vec::new(),
    }
}

pub fn view(src: &Sources<'_>) -> ReplayView {
    let recording = src
        .result
        .and_then(|r| r.outputs.get("recording"))
        .and_then(|v| v.as_recording());
    let image = src
        .result
        .and_then(|r| r.outputs.get("image"))
        .and_then(|v| match v {
            opsis_types::values::CompileValue::Image(image) => image.src.clone(),
            _ => None,
        });
    let (shown, file) = recording.map_or((None, None), |r| match &r.source {
        TakeSource::Terminal(t) => (t.preview.clone(), t.cast.clone()),
        TakeSource::Browser(b) => (b.video.clone(), b.video.clone()),
    });
    let trace = src
        .trace
        .map(str::to_owned)
        .or_else(|| src.result.and_then(|r| r.trace_id.clone()));
    let alert = src.error.map_or_else(String::new, |error| {
        let mut text = error.lines().next().unwrap_or_default().to_owned();
        if file.is_some() {
            text.push_str(" The footage below is from the last successful take.");
        }
        if trace.is_some() {
            text.push_str(" Open the Trace tab to see where it stopped.");
        }
        text
    });
    let actions: Vec<ReplayActionView> =
        recording
            .and_then(|r| r.log.as_ref())
            .map_or_default(|log| {
                log.actions
                    .iter()
                    .map(|a| ReplayActionView {
                        at: a.t,
                        time: format_duration(a.t),
                        kind: plain(&a.kind),
                        label: a
                            .label
                            .clone()
                            .or_else(|| a.text.clone())
                            .unwrap_or_default(),
                        selector: a.selector_used.clone().unwrap_or_default(),
                        node_id: a.node_id.clone().unwrap_or_default(),
                    })
                    .collect()
            });
    let description = recording.map_or_else(
        || "Last run of this node".to_owned(),
        |r| {
            let (width, height) = r
                .log
                .as_ref()
                .map_or((0, 0), |l| (l.viewport.width, l.viewport.height));
            format!(
                "{} · {} actions · {width}×{height}",
                format_duration(r.duration_ms),
                actions.len()
            )
        },
    );
    ReplayView {
        shown: true,
        node_id: src.node_id.to_owned(),
        title: src.title.clone(),
        description,
        tab: src.tab.to_owned(),
        alert,
        video_file: shown.as_deref().map_or_default(src.resolve),
        image_file: image.as_deref().map_or_default(src.resolve),
        reveal: file.unwrap_or_default(),
        trace_id: trace.unwrap_or_default(),
        can_replay: src.capture || recording.is_some(),
        actions,
    }
}

pub const fn failed_trace(error: String) -> TraceView {
    TraceView {
        error,
        label: String::new(),
        width: 0,
        height: 0,
        picked: -1,
        warnings: 0,
        actions: Vec::new(),
        frames: Vec::new(),
        console: Vec::new(),
        network: Vec::new(),
        errors: Vec::new(),
    }
}

pub fn trace_view(trace: &TraceModel, resolve: &dyn Fn(&str) -> String) -> TraceView {
    let depth = depths(&trace.actions);
    let first = trace.frames.first();
    let (width, height) = trace.viewport.as_ref().map_or_else(
        || first.map_or((0, 0), |f| (f.width, f.height)),
        |v| (v.width, v.height),
    );
    let ids: HashMap<&str, usize> = trace
        .actions
        .iter()
        .enumerate()
        .map(|(i, a)| (a.id.as_str(), i))
        .collect();
    TraceView {
        error: String::new(),
        label: trace.header.label.clone(),
        width,
        height,
        picked: index(picked(&trace.actions)),
        warnings: opsis_types::numbers::count(
            trace
                .console
                .iter()
                .filter(|c| matches!(c.level, ConsoleLevel::Error | ConsoleLevel::Warning))
                .count(),
        ),
        actions: trace
            .actions
            .iter()
            .zip(depth)
            .map(|(a, depth)| TraceActionView {
                id: a.id.clone(),
                depth,
                group: a.group,
                failed: a.error.is_some(),
                title: a.title.clone(),
                target: a.target.clone().unwrap_or_default(),
                node_id: a.node_id.clone().unwrap_or_default(),
                start: a.start,
                end: a.end,
                at: at_text(trace.start, a.start),
                length: format_precise_duration(a.end - a.start),
                error: a.error.clone().unwrap_or_default(),
                logs: a.logs.clone(),
                has_point: a.point.is_some(),
                point_x: a.point.as_ref().map_or(0.0, |p| p.x),
                point_y: a.point.as_ref().map_or(0.0, |p| p.y),
                has_box: a.bounds.is_some(),
                box_x: a.bounds.as_ref().map_or(0.0, |b| b.x),
                box_y: a.bounds.as_ref().map_or(0.0, |b| b.y),
                box_width: a.bounds.as_ref().map_or(0.0, |b| b.width),
                box_height: a.bounds.as_ref().map_or(0.0, |b| b.height),
                before: index(frame_at(&trace.frames, a.start)),
                after: index(frame_at(&trace.frames, a.end)),
            })
            .collect(),
        frames: trace
            .frames
            .iter()
            .map(|f| TraceFrameView {
                file: resolve(&f.src),
                width: f.width,
                height: f.height,
            })
            .collect(),
        console: trace
            .console
            .iter()
            .map(|c| TraceConsoleView {
                time: c.time,
                at: at_text(trace.start, c.time),
                level: level_word(c.level).to_owned(),
                text: c.text.clone(),
            })
            .collect(),
        network: trace
            .network
            .iter()
            .map(|r| TraceRequestView {
                start: r.start,
                at: at_text(trace.start, r.start),
                method: r.method.clone(),
                status: r
                    .failure
                    .clone()
                    .unwrap_or_else(|| r.status.map_or_default(|s| s.to_string())),
                failed: r.failure.is_some() || r.status.is_some_and(|s| s >= 400),
                url: r.url.clone(),
                kind: if r.resource_type.is_empty() {
                    r.mime_type.split(';').next().unwrap_or_default().to_owned()
                } else {
                    r.resource_type.clone()
                },
                size: r.size.map_or_default(|size| {
                    opsis_types::numbers::byte_size(opsis_types::numbers::truncate::<u64>(
                        size.max(0.0),
                    ))
                }),
                length: format_precise_duration(r.duration),
            })
            .collect(),
        errors: trace
            .errors
            .iter()
            .map(|e| TraceErrorView {
                at: at_text(trace.start, e.time),
                text: e.stack.clone().unwrap_or_else(|| e.message.clone()),
                action: index(e.action_id.as_deref().and_then(|id| ids.get(id).copied())),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use opsis_types::trace::{TraceAction, TraceFrame};
    use proptest::prelude::*;

    use super::{depths, frame_at};

    fn action(id: usize, parent: Option<usize>) -> TraceAction {
        TraceAction {
            id: format!("a{id}"),
            parent_id: parent.map(|p| format!("a{p}")),
            group: false,
            node_id: None,
            title: String::new(),
            target: None,
            start: 0.0,
            end: 0.0,
            error: None,
            logs: Vec::new(),
            point: None,
            bounds: None,
        }
    }

    proptest! {
        #[test]
        fn a_child_sits_one_deeper_than_its_parent(parents in proptest::collection::vec(proptest::option::of(0usize..40), 1..40)) {
            let actions: Vec<TraceAction> = parents
                .iter()
                .enumerate()
                .map(|(i, p)| action(i, p.filter(|p| *p < i)))
                .collect();
            let depth = depths(&actions);
            for (i, a) in actions.iter().enumerate() {
                let parent = a.parent_id.as_deref().and_then(|p| actions.iter().position(|b| b.id == p));
                let expected = parent.map_or(0, |parent| depth[parent] + 1);
                prop_assert_eq!(depth[i], expected);
            }
        }

        #[test]
        fn a_moment_shows_the_last_frame_taken_by_then(mut times in proptest::collection::vec(0f64..1000.0, 1..30), at in -10f64..1100.0) {
            times.sort_by(f64::total_cmp);
            let frames: Vec<TraceFrame> = times
                .iter()
                .map(|&time| TraceFrame { src: String::new(), time, width: 1, height: 1 })
                .collect();
            let shown = frame_at(&frames, at).unwrap();
            if at >= times[0] {
                prop_assert!(times[shown] <= at);
                prop_assert!(shown + 1 == times.len() || times[shown + 1] > at);
            } else {
                prop_assert_eq!(shown, 0);
            }
        }
    }
}
