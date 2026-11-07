use std::collections::HashSet;

use indexmap::IndexMap;
use opsis_compile::catalog::EditorNode;
use opsis_qt::{ArtifactView, Choice, RunLogView, RunPanelView, RunRowView};
use opsis_types::events::{LogLevel, NodeRunStatus, RunState};
use opsis_types::values::{Artifact, ArtifactKind};
use opsis_types::workflow::Workflow;

const CANCELLED: &str = "Cancelled";

pub struct Sources<'a> {
    pub workflow: &'a Workflow,
    pub catalog: &'a IndexMap<String, EditorNode>,
    pub outdated: &'a [String],
    pub selected: Option<&'a str>,
    pub resolve: &'a dyn Fn(&str) -> String,
}

fn name_of(src: &Sources<'_>, id: &str) -> String {
    src.workflow.nodes.iter().find(|n| n.id == id).map_or_else(
        || "A deleted node".to_owned(),
        |node| {
            node.label
                .clone()
                .filter(|l| !l.is_empty())
                .or_else(|| src.catalog.get(&node.kind).map(|d| d.title.clone()))
                .unwrap_or_default()
        },
    )
}

const fn status_word(status: NodeRunStatus) -> &'static str {
    match status {
        NodeRunStatus::Idle => "idle",
        NodeRunStatus::Queued => "queued",
        NodeRunStatus::Running => "running",
        NodeRunStatus::Cached => "cached",
        NodeRunStatus::Done => "done",
        NodeRunStatus::Error => "error",
        NodeRunStatus::Skipped => "skipped",
    }
}

pub fn row_state(
    status: Option<NodeRunStatus>,
    failed_before: bool,
    changed: bool,
) -> &'static str {
    let shown = status.map_or(
        if failed_before {
            "failedBefore"
        } else {
            "fromLastRun"
        },
        status_word,
    );
    if changed && matches!(shown, "done" | "cached" | "fromLastRun") {
        "changed"
    } else {
        shown
    }
}

const fn kind_word(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::Video => "video",
        ArtifactKind::Image => "image",
        ArtifactKind::Audio => "audio",
        ArtifactKind::Json => "json",
        ArtifactKind::File => "file",
    }
}

pub fn artifact(resolve: &dyn Fn(&str) -> String, artifact: &Artifact) -> ArtifactView {
    ArtifactView {
        kind: kind_word(artifact.kind).to_owned(),
        path: artifact.path.clone(),
        file: resolve(&artifact.path),
        name: artifact
            .label
            .clone()
            .unwrap_or_else(|| artifact.path.clone()),
    }
}

const fn level_word(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Debug => "debug",
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
    }
}

pub const fn empty() -> RunPanelView {
    RunPanelView {
        running: false,
        title: String::new(),
        failure: String::new(),
        failed: Vec::new(),
        skipped: 0,
        rows: Vec::new(),
        logs: Vec::new(),
        filtered: false,
        filter_id: String::new(),
        filter_type: String::new(),
        artifacts: Vec::new(),
    }
}

pub fn view(state: &RunState, src: &Sources<'_>) -> RunPanelView {
    let run = &state.view;
    let progress = &run.progress;
    let outdated: HashSet<&str> = src.outdated.iter().map(String::as_str).collect();
    let rows = src
        .workflow
        .nodes
        .iter()
        .filter(|n| {
            progress.statuses.contains_key(&n.id)
                || progress.errors.contains_key(&n.id)
                || progress
                    .results
                    .get(&n.id)
                    .is_some_and(|r| !r.artifacts.is_empty())
        })
        .map(|n| {
            let status = progress.statuses.get(&n.id).copied();
            RunRowView {
                id: n.id.clone(),
                name: name_of(src, &n.id),
                state: row_state(
                    status,
                    progress.errors.contains_key(&n.id),
                    outdated.contains(n.id.as_str()),
                )
                .to_owned(),
                progress: if status == Some(NodeRunStatus::Running) {
                    progress.progress.get(&n.id).map_or(-1.0, |p| p.value)
                } else {
                    -1.0
                },
                selected: src.selected == Some(n.id.as_str()),
            }
        })
        .collect();
    let filter = src
        .selected
        .and_then(|id| src.workflow.nodes.iter().find(|n| n.id == id));
    let error = run.last_error.clone().unwrap_or_default();
    RunPanelView {
        running: run.running,
        title: if run.running {
            "Running…".to_owned()
        } else if error == CANCELLED {
            "Last run stopped".to_owned()
        } else if error.is_empty() {
            "Nodes".to_owned()
        } else {
            "Last run failed".to_owned()
        },
        failure: if run.running { String::new() } else { error },
        failed: run
            .failed_node_ids
            .iter()
            .map(|id| Choice {
                value: id.clone(),
                label: name_of(src, id),
            })
            .collect(),
        skipped: u32::try_from(run.skipped_node_ids.len()).unwrap_or(u32::MAX),
        rows,
        logs: state
            .logs
            .iter()
            .filter(|l| {
                src.selected
                    .is_none_or(|id| l.node_id.as_deref() == Some(id))
            })
            .map(|l| RunLogView {
                id: l.id,
                at: l.at,
                level: level_word(l.level).to_owned(),
                message: l.message.clone(),
            })
            .collect(),
        filtered: src.selected.is_some(),
        filter_id: filter.map_or_default(|n| n.id.clone()),
        filter_type: filter.map_or_default(|n| n.kind.clone()),
        artifacts: filter
            .map_or_else(
                || run.exports.clone(),
                |n| {
                    progress
                        .results
                        .get(&n.id)
                        .map_or_default(|r| r.artifacts.clone())
                },
            )
            .iter()
            .map(|a| artifact(src.resolve, a))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn status() -> impl Strategy<Value = Option<NodeRunStatus>> {
        proptest::option::of(proptest::sample::select(vec![
            NodeRunStatus::Idle,
            NodeRunStatus::Queued,
            NodeRunStatus::Running,
            NodeRunStatus::Cached,
            NodeRunStatus::Done,
            NodeRunStatus::Error,
            NodeRunStatus::Skipped,
        ]))
    }

    proptest! {
        #[test]
        fn only_settled_rows_read_as_changed(status in status(), failed in any::<bool>(), changed in any::<bool>()) {
            let state = row_state(status, failed, changed);
            let unchanged = row_state(status, failed, false);
            if state == "changed" {
                prop_assert!(changed && matches!(unchanged, "done" | "cached" | "fromLastRun"));
            } else {
                prop_assert_eq!(state, unchanged);
            }
        }
    }
}
