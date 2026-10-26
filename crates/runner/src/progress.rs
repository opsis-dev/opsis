use std::collections::{HashSet, VecDeque};

use indexmap::IndexMap;
use opsis_compile::registry::is_output_type;
use opsis_types::events::{
    CANCELLED, LogLevel, LogLine, NodeProgress, NodeRunStatus, RunEvent, RunProgress, RunView,
};
use opsis_types::values::{Artifact, LastRun, NodeFailure, NodeResult};

fn forget(progress: &mut RunProgress, ids: &[String]) {
    for id in ids {
        progress.progress.shift_remove(id);
        progress.cached.shift_remove(id);
        progress.errors.shift_remove(id);
        progress.traces.shift_remove(id);
    }
}

pub fn apply_run_event(state: &mut RunProgress, event: &RunEvent) {
    match event {
        RunEvent::RunStart {
            run_id, node_ids, ..
        } => {
            state.run_id = Some(run_id.clone());
            for id in node_ids {
                state.statuses.insert(id.clone(), NodeRunStatus::Queued);
            }
            forget(state, node_ids);
        }
        RunEvent::NodeStart { node_id, .. } => {
            state
                .statuses
                .insert(node_id.clone(), NodeRunStatus::Running);
        }
        RunEvent::NodeProgress {
            node_id,
            progress,
            message,
            ..
        } => {
            state.progress.insert(
                node_id.clone(),
                NodeProgress {
                    value: *progress,
                    message: message.clone(),
                },
            );
        }
        RunEvent::NodeDone {
            node_id,
            cached,
            result,
            ..
        } => {
            state.statuses.insert(
                node_id.clone(),
                if *cached {
                    NodeRunStatus::Cached
                } else {
                    NodeRunStatus::Done
                },
            );
            state.cached.insert(node_id.clone(), *cached);
            state.progress.insert(
                node_id.clone(),
                NodeProgress {
                    value: 1.0,
                    message: None,
                },
            );
            state.errors.shift_remove(node_id);
            if let Some(result) = result {
                if !result.outputs.is_empty() || !result.artifacts.is_empty() {
                    state.results.insert(node_id.clone(), (**result).clone());
                }
                if let Some(trace) = &result.trace_id {
                    state.traces.insert(node_id.clone(), trace.clone());
                }
            }
        }
        RunEvent::NodeError {
            node_id,
            message,
            trace_id,
            ..
        } => {
            if message == CANCELLED {
                state.statuses.shift_remove(node_id);
                return;
            }
            state.statuses.insert(node_id.clone(), NodeRunStatus::Error);
            state.errors.insert(node_id.clone(), message.clone());
            if let Some(trace) = trace_id {
                state.traces.insert(node_id.clone(), trace.clone());
            }
        }
        RunEvent::NodeSkipped { node_id, .. } => {
            state
                .statuses
                .insert(node_id.clone(), NodeRunStatus::Skipped);
        }
        RunEvent::NodeLog { .. } | RunEvent::RunDone { .. } => {}
    }
}

pub fn failure_message(failed_names: &[String], skipped: usize) -> String {
    let skipped_text = if skipped > 0 {
        format!(", {skipped} skipped")
    } else {
        String::new()
    };
    match failed_names {
        [] if skipped == 1 => "1 node was skipped".to_owned(),
        [] => format!("{skipped} nodes were skipped"),
        [one] => format!("{one} failed{skipped_text}"),
        many => format!(
            "{} nodes failed: {}{skipped_text}",
            many.len(),
            many.join(", ")
        ),
    }
}

pub fn exported_artifacts<'a>(
    nodes: impl IntoIterator<Item = (&'a str, &'a str)>,
    results: &IndexMap<String, NodeResult>,
) -> Vec<Artifact> {
    let mut exports: Vec<&NodeResult> = nodes
        .into_iter()
        .filter(|(_, kind)| is_output_type(kind))
        .filter_map(|(id, _)| results.get(id))
        .collect();
    exports.sort_by_key(|r| std::cmp::Reverse(r.finished_at));
    let mut seen = HashSet::new();
    exports
        .into_iter()
        .flat_map(|r| r.artifacts.iter())
        .filter(|a| seen.insert(a.path.clone()))
        .cloned()
        .collect()
}

