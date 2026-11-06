use std::collections::HashSet;

use opsis_types::workflow::{Position, Workflow, WorkflowEdge, WorkflowNode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CLIPBOARD_TAG: &str = "opsis/v1";
pub const PASTE_OFFSET: Position = Position { x: 60.0, y: 60.0 };
pub const UNREADABLE: &str =
    "Couldn't paste the copied nodes: the clipboard holds nodes this project can't read.";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subgraph {
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
}

#[derive(Debug, Serialize)]
struct Tagged<'a> {
    tag: &'static str,
    nodes: &'a [WorkflowNode],
    edges: &'a [WorkflowEdge],
}

#[derive(Debug, PartialEq)]
pub enum Pasted {
    Foreign,
    Unreadable,
    Nodes(Subgraph),
}

pub fn selected(wf: &Workflow, ids: &[String]) -> Option<Subgraph> {
    let chosen: HashSet<&str> = ids.iter().map(String::as_str).collect();
    let nodes: Vec<WorkflowNode> = wf
        .nodes
        .iter()
        .filter(|node| chosen.contains(node.id.as_str()))
        .cloned()
        .collect();
    if nodes.is_empty() {
        return None;
    }
    let edges = wf
        .edges
        .iter()
        .filter(|edge| {
            chosen.contains(edge.source.as_str()) && chosen.contains(edge.target.as_str())
        })
        .cloned()
        .collect();
    Some(Subgraph { nodes, edges })
}

pub fn copy_text(subgraph: &Subgraph) -> String {
    serde_json::to_string(&Tagged {
        tag: CLIPBOARD_TAG,
        nodes: &subgraph.nodes,
        edges: &subgraph.edges,
    })
    .unwrap_or_default()
}

pub fn read(text: &str) -> Pasted {
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return Pasted::Foreign;
    };
    if value.get("tag").and_then(Value::as_str) != Some(CLIPBOARD_TAG) {
        return Pasted::Foreign;
    }
    serde_json::from_value::<Subgraph>(value).map_or(Pasted::Unreadable, Pasted::Nodes)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn todo_mvc() -> Workflow {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/todomvc")
            .join(opsis_types::workflow::WORKFLOW_FILE);
        serde_json::from_str(&std::fs::read_to_string(path).expect("the example"))
            .expect("a workflow")
    }

    proptest! {
        #[test]
        fn copied_nodes_paste_back_as_the_same_subgraph(mask in proptest::collection::vec(any::<bool>(), 40)) {
            let wf = todo_mvc();
            let ids: Vec<String> = wf
                .nodes
                .iter()
                .zip(mask.iter().cycle())
                .filter(|(_, keep)| **keep)
                .map(|(node, _)| node.id.clone())
                .collect();
            match selected(&wf, &ids) {
                None => prop_assert!(ids.is_empty()),
                Some(subgraph) => {
                    prop_assert!(subgraph.edges.iter().all(|edge| ids.contains(&edge.source) && ids.contains(&edge.target)));
                    prop_assert_eq!(read(&copy_text(&subgraph)), Pasted::Nodes(subgraph));
                }
            }
        }
    }

    proptest! {
        #[test]
        fn text_without_the_tag_is_left_alone(text in "[^{].*") {
            prop_assert_eq!(read(&text), Pasted::Foreign);
        }

        #[test]
        fn tagged_text_of_the_wrong_shape_is_unreadable(count in any::<u32>()) {
            let text = format!(r#"{{"tag":"{CLIPBOARD_TAG}","nodes":{count}}}"#);
            prop_assert_eq!(read(&text), Pasted::Unreadable);
        }
    }
}
