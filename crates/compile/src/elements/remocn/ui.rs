use opsis_types::looks::NodeLook;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::choices::{
    AgentStepIcon, DarkOrLight, FontWeight, LightOrDark, Orientation, TerminalLineKind,
};
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct TerminalSimulatorLine {
    #[default = "Done"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default(TerminalLineKind::Log)]
    #[schemars(description = "Kind")]
    pub r#type: TerminalLineKind,
    #[default = 0.2667]
    #[schemars(description = "Wait before", range(min = 0, max = 20), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub delay: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct CodeMorphStep {
    #[schemars(description = "Code", extend("x-widget" = "code"))]
    pub code: String,
    #[schemars(description = "Show at", range(min = 0, max = 120), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub at: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoEnterLogo {
    #[default = "app:placeholders/logo-mark.svg"]
    #[schemars(description = "Image", extend("x-widget" = "asset", "x-accept" = "image/*"))]
    pub image: String,
    #[default = "#2563eb"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub background: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct CheckListItem {
    #[default = "New item"]
    #[schemars(description = "Text")]
    pub text: String,
    #[default = true]
    #[schemars(description = "Ticked")]
    pub checked: bool,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct AgentRunRunStep {
    #[default = "Search logs"]
    #[schemars(description = "Name")]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Detail", extend("type" = "string"))]
    pub detail: Option<String>,
    #[default = 1.5]
    #[schemars(description = "Length", range(min = 0.2, max = 20), extend("x-step" = 0.1, "x-unit" = "s"))]
    pub duration: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Icon", extend("type" = "string", "enum" = ["chart","database","search","book","globe","terminal","file"]))]
    pub icon: Option<AgentStepIcon>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Sources", extend("type" = "array"))]
    pub sources: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct TerminalSimulator {
    #[default = "$"]
    #[schemars(description = "Prompt")]
    pub prompt: String,
    #[default = "~/projects/acme"]
    #[schemars(description = "Title")]
    pub title: String,
    #[default = "#0a0a0a"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub background: String,
    #[default = "#1a1a1a"]
    #[schemars(description = "Title bar colour", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub chrome_color: String,
    #[default = 18.0]
    #[schemars(description = "Font size", range(min = 10, max = 32), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = 1.0]
    #[schemars(description = "Chars / frame", range(min = 0.25, max = 6), extend("x-step" = 0.25))]
    pub chars_per_frame: f64,
    #[default = 1.0]
    #[schemars(description = "Chunk size", range(min = 1, max = 20), extend("x-step" = 1))]
    pub chunk_size: f64,
    #[default(vec![
        TerminalSimulatorLine { text: "deploy --production".into(), r#type: TerminalLineKind::Command, delay: 0.0 },
        TerminalSimulatorLine { text: "Running tests...".into(), delay: 0.2, ..Default::default() },
        TerminalSimulatorLine { text: "128 tests passed".into(), r#type: TerminalLineKind::Success, delay: 0.3333 },
        TerminalSimulatorLine { text: "Building the image...".into(), ..Default::default() },
        TerminalSimulatorLine { text: "Image ready in 41s".into(), r#type: TerminalLineKind::Success, delay: 0.3333 },
        TerminalSimulatorLine { text: "Rolling out to everyone...".into(), ..Default::default() },
        TerminalSimulatorLine { text: "Live in production".into(), r#type: TerminalLineKind::Success, delay: 0.4 },
    ])]
    #[schemars(description = "Lines", extend("x-help" = "Lines ending in ... pause before the next one."))]
    pub lines: Vec<TerminalSimulatorLine>,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct GlassCodeBlock {
    #[default = "hero.tsx"]
    #[schemars(description = "Title")]
    pub title: String,
    #[default = 760.0]
    #[schemars(description = "Width", range(min = 300, max = 1100), extend("x-step" = 10))]
    pub width: f64,
    #[default = 460.0]
    #[schemars(description = "Height", range(min = 200, max = 700), extend("x-step" = 10))]
    pub height: f64,
    #[default = 16.0]
    #[schemars(description = "Font size", range(min = 10, max = 28), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = "rgba(10, 10, 10, 0.6)"]
    #[schemars(description = "Glass color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub glass_color: String,
    #[default = 0.1333]
    #[schemars(description = "Line stagger", range(min = 0, max = 1), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub stagger_frames: f64,
    #[default = true]
    #[schemars(description = "Traffic lights")]
    pub show_traffic_lights: bool,
    #[default = "export async function publish(draft: Draft) {\n  const page = await render(draft);\n  await upload(page, { public: true });\n  return page.url;\n}"]
    #[schemars(description = "Code", extend("x-widget" = "code"))]
    pub code: String,
    #[schemars(description = "Glow behind the window")]
    pub aura: bool,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct CodeMorph {
    #[default = "invoices.tsx"]
    #[schemars(description = "Filename")]
    pub filename: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme")]
    pub theme: DarkOrLight,
    #[default = "#0ea5e9"]
    #[schemars(description = "Accent", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default = 1.2]
    #[schemars(description = "Morph time", range(min = 0.4, max = 3), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub morph_duration: f64,
    #[default = 20.0]
    #[schemars(description = "Font size", range(min = 12, max = 32), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = true]
    #[schemars(description = "Highlight changes")]
    pub highlight: bool,
    #[default = true]
    #[schemars(description = "Line numbers")]
    pub line_numbers: bool,
    #[default = true]
    #[schemars(description = "Window dots")]
    pub window_dots: bool,
    #[default(vec![
        CodeMorphStep { code: "import { loadInvoices } from \"./api\";\n\nexport async function showInvoices(list: HTMLElement) {\n  list.textContent = \"Loading…\";\n  const invoices = await loadInvoices();\n  list.textContent = \"\";\n  for (const invoice of invoices) {\n    const row = document.createElement(\"li\");\n    row.textContent = invoice.title;\n    list.append(row);\n  }\n}".into(), ..Default::default() },
        CodeMorphStep { at: 1.6, code: "import { loadInvoices } from \"./api\";\nimport { renderRows } from \"./rows\";\n\nexport async function showInvoices(list: HTMLElement) {\n  list.textContent = \"Loading…\";\n  renderRows(list, await loadInvoices());\n}".into() },
    ])]
    #[schemars(description = "Versions", extend("x-help" = "Each version of the code and when it starts to show."))]
    pub steps: Vec<CodeMorphStep>,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct LogoEnter {
    #[default = 118.0]
    #[schemars(description = "Diameter", range(min = 48, max = 320), extend("x-step" = 1, "x-unit" = "px"))]
    pub diameter: f64,
    #[default = 38.0]
    #[schemars(description = "Overlap", range(min = 0, max = 200), extend("x-step" = 1, "x-unit" = "px"))]
    pub overlap: f64,
    #[default = "#fff"]
    #[schemars(description = "Ring", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub ring_color: String,
    #[default(Orientation::Horizontal)]
    #[schemars(description = "Orientation")]
    pub orientation: Orientation,
    #[default = 0.2333]
    #[schemars(description = "Stagger", range(min = 0), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub stagger: f64,
    #[default(vec![
        LogoEnterLogo::default(),
        LogoEnterLogo { background: "#0f766e".into(), ..Default::default() },
        LogoEnterLogo { background: "#b45309".into(), ..Default::default() },
        LogoEnterLogo { background: "#7c3aed".into(), ..Default::default() },
    ])]
    #[schemars(description = "Logos", length(min = 1, max = 16))]
    pub logos: Vec<LogoEnterLogo>,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct CheckList {
    #[default = 40.0]
    #[schemars(description = "Font size", range(min = 24, max = 64), extend("x-step" = 2))]
    pub font_size: f64,
    #[default = 0.6]
    #[schemars(description = "Time between items", range(min = 0.2, max = 1.6), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub item_gap: f64,
    #[default = 0.3]
    #[schemars(description = "Time between ticks", range(min = 0.1, max = 0.8), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub close_gap: f64,
    #[default = 1.6]
    #[schemars(description = "Letters / pose", range(min = 0.8, max = 3), extend("x-step" = 0.2))]
    pub per_step: f64,
    #[default = 3.0]
    #[schemars(description = "Stroke width", range(min = 1, max = 8), extend("x-step" = 1))]
    pub stroke_width: f64,
    #[default = "#26242c"]
    #[schemars(description = "Label color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default = "#6f7f35"]
    #[schemars(description = "Tick color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub tick_color: String,
    #[default = 0.1]
    #[schemars(description = "Time per pose", range(min = 0.0333, max = 0.2), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub step: f64,
    #[default(vec![
        CheckListItem { text: "Record the walkthrough".into(), ..Default::default() },
        CheckListItem { text: "Add captions and zooms".into(), ..Default::default() },
        CheckListItem { text: "Export for the launch".into(), checked: false },
    ])]
    #[schemars(description = "Items")]
    pub items: Vec<CheckListItem>,
    #[default = 720.0]
    #[schemars(description = "Width", range(min = 200, max = 1800), extend("x-step" = 10, "x-unit" = "px"))]
    pub width: f64,
    #[default = "#26242c"]
    #[schemars(description = "Box colour", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub box_color: String,
    #[default(FontWeight::W600)]
    #[schemars(description = "Font weight")]
    pub weight: FontWeight,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Row gap", range(min = 0, max = 200), extend("x-zero-label" = "Auto", "x-step" = 1, "x-unit" = "px", "x-advanced" = true, "type" = "number"))]
    pub row_gap: Option<f64>,
    #[default = "checklist"]
    #[schemars(description = "Variation", extend("x-help" = "Change it to draw the handwriting a little differently.", "x-advanced" = true))]
    pub seed: String,
    #[default = "white"]
    #[schemars(description = "Background", extend("x-help" = "Set it to transparent to lay the text over other layers.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub background: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SearchReveal {
    #[default = "acme"]
    #[schemars(description = "Search text")]
    pub text: String,
    #[default = 464.0]
    #[schemars(description = "Field width", range(min = 320, max = 640), extend("x-step" = 8))]
    pub field_width: f64,
    #[default = 34.0]
    #[schemars(description = "Font size", range(min = 18, max = 54), extend("x-step" = 1))]
    pub font_size: f64,
    #[default = 0.1333]
    #[schemars(description = "Time per letter", range(min = 0.0333, max = 0.4), extend("x-step" = 0.01, "x-unit" = "s", "x-frames-at" = 30))]
    pub frames_per_character: f64,
    #[default = true]
    #[schemars(description = "Graphic panel")]
    pub show_panel: bool,
    #[default = true]
    #[schemars(description = "Construction guides")]
    pub show_guides: bool,
    #[default = "#26232b"]
    #[schemars(description = "Text and markers", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub color: String,
    #[default = "#ffffff"]
    #[schemars(description = "Search field", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub field_color: String,
    #[default = "#cbc5cf"]
    #[schemars(description = "Circles and search icon", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub ring_color: String,
    #[default = "#a800b7"]
    #[schemars(description = "Panel color", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub panel_color: String,
    #[default = "#e8f99a"]
    #[schemars(description = "Graphic accents", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default = "#f5f1f5"]
    #[schemars(description = "Background", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub background_color: String,
    #[schemars(description = "Static final pose")]
    pub reduced_motion: bool,
    #[schemars(description = "Font", extend("x-widget" = "font", "x-help" = "Project font follows the fonts set for the whole project."))]
    pub font_family: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct AgentRun {
    #[default(LightOrDark::Light)]
    #[schemars(description = "Theme")]
    pub theme: LightOrDark,
    #[default = "#2563eb"]
    #[schemars(description = "Accent", extend("x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default = 3.0]
    #[schemars(description = "Compression", range(min = 1, max = 8), extend("x-step" = 0.5))]
    pub compression: f64,
    #[default = 1.0]
    #[schemars(description = "Seed", range(min = 1, max = 99), extend("x-step" = 1))]
    pub seed: f64,
    #[default = "Checkout conversion drop"]
    #[schemars(description = "Title")]
    pub run_title: String,
    #[default = "Why did checkout conversion drop last week?"]
    #[schemars(description = "Prompt")]
    pub run_prompt: String,
    #[default(vec!["Compare the checkout funnel with the week before".into(), "Break the drop down by device and step".into(), "Check what shipped and what the error logs say".into()])]
    #[schemars(description = "Plan")]
    pub run_plan: Vec<String>,
    #[default(vec![
        AgentRunRunStep { icon: Some(AgentStepIcon::Chart), name: "Query analytics".into(), detail: Some("checkout funnel, last 7 days vs prior 7".into()), duration: 1.4, ..Default::default() },
        AgentRunRunStep { icon: Some(AgentStepIcon::Database), name: "Run a query".into(), detail: Some("conversion by device and step".into()), duration: 2.3, ..Default::default() },
        AgentRunRunStep { icon: Some(AgentStepIcon::Search), detail: Some("card payment errors on tablets".into()), duration: 1.1, ..Default::default() },
        AgentRunRunStep { icon: Some(AgentStepIcon::Book), name: "Read 12 sources".into(), duration: 3.2, sources: Some(vec!["Checkout funnel".into(), "Device report".into(), "Release 4.12".into(), "Change #2186".into(), "Payments guide".into(), "Error log".into(), "Incident notes".into(), "Runbook".into(), "Support tickets".into(), "Status page".into(), "Team notes".into(), "Design spec".into()]), ..Default::default() },
    ])]
    #[schemars(description = "Steps")]
    pub run_steps: Vec<AgentRunRunStep>,
    #[default = "## Card payments broke on tablets in release 4.12\n\nCheckout conversion fell from 3.4% to 2.7% last week, a 21% drop [1]. All of it comes from the payment step on tablets, 41% of checkouts [2].\n\n- Tablet conversion halved, from 3.5% to 1.8%\n- Phones and desktops held steady at 3.3%\n- 4.12 moved the card form into a frame without the payment permission [3]\n\n```tsx\n<iframe\n  src={cardFormUrl}\n  allow=\"payment\"\n/>\n```\n\n**Fix:** add the permission and ship 4.12.1 today."]
    #[schemars(description = "Answer", extend("x-widget" = "textarea", "x-help" = "Use ## for a heading, - for a list, **bold**, `code` and [1] to cite a source."))]
    pub run_answer: String,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25))]
    pub speed: f64,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<TerminalSimulator>(
            element(
                ElementKind::Component,
                "terminal-simulator",
                "Terminal simulator",
                "Terminal window that types out commands and rolls old lines off the top.",
                C::Ui,
                240,
            )
                .source("Remocn", "https://remocn.dev/docs/components/terminal-simulator", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:square-terminal")),
        ),
        meta::<GlassCodeBlock>(
            element(
                ElementKind::Component,
                "glass-code-block",
                "Glass code block",
                "A frosted-glass code editor that reveals the code line by line.",
                C::Ui,
                180,
            )
                .source("Remocn", "https://remocn.dev/docs/components/glass-code-block", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:square-code")),
        ),
        meta::<CodeMorph>(
            element(
                ElementKind::Component,
                "code-morph",
                "Code morph",
                "A flat code editor that morphs between versions of a file, gliding each surviving token to its new line and column while removed code dissolves and new code fades in.",
                C::Ui,
                138,
            )
                .source("Remocn", "https://remocn.dev/docs/components/code-morph", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:code-xml").morphs("lucide:braces")),
        ),
        meta::<LogoEnter>(
            element(
                ElementKind::Component,
                "logo-enter",
                "Logo enter",
                "A stacked group of round brand chips that spring in one-by-one, then hold. Swap the logos to feature any set of marks.",
                C::Ui,
                90,
            )
                .source("Remocn", "https://remocn.dev/docs/components/logo-enter", "MIT")
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:layout-grid-circles")),
        ),
        meta::<CheckList>(
            element(
                ElementKind::Component,
                "check-list",
                "Check list",
                "A handwritten checklist that writes itself out in full, then ticks and strikes through every done item, one at a time.",
                C::Ui,
                360,
            )
                .source("Remocn", "https://remocn.dev/docs/components/check-list", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:list-checks")),
        ),
        meta::<SearchReveal>(
            element(
                ElementKind::Component,
                "search-reveal",
                "Search reveal",
                "Expanding construction circles snap into a search pill, type editable text, and reveal an abstract graphic panel.",
                C::Ui,
                108,
            )
                .source("Remocn", "https://remocn.dev/docs/components/search-reveal", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:search")),
        ),
        meta::<AgentRun>(
            element(
                ElementKind::Component,
                "agent-run",
                "Agent run",
                "An AI assistant working through a plan, ticking off each step.",
                C::Ui,
                355,
            )
                .source("Remocn", "https://remocn.dev/docs/components/agent-run", "MIT")
                .authored_clock()
                .look(NodeLook::of("lucide:bot")),
        ),
    ]
}