pub const MAX_LOG_LINES: usize = 2000;
const RUN_FAILED: &str = "Failed";

#[derive(Debug, Default)]
pub struct RunLog {
    next: u64,
    lines: VecDeque<LogLine>,
}

impl RunLog {
    pub fn push(
        &mut self,
        at: i64,
        level: LogLevel,
        node_id: Option<String>,
        message: String,
    ) -> LogLine {
        self.next += 1;
        let line = LogLine {
            id: self.next,
            at,
            level,
            node_id,
            message,
        };
        self.lines.push_back(line.clone());
        while self.lines.len() > MAX_LOG_LINES {
            self.lines.pop_front();
        }
        line
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn lines(&self) -> Vec<LogLine> {
        self.lines.iter().cloned().collect()
    }
}

fn newer_failures(last_run: &LastRun) -> impl Iterator<Item = (&String, &NodeFailure)> {
    last_run
        .failures
        .iter()
        .flatten()
        .filter(|(id, f)| f.at > last_run.nodes.get(*id).map_or(0, |r| r.finished_at))
}

pub fn seeded_view(last_run: Option<&LastRun>) -> RunView {
    let mut view = RunView::default();
    let Some(last_run) = last_run else {
        return view;
    };
    let progress = &mut view.progress;
    progress.results = last_run.nodes.clone();
    for (id, result) in &last_run.nodes {
        if let Some(trace) = &result.trace_id {
            progress.traces.insert(id.clone(), trace.clone());
        }
    }
    for (id, failure) in newer_failures(last_run) {
        if let Some(trace) = &failure.trace_id {
            progress.traces.insert(id.clone(), trace.clone());
        }
        progress.errors.insert(id.clone(), failure.message.clone());
    }
    view
}

fn fresh_run(view: &mut RunView, running: bool) {
    view.running = running;
    view.outputs.clear();
    view.last_error = None;
    view.failed_node_ids.clear();
    view.skipped_node_ids.clear();
}

pub fn begin_run(view: &mut RunView, run_id: &str) {
    if view.progress.run_id.as_deref() == Some(run_id) {
        return;
    }
    fresh_run(view, true);
    view.progress.run_id = Some(run_id.to_owned());
}

pub fn apply_to_view(
    view: &mut RunView,
    log: &mut RunLog,
    event: &RunEvent,
    now: i64,
) -> Option<LogLine> {
    match event {
        RunEvent::RunStart { .. } => {
            apply_run_event(&mut view.progress, event);
            fresh_run(view, true);
            None
        }
        RunEvent::NodeLog {
            node_id,
            level,
            message,
            at,
            ..
        } => Some(log.push(*at, *level, node_id.clone(), message.clone())),
        RunEvent::NodeError {
            node_id, message, ..
        } => {
            apply_run_event(&mut view.progress, event);
            Some(log.push(now, LogLevel::Error, Some(node_id.clone()), message.clone()))
        }
        RunEvent::RunDone {
            run_id,
            ok,
            cancelled,
            error,
            failed_node_ids,
            skipped_node_ids,
            outputs,
            ..
        } => {
            view.progress.run_id = Some(run_id.clone());
            view.running = false;
            view.outputs.clone_from(outputs);
            view.failed_node_ids = failed_node_ids.clone().unwrap_or_default();
            view.skipped_node_ids = skipped_node_ids.clone().unwrap_or_default();
            view.last_error = (!ok).then(|| {
                error
                    .clone()
                    .unwrap_or_else(|| if *cancelled { CANCELLED } else { RUN_FAILED }.to_owned())
            });
            None
        }
        _ => {
            apply_run_event(&mut view.progress, event);
            None
        }
    }
}

pub fn forget_errors(view: &mut RunView, ids: &[String]) -> bool {
    let before = view.progress.errors.len();
    view.progress.errors.retain(|id, _| !ids.contains(id));
    view.progress.errors.len() != before
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn result(finished_at: i64, paths: &[&str]) -> NodeResult {
        let artifacts: Vec<_> = paths
            .iter()
            .map(|p| json!({ "kind": "video", "path": p }))
            .collect();
        serde_json::from_value(json!({ "hash": "h", "outputs": {}, "artifacts": artifacts, "finishedAt": finished_at, "durationMs": 0 })).unwrap()
    }

    fn event(value: serde_json::Value) -> RunEvent {
        serde_json::from_value(value).unwrap()
    }

    fn run(events: Vec<RunEvent>) -> RunProgress {
        let mut state = RunProgress::default();
        for e in &events {
            apply_run_event(&mut state, e);
        }
        state
    }

    fn statuses(state: &RunProgress) -> serde_json::Value {
        serde_json::to_value(&state.statuses).unwrap()
    }

    #[test]
    fn follows_a_run_from_start_to_finish() {
        let done = RunEvent::NodeDone {
            run_id: "r".into(),
            node_id: "a".into(),
            cached: true,
            result: Some(Box::new(result(1, &["a.mp4"]))),
        };
        let p = run(vec![
            event(
                json!({ "type": "run:start", "runId": "r", "nodeIds": ["a", "b", "c", "d"], "at": 0 }),
            ),
            done,
            event(json!({ "type": "node:start", "runId": "r", "nodeId": "b", "at": 1 })),
            event(
                json!({ "type": "node:progress", "runId": "r", "nodeId": "b", "progress": 0.4, "message": "Recording" }),
            ),
            event(
                json!({ "type": "node:error", "runId": "r", "nodeId": "c", "message": "boom", "traceId": "t" }),
            ),
            event(
                json!({ "type": "node:skipped", "runId": "r", "nodeId": "d", "reason": "Something before it failed" }),
            ),
        ]);
        assert_eq!(p.run_id.as_deref(), Some("r"));
        assert_eq!(
            statuses(&p),
            json!({ "a": "cached", "b": "running", "c": "error", "d": "skipped" })
        );
        assert_eq!(
            serde_json::to_value(&p.progress).unwrap(),
            json!({ "a": { "value": 1.0 }, "b": { "value": 0.4, "message": "Recording" } })
        );
        assert_eq!(
            serde_json::to_value(&p.errors).unwrap(),
            json!({ "c": "boom" })
        );
        assert_eq!(
            serde_json::to_value(&p.traces).unwrap(),
            json!({ "c": "t" })
        );
        assert_eq!(p.results.keys().collect::<Vec<_>>(), ["a"]);
    }

    #[test]
    fn clears_only_the_nodes_a_new_run_includes_and_forgets_their_old_traces() {
        let mut state = run(vec![
            event(json!({ "type": "run:start", "runId": "r1", "nodeIds": ["a", "b"], "at": 0 })),
            RunEvent::NodeDone {
                run_id: "r1".into(),
                node_id: "a".into(),
                cached: false,
                result: Some(Box::new(result(1, &[]))),
            },
            event(json!({ "type": "node:error", "runId": "r1", "nodeId": "b", "message": "boom" })),
        ]);
        apply_run_event(
            &mut state,
            &event(json!({ "type": "run:start", "runId": "r2", "nodeIds": ["a"], "at": 2 })),
        );
        assert_eq!(statuses(&state), json!({ "a": "queued", "b": "error" }));
        assert_eq!(
            serde_json::to_value(&state.errors).unwrap(),
            json!({ "b": "boom" })
        );
        assert!(state.progress.is_empty());
        let mut failed = run(vec![
            event(json!({ "type": "run:start", "runId": "r1", "nodeIds": ["a", "b"], "at": 0 })),
            event(
                json!({ "type": "node:error", "runId": "r1", "nodeId": "a", "message": "boom", "traceId": "t1" }),
            ),
            event(
                json!({ "type": "node:error", "runId": "r1", "nodeId": "b", "message": "boom", "traceId": "t2" }),
            ),
        ]);
        apply_run_event(
            &mut failed,
            &event(json!({ "type": "run:start", "runId": "r2", "nodeIds": ["a"], "at": 1 })),
        );
        apply_run_event(
            &mut failed,
            &RunEvent::NodeDone {
                run_id: "r2".into(),
                node_id: "a".into(),
                cached: false,
                result: Some(Box::new(result(2, &[]))),
            },
        );
        assert_eq!(
            serde_json::to_value(&failed.traces).unwrap(),
            json!({ "b": "t2" })
        );
    }

    #[test]
    fn names_the_failed_nodes_and_counts_skips_apart() {
        assert_eq!(
            failure_message(&["Export film".into()], 0),
            "Export film failed"
        );
        assert_eq!(
            failure_message(&["Export film".into(), "GIF".into()], 3),
            "2 nodes failed: Export film, GIF, 3 skipped"
        );
        assert_eq!(failure_message(&[], 1), "1 node was skipped");
    }

    #[test]
    fn lists_every_exports_files_newest_first_and_once() {
        let results: IndexMap<String, NodeResult> = [
            ("film", result(1, &["film.mp4"])),
            ("code", result(5, &["code"])),
            ("rec", result(9, &["rec.mp4"])),
        ]
        .into_iter()
        .map(|(k, r)| (k.to_owned(), r))
        .collect();
        let files = exported_artifacts(
            [
                ("film", "output.render"),
                ("code", "output.exportProject"),
                ("rec", "capture.record"),
            ],
            &results,
        );
        assert_eq!(
            files.iter().map(|a| a.path.as_str()).collect::<Vec<_>>(),
            ["code", "film.mp4"]
        );
        let results: IndexMap<String, NodeResult> = [
            ("a", result(1, &["out/project-og.png"])),
            (
                "b",
                result(4, &["out/project-og.png", "out/project-square.png"]),
            ),
        ]
        .into_iter()
        .map(|(k, r)| (k.to_owned(), r))
        .collect();
        let files = exported_artifacts([("a", "output.still"), ("b", "output.still")], &results);
        assert_eq!(
            files.iter().map(|a| a.path.as_str()).collect::<Vec<_>>(),
            ["out/project-og.png", "out/project-square.png"]
        );
    }

    #[test]
    fn forgets_an_earlier_failure_when_the_node_finishes_and_a_stopped_node_keeps_no_status() {
        let mut state = RunProgress {
            run_id: Some("r1".into()),
            ..RunProgress::default()
        };
        apply_run_event(
            &mut state,
            &event(
                json!({ "type": "node:error", "runId": "r1", "nodeId": "wait", "message": "The browser closed before the action finished." }),
            ),
        );
        apply_run_event(
            &mut state,
            &event(
                json!({ "type": "node:done", "runId": "r2", "nodeId": "wait", "cached": false }),
            ),
        );
        assert!(state.errors.is_empty());
        let mut running = RunProgress::default();
        apply_run_event(
            &mut running,
            &event(json!({ "type": "node:start", "runId": "r", "nodeId": "rec", "at": 0 })),
        );
        apply_run_event(
            &mut running,
            &event(
                json!({ "type": "node:error", "runId": "r", "nodeId": "rec", "message": CANCELLED }),
            ),
        );
        assert!(running.statuses.get("rec").is_none() && running.errors.is_empty());
    }

    fn traced(finished_at: i64, trace: &str) -> NodeResult {
        serde_json::from_value(json!({ "hash": "h", "outputs": {}, "artifacts": [], "finishedAt": finished_at, "durationMs": 0, "traceId": trace })).unwrap()
    }

    fn apply_all(view: &mut RunView, log: &mut RunLog, events: Vec<serde_json::Value>) {
        for e in events {
            apply_to_view(view, log, &event(e), 0);
        }
    }

    #[test]
    fn opens_the_newest_trace_of_each_node_failed_or_not() {
        let last: LastRun = serde_json::from_value(json!({
            "updatedAt": 300,
            "nodes": { "rec": traced(100, "ok-old"), "still": traced(250, "ok-new") },
            "failures": {
                "rec": { "message": "Assertion failed", "at": 200, "traceId": "failed-new" },
                "still": { "message": "Timeout", "at": 150, "traceId": "failed-old" },
                "login": { "message": "Closed", "at": 120 },
            },
        }))
        .unwrap();
        let view = seeded_view(Some(&last));
        assert_eq!(
            serde_json::to_value(&view.progress.traces).unwrap(),
            json!({ "rec": "failed-new", "still": "ok-new" })
        );
        assert_eq!(
            serde_json::to_value(&view.progress.errors).unwrap(),
            json!({ "rec": "Assertion failed", "login": "Closed" })
        );
    }

    #[test]
    fn points_the_trace_at_the_failed_take_when_a_node_fails_during_a_run() {
        let last: LastRun =
            serde_json::from_value(json!({ "updatedAt": 1, "nodes": { "rec": traced(1, "ok") } }))
                .unwrap();
        let mut view = seeded_view(Some(&last));
        let mut log = RunLog::default();
        apply_all(
            &mut view,
            &mut log,
            vec![
                json!({ "type": "node:error", "runId": "r", "nodeId": "rec", "message": "boom", "traceId": "failed" }),
            ],
        );
        assert_eq!(view.progress.traces["rec"], "failed");
    }

    #[test]
    fn keeps_an_earlier_failure_when_a_later_run_leaves_that_node_out() {
        let mut view = seeded_view(None);
        let mut log = RunLog::default();
        apply_all(
            &mut view,
            &mut log,
            vec![
                json!({ "type": "run:start", "runId": "r1", "nodeIds": ["a", "b", "c"], "at": 0 }),
                json!({ "type": "node:done", "runId": "r1", "nodeId": "a", "cached": false, "result": traced(1, "x") }),
                json!({ "type": "node:error", "runId": "r1", "nodeId": "b", "message": "boom" }),
                json!({ "type": "node:skipped", "runId": "r1", "nodeId": "c", "reason": "Something before it failed" }),
                json!({ "type": "run:done", "runId": "r1", "ok": false, "cancelled": false, "error": "B failed, 1 skipped", "failedNodeIds": ["b"], "skippedNodeIds": ["c"], "outputs": [], "at": 2 }),
            ],
        );
        assert_eq!(view.failed_node_ids, ["b"]);
        assert_eq!(view.last_error.as_deref(), Some("B failed, 1 skipped"));
        begin_run(&mut view, "r2");
        apply_all(
            &mut view,
            &mut log,
            vec![
                json!({ "type": "run:start", "runId": "r2", "nodeIds": ["c"], "at": 3 }),
                json!({ "type": "node:done", "runId": "r2", "nodeId": "c", "cached": false, "result": traced(4, "y") }),
                json!({ "type": "run:done", "runId": "r2", "ok": true, "cancelled": false, "outputs": [], "at": 5 }),
            ],
        );
        assert_eq!(view.progress.errors["b"], "boom");
        assert_eq!(view.progress.statuses["b"], NodeRunStatus::Error);
        assert_eq!(view.progress.statuses["c"], NodeRunStatus::Done);
        assert!(!view.running);
        assert!(view.failed_node_ids.is_empty());
    }

    #[test]
    fn keeps_a_started_run_running_until_it_is_done() {
        let mut view = seeded_view(None);
        let mut log = RunLog::default();
        begin_run(&mut view, "fast");
        assert!(view.running);
        apply_all(
            &mut view,
            &mut log,
            vec![
                json!({ "type": "run:start", "runId": "fast", "nodeIds": ["a"], "at": 0 }),
                json!({ "type": "run:done", "runId": "fast", "ok": true, "cancelled": false, "outputs": [], "at": 1 }),
            ],
        );
        begin_run(&mut view, "fast");
        assert!(!view.running);
    }

    #[test]
    fn keeps_the_log_at_its_limit_however_lines_arrive() {
        let mut view = seeded_view(None);
        let mut log = RunLog::default();
        for i in 0..MAX_LOG_LINES {
            apply_all(
                &mut view,
                &mut log,
                vec![
                    json!({ "type": "node:log", "runId": "r", "nodeId": "a", "level": "info", "message": format!("log {i}"), "at": i }),
                    json!({ "type": "node:error", "runId": "r", "nodeId": "a", "message": format!("error {i}") }),
                ],
            );
        }
        let lines = log.lines();
        assert_eq!(lines.len(), MAX_LOG_LINES);
        assert_eq!(
            lines.last().map(|l| l.message.as_str()),
            Some(format!("error {}", MAX_LOG_LINES - 1).as_str())
        );
    }

    #[test]
    fn forgets_the_errors_of_edited_nodes() {
        let mut view = seeded_view(None);
        view.progress.errors.insert("a".into(), "boom".into());
        assert!(forget_errors(&mut view, &["a".to_owned()]));
        assert!(!forget_errors(&mut view, &["a".to_owned()]));
    }
}
