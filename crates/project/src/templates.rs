use std::sync::LazyLock;

use opsis_compile::ids::random_id;
use opsis_types::cards::Icon;
use opsis_types::workflow::Workflow;
use serde::{Deserialize, Serialize};
use specta::Type;

pub const DEFAULT_PROJECT_NAME: &str = "Demo";
const BLANK_TEMPLATE: &str = "blank";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TemplateInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon: Icon,
}

#[derive(Deserialize)]
struct Template {
    #[serde(flatten)]
    info: TemplateInfo,
    workflow: Workflow,
}

static TEMPLATES: LazyLock<Vec<Template>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("templates.json")).expect("templates parse")
});

pub fn templates() -> Vec<TemplateInfo> {
    TEMPLATES.iter().map(|t| t.info.clone()).collect()
}

pub fn build_template(id: &str, name: &str) -> Option<Workflow> {
    let template = TEMPLATES.iter().find(|t| t.info.id == id)?;
    let mut workflow = template.workflow.clone();
    workflow.id = random_id("wf");
    workflow.name = match name.trim() {
        "" => DEFAULT_PROJECT_NAME.to_owned(),
        name => name.to_owned(),
    };
    Some(workflow)
}

pub fn examples() -> Vec<(String, Workflow)> {
    TEMPLATES
        .iter()
        .filter(|t| t.info.id != BLANK_TEMPLATE)
        .map(|t| {
            let mut workflow = t.workflow.clone();
            workflow.id = format!("example_{}", t.info.id.replace('-', "_"));
            workflow.name.clone_from(&t.info.title);
            (t.info.id.clone(), workflow)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use opsis_compile::check::workflow_problems;

    use super::*;

    #[test]
    fn every_template_builds_a_named_workflow_with_a_fresh_id_and_no_problems() {
        let infos = templates();
        assert_eq!(infos.first().map(|t| t.id.as_str()), Some("blank"));
        for info in infos {
            let one = build_template(&info.id, "  ").unwrap();
            let two = build_template(&info.id, "Launch").unwrap();
            assert_eq!(
                (one.name.as_str(), two.name.as_str()),
                (DEFAULT_PROJECT_NAME, "Launch")
            );
            assert_ne!(one.id, two.id);
            assert!(
                one.id.starts_with("wf_") && one.id.len() == 11,
                "{}",
                one.id
            );
            assert_eq!(
                workflow_problems(&one)
                    .iter()
                    .filter(|p| p.level == opsis_compile::graph::IssueLevel::Error)
                    .count(),
                0,
                "{}",
                info.id
            );
        }
        assert_eq!(build_template("missing", "x"), None);
    }
}
