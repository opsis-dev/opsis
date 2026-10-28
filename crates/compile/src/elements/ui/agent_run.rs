use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::entry::{element, meta};
use crate::elements::shared::LightOrDark;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "kebab-case")]
pub enum AgentStepIcon {
    Chart,
    Database,
    Search,
    Book,
    Globe,
    Terminal,
    File,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct AgentRunRunStep {
    #[default = "Search logs"]
    #[schemars(description = "Name")]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Detail", extend("type" = "string"))]
    pub detail: Option<String>,
    #[default = 1.5]
    #[schemars(
        description = "Length",
        range(min = 0.2, max = 20),
        extend("x-step" = 0.1, "x-unit" = "s")
    )]
    pub duration: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        description = "Icon",
        extend(
            "type" = "string",
            "enum" = ["chart", "database", "search", "book", "globe", "terminal", "file"]
        )
    )]
    pub icon: Option<AgentStepIcon>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Sources", extend("type" = "array"))]
    pub sources: Option<Vec<String>>,
}

#[derive(
    Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema, specta::Type,
)]
#[serde(rename_all = "camelCase", default)]
pub struct AgentRun {
    #[default(LightOrDark::Light)]
    #[schemars(description = "Theme")]
    pub theme: LightOrDark,
    #[default = "#2563eb"]
    #[schemars(
        description = "Accent",
        extend("x-check" = { "rule": "colour", "message": "Pick a colour" })
    )]
    pub accent_color: String,
    #[default = 3.0]
    #[schemars(description = "Compression", range(min = 1, max = 8), extend("x-step" = 0.5))]
    pub compression: f64,
    #[default = 1]
    #[schemars(description = "Seed", range(min = 1, max = 99), extend("x-step" = 1))]
    pub seed: u32,
    #[default = "Checkout conversion drop"]
    #[schemars(description = "Title")]
    pub run_title: String,
    #[default = "Why did checkout conversion drop last week?"]
    #[schemars(description = "Prompt")]
    pub run_prompt: String,
    #[default(vec![
        "Compare the checkout funnel with the week before".into(),
        "Break the drop down by device and step".into(),
        "Check what shipped and what the error logs say".into(),
    ])]
    #[schemars(description = "Plan")]
    pub run_plan: Vec<String>,
    #[default(vec![
        AgentRunRunStep {
            icon: Some(AgentStepIcon::Chart),
            name: "Query analytics".into(),
            detail: Some("checkout funnel, last 7 days vs prior 7".into()),
            duration: 1.4,
            ..Default::default()
        },
        AgentRunRunStep {
            icon: Some(AgentStepIcon::Database),
            name: "Run a query".into(),
            detail: Some("conversion by device and step".into()),
            duration: 2.3,
            ..Default::default()
        },
        AgentRunRunStep {
            icon: Some(AgentStepIcon::Search),
            detail: Some("card payment errors on tablets".into()),
            duration: 1.1,
            ..Default::default()
        },
        AgentRunRunStep {
            icon: Some(AgentStepIcon::Book),
            name: "Read 12 sources".into(),
            duration: 3.2,
            sources: Some(vec!["Checkout funnel".into(), "Device report".into(), "Release 4.12".into(), "Change #2186".into(), "Payments guide".into(), "Error log".into(), "Incident notes".into(), "Runbook".into(), "Support tickets".into(), "Status page".into(), "Team notes".into(), "Design spec".into()]),
            ..Default::default()
        },
    ])]
    #[schemars(description = "Steps")]
    pub run_steps: Vec<AgentRunRunStep>,
    #[default = "## Card payments broke on tablets in release 4.12\n\nCheckout conversion fell from 3.4% to 2.7% last week, a 21% drop [1]. All of it comes from the payment step on tablets, 41% of checkouts [2].\n\n- Tablet conversion halved, from 3.5% to 1.8%\n- Phones and desktops held steady at 3.3%\n- 4.12 moved the card form into a frame without the payment permission [3]\n\n```tsx\n<iframe\n  src={cardFormUrl}\n  allow=\"payment\"\n/>\n```\n\n**Fix:** add the permission and ship 4.12.1 today."]
    #[schemars(
        description = "Answer",
        extend(
            "x-widget" = "textarea",
            "x-help" = "Use ## for a heading, - for a list, **bold**, `code` and [1] to cite a source."
        )
    )]
    pub run_answer: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

pub fn definition() -> ElementMeta {
    meta::<AgentRun>(
        element(
            ElementKind::Component,
            "agent-run",
            "Agent run",
            "An AI assistant working through a plan, ticking off each step.",
            ElementCategory::Ui,
            355,
        )
        .authored_clock()
        .look(NodeLook::of("lucide:bot")),
    )
}
