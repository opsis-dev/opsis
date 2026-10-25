use std::collections::HashSet;

use indexmap::IndexMap;
use opsis_compile::registry::is_output_type;
use opsis_types::events::{CANCELLED, NodeProgress, NodeRunStatus, RunEvent, RunProgress};
use opsis_types::values::{Artifact, NodeResult};

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

pub fn resume_run_progress(state: &mut RunProgress, saved: &RunProgress) {
    let ids: Vec<String> = saved.statuses.keys().cloned().collect();
    forget(state, &ids);
    state.run_id.clone_from(&saved.run_id);
    state.statuses.extend(saved.statuses.clone());
    state.progress.extend(saved.progress.clone());
    state.cached.extend(saved.cached.clone());
    state.errors.extend(saved.errors.clone());
    state.traces.extend(saved.traces.clone());
    state.results.extend(saved.results.clone());
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
    exports.sort_by(|a, b| b.finished_at.total_cmp(&a.finished_at));
    let mut seen = HashSet::new();
    exports
        .into_iter()
        .flat_map(|r| r.artifacts.iter())
        .filter(|a| seen.insert(a.path.clone()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn result(finished_at: f64, paths: &[&str]) -> NodeResult {
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
            result: Some(Box::new(result(1.0, &["a.mp4"]))),
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
                result: Some(Box::new(result(1.0, &[]))),
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
                result: Some(Box::new(result(2.0, &[]))),
            },
        );
        assert_eq!(
            serde_json::to_value(&failed.traces).unwrap(),
            json!({ "b": "t2" })
        );
    }

    #[test]
    fn resumes_over_a_freshly_opened_project() {
        let mut opened = RunProgress::default();
        opened.errors.insert("old".into(), "earlier".into());
        opened.results.insert("x".into(), result(1.0, &[]));
        let saved = run(vec![
            event(json!({ "type": "run:start", "runId": "r", "nodeIds": ["a", "b"], "at": 0 })),
            RunEvent::NodeDone {
                run_id: "r".into(),
                node_id: "a".into(),
                cached: true,
                result: Some(Box::new(result(2.0, &["a.mp4"]))),
            },
            event(json!({ "type": "node:progress", "runId": "r", "nodeId": "b", "progress": 0.5 })),
        ]);
        resume_run_progress(&mut opened, &saved);
        assert_eq!(opened.run_id.as_deref(), Some("r"));
        assert_eq!(statuses(&opened), json!({ "a": "cached", "b": "queued" }));
        assert_eq!(
            opened.progress["b"],
            NodeProgress {
                value: 0.5,
                message: None
            }
        );
        assert_eq!(
            serde_json::to_value(&opened.errors).unwrap(),
            json!({ "old": "earlier" })
        );
        let mut keys: Vec<&String> = opened.results.keys().collect();
        keys.sort();
        assert_eq!(keys, ["a", "x"]);
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
            ("film", result(1.0, &["film.mp4"])),
            ("code", result(5.0, &["code"])),
            ("rec", result(9.0, &["rec.mp4"])),
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
            ("a", result(1.0, &["out/project-og.png"])),
            (
                "b",
                result(4.0, &["out/project-og.png", "out/project-square.png"]),
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
}
