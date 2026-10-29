use opsis_macros::settings;
use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::theme::{Theme, neutral_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings]
#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct AiPromptFlow {
    #[setting("Theme", transform = neutral_themes, widget = "theme")]
    pub theme: Theme,
    #[setting("Prompt", default = "Summarize this thread")]
    pub prompt: String,
    #[setting("Button label", default = "Generate")]
    pub button_label: String,
    #[setting(
        "Answer",
        default = vec![
            "The thread debates the Q3 roadmap: ship the editor first,".into(),
            "defer billing to Q4, and pull the migration forward so".into(),
            "infra is unblocked before the team scales next quarter.".into(),
        ],
        help = "One line of the answer per item."
    )]
    pub answer_lines: Vec<String>,
    #[setting("Notice", default = "Response ready")]
    pub toast_title: String,
}

pub fn definition() -> ElementMeta {
    meta::<AiPromptFlow>(
            element(
                ElementKind::Component,
                "ai-prompt-flow",
                "AI prompt flow",
                "An AI prompt composition: a prompt types into the field, the Generate button runs hover → press → loading, a skeleton shimmer reveals, then crossfades into the generated answer, and a ready toast slides in. A pure orchestrator — the shimmer is owned by the skeleton-block motion atom; every other channel comes from a composed primitive's hook.",
                ElementCategory::Scene,
                240,
            )
                .authored_clock()
                .exact_design(1280, 720)
                .look(NodeLook::of("lucide:wand-sparkles")),
        )
}
