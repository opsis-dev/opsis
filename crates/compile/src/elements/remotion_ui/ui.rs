use opsis_types::looks::{NodeLook, NodeMotion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::elements::choices::DarkOrLight;
use crate::elements::core::Colour;
use crate::elements::entry::{element, meta};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct KanbanMoveCard {
    #[default = "CSV export"]
    #[schemars(description = "Title")]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Meta", extend("type" = "string"))]
    pub meta: Option<String>,
    #[default = 2.0]
    #[schemars(description = "Column")]
    pub column: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Tint", extend("x-widget" = "color", "x-zero-label" = "None", "type" = "string"))]
    pub tint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Move to", extend("type" = "number"))]
    pub move_to: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Move at seconds", extend("type" = "number"))]
    pub move_at_seconds: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct CommitGraphCommit {
    #[default = "Merge retry backoff"]
    #[schemars(description = "Message")]
    pub message: String,
    #[default = "b8c4e05"]
    #[schemars(description = "Hash")]
    pub hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Lane", extend("type" = "number"))]
    pub lane: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Parent", extend("type" = "number"))]
    pub parent: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Merge from", extend("type" = "number"))]
    pub merge_from: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Ref", extend("type" = "string"))]
    pub r#ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct CodeReveal {
    #[default = "export async function checkout(cart: Cart) {\n  const total = cart.items.reduce((sum, item) => sum + item.price, 0);\n  const order = await api.orders.create({ items: cart.items, total });\n\n  return order.id;\n}"]
    #[schemars(description = "Code", extend("x-widget" = "code", "x-help" = "Source shown in the editor; surrounding blank lines are trimmed."))]
    pub code: String,
    #[default(vec![3.0])]
    #[schemars(description = "Highlighted lines", extend("x-help" = "Line numbers as shown in the gutter, focused once the listing finishes writing."))]
    pub highlighted_lines: Vec<f64>,
    #[default = "checkout.ts"]
    #[schemars(description = "Title", extend("x-help" = "Filename on the editor tab."))]
    pub title: String,
    #[default = "tsx"]
    #[schemars(description = "Language", extend("x-help" = "Language badge on the right of the header.", "x-advanced" = true))]
    pub language: String,
    #[default = 1.0]
    #[schemars(description = "Start line", extend("x-help" = "First line number in the gutter, for excerpts.", "x-advanced" = true))]
    pub start_line: f64,
    #[default = true]
    #[schemars(description = "Line numbers", extend("x-help" = "Shows the gutter."))]
    pub show_line_numbers: bool,
    #[default = "#E8B86D"]
    #[schemars(description = "Accent color", extend("x-help" = "Caret, focus band and glow color.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Editor palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-help" = "Animation speed multiplier."))]
    pub speed: f64,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Leave empty to follow the theme."))]
    pub background_color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct CodeDiffWipe {
    #[default = "export function render(scene) {\n  const frames = scene.frames;\n  return frames.map(draw);\n}"]
    #[schemars(description = "Before", extend("x-widget" = "code", "x-help" = "Source before the patch."))]
    pub before: String,
    #[default = "export function render(scene, options) {\n  const frames = scene.frames;\n  const scale = options.scale ?? 1;\n  return frames.map((frame) => draw(frame, scale));\n}"]
    #[schemars(description = "After", extend("x-widget" = "code", "x-help" = "Source after the patch; the scene diffs the two."))]
    pub after: String,
    #[default = "render.ts"]
    #[schemars(description = "Title", extend("x-help" = "Filename on the window header."))]
    pub title: String,
    #[default = 1.7]
    #[schemars(description = "Wipe (seconds)", range(min = 0.05, max = 10), extend("x-step" = 0.01, "x-help" = "Seconds the apply front takes to travel the file.", "x-advanced" = true))]
    pub wipe_seconds: f64,
    #[default = "#E8B86D"]
    #[schemars(description = "Accent color", extend("x-help" = "Apply front and glow color.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Editor palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-help" = "Animation speed multiplier."))]
    pub speed: f64,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Leave empty to follow the theme."))]
    pub background_color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct KanbanMove {
    #[default(vec!["Backlog".into(), "In progress".into(), "Shipped".into()])]
    #[schemars(description = "Columns", extend("x-help" = "Column headings, left to right. Cards address columns by index."))]
    pub columns: Vec<String>,
    #[default(vec![
        KanbanMoveCard { title: "Onboarding checklist".into(), meta: Some("APP-218 · Ana".into()), column: 0.0, tint: Some("#8FB8F0".into()), ..Default::default() },
        KanbanMoveCard { title: "Retry failed payments".into(), meta: Some("APP-204 · Piotr".into()), column: 0.0, tint: Some("#E8B86D".into()), move_to: Some(1.0), move_at_seconds: Some(0.95) },
        KanbanMoveCard { title: "Dark mode".into(), meta: Some("APP-231 · Dai".into()), column: 0.0, tint: Some("#C99BE8".into()), ..Default::default() },
        KanbanMoveCard { title: "Search filters".into(), meta: Some("APP-190 · Ana".into()), column: 1.0, tint: Some("#7DD3E8".into()), move_to: Some(2.0), move_at_seconds: Some(1.72) },
        KanbanMoveCard { title: "Team invites".into(), meta: Some("APP-177 · Kit".into()), column: 1.0, tint: Some("#9BD4A0".into()), ..Default::default() },
        KanbanMoveCard { meta: Some("APP-166 · Dai".into()), tint: Some("#9BD4A0".into()), ..Default::default() },
    ])]
    #[schemars(description = "Cards", extend("x-help" = "Title, meta line, starting column, rail tint, and the one move the card makes."))]
    pub cards: Vec<KanbanMoveCard>,
    #[default = 0.62]
    #[schemars(description = "Move time (seconds)", range(min = 0.05, max = 10), extend("x-step" = 0.01, "x-help" = "How long a card takes to arc from column to column.", "x-advanced" = true))]
    pub move_seconds: f64,
    #[default = 0.08]
    #[schemars(description = "Card stagger (seconds)", extend("x-step" = 0.01, "x-help" = "Seconds between each card's arrival during the opening deal.", "x-advanced" = true))]
    pub deal_stagger_seconds: f64,
    #[default = "#E8B86D"]
    #[schemars(description = "Accent color", extend("x-help" = "Destination column glow, and the fallback card rail.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-help" = "Animation speed multiplier."))]
    pub speed: f64,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Leave empty to follow the theme."))]
    pub background_color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct FileTreeReveal {
    #[default(vec!["src/components/cart.tsx".into(), "src/components/checkout.tsx".into(), "src/lib/api.ts".into(), "src/index.ts".into(), "public/logo.svg".into(), "README.md".into(), "package.json".into()])]
    #[schemars(description = "Files", extend("x-help" = "One path per item, like src/app.ts. End a folder with / to show it empty."))]
    pub nodes: Vec<String>,
    #[default = "acme-app"]
    #[schemars(description = "Title", extend("x-help" = "Title bar over the tree. Omit to drop the header."))]
    pub title: String,
    #[default = "src/components/checkout.tsx"]
    #[schemars(description = "Highlighted file", extend("x-help" = "Slash-joined path of the file that lights up at the end. Omit to select nothing."))]
    pub selected_path: String,
    #[default = 0.18]
    #[schemars(description = "Row stagger", extend("x-step" = 0.01, "x-help" = "Seconds between one row appearing and the next.", "x-advanced" = true))]
    pub row_stagger: f64,
    #[default = "#E8B86D"]
    #[schemars(description = "Accent color", extend("x-help" = "Selected row's bar, tint and label.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-help" = "Animation speed multiplier."))]
    pub speed: f64,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Leave empty to follow the theme."))]
    pub background_color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct DragDropFlow {
    #[default = "hero-take.mp4"]
    #[schemars(description = "File name", extend("x-help" = "File the cursor picks up."))]
    pub file_name: String,
    #[default = "48.2 MB"]
    #[schemars(description = "File size", extend("x-help" = "Size shown once the upload completes."))]
    pub file_size: String,
    #[default(vec!["b-roll-rooftop.mov".into(), "interview-cam-b.mp4".into()])]
    #[schemars(description = "Other files", extend("x-help" = "Other rows in the source list."))]
    pub siblings: Vec<String>,
    #[default = "Media library"]
    #[schemars(description = "Library title", extend("x-help" = "Heading on the source panel."))]
    pub source_label: String,
    #[default = "Drop your clip"]
    #[schemars(description = "Label", extend("x-help" = "Idle prompt in the drop zone."))]
    pub label: String,
    #[default = "MP4 or MOV, up to 2 GB"]
    #[schemars(description = "Hint", extend("x-help" = "Second line under the prompt."))]
    pub hint: String,
    #[default = "Release to upload"]
    #[schemars(description = "Label while dragging", extend("x-help" = "Prompt while the file is held over the zone."))]
    pub active_label: String,
    #[default = "Uploaded"]
    #[schemars(description = "Label when done", extend("x-help" = "Label once the upload finishes."))]
    pub done_label: String,
    #[default = "#E8B86D"]
    #[schemars(description = "Accent color", extend("x-help" = "Zone, cursor ring, and progress colour.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-help" = "Animation speed multiplier."))]
    pub speed: f64,
    #[default = "Uploading"]
    #[schemars(description = "Label while uploading", extend("x-help" = "Label while the file uploads."))]
    pub uploading_label: String,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Leave empty to follow the theme."))]
    pub background_color: String,
}

#[derive(Debug, Clone, PartialEq, SmartDefault, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct CommitGraph {
    #[default(vec![
        CommitGraphCommit { message: "Seed the registry manifest".into(), hash: "9c41f0a".into(), ..Default::default() },
        CommitGraphCommit { message: "Add render queue worker".into(), hash: "3ab77e2".into(), ..Default::default() },
        CommitGraphCommit { message: "Branch: retry backoff".into(), hash: "d0e91c4".into(), lane: Some(1.0), parent: Some(1.0), ..Default::default() },
        CommitGraphCommit { message: "Cap retries at five".into(), hash: "51b2fa8".into(), lane: Some(1.0), ..Default::default() },
        CommitGraphCommit { message: "Ship caption presets".into(), hash: "7f30dd1".into(), lane: Some(0.0), parent: Some(1.0), ..Default::default() },
        CommitGraphCommit { lane: Some(0.0), parent: Some(4.0), merge_from: Some(3.0), r#ref: Some("main".into()), ..Default::default() },
    ])]
    #[schemars(description = "Commits")]
    pub commits: Vec<CommitGraphCommit>,
    #[default = "git log --graph --oneline"]
    #[schemars(description = "Window title", extend("x-help" = "Text in the title bar. Omit to drop the chrome header."))]
    pub window_title: String,
    #[default = 0.34]
    #[schemars(description = "Start at (seconds)", extend("x-step" = 0.01, "x-help" = "Second the first commit lands.", "x-advanced" = true))]
    pub start_at_seconds: f64,
    #[default = 0.36]
    #[schemars(description = "Step (seconds)", range(min = 0.05, max = 10), extend("x-step" = 0.01, "x-help" = "Seconds between commits. Each edge draws over this same gap.", "x-advanced" = true))]
    pub step_seconds: f64,
    #[default = "#E8B86D"]
    #[schemars(description = "Accent color", extend("x-help" = "Trunk colour, and the branch ref chip.", "x-check" = {"rule":"colour","message":"Pick a colour"}))]
    pub accent_color: String,
    #[default(vec!["#7DD3E8".into(), "#C99BE8".into(), "#9BD4A0".into(), "#E8B86D".into()])]
    #[schemars(description = "Branch colors", length(min = 1), extend("x-help" = "Colours for branch lanes 1 and up."))]
    pub lane_colors: Vec<Colour>,
    #[default(DarkOrLight::Dark)]
    #[schemars(description = "Theme", extend("x-help" = "Surface palette."))]
    pub theme: DarkOrLight,
    #[default = 1.0]
    #[schemars(description = "Speed", range(min = 0.25, max = 4), extend("x-step" = 0.25, "x-help" = "Animation speed multiplier."))]
    pub speed: f64,
    #[schemars(description = "Background", extend("x-widget" = "color", "x-zero-label" = "Theme", "x-help" = "Leave empty to follow the theme."))]
    pub background_color: String,
}

pub(super) fn elements() -> Vec<ElementMeta> {
    use ElementCategory as C;
    vec![
        meta::<CodeReveal>(
            element(
                ElementKind::Component,
                "remotion-ui/code-reveal",
                "Code reveal",
                "A code editor that types out a file, then highlights the lines that matter.",
                C::Ui,
                165,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/code-reveal", "MIT")
                .look(NodeLook::of("lucide:code")),
        ),
        meta::<CodeDiffWipe>(
            element(
                ElementKind::Component,
                "remotion-ui/code-diff-wipe",
                "Code diff",
                "A code editor that applies a change line by line: removed lines collapse and new lines type in.",
                C::Ui,
                135,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/code-diff-wipe", "MIT")
                .look(NodeLook::of("lucide:file-diff")),
        ),
        meta::<KanbanMove>(
            element(
                ElementKind::Component,
                "remotion-ui/kanban-move",
                "Kanban board",
                "A task board where cards move from one column to the next.",
                C::Ui,
                120,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/kanban-move", "MIT")
                .look(NodeLook::of("lucide:square-kanban")),
        ),
        meta::<FileTreeReveal>(
            element(
                ElementKind::Component,
                "remotion-ui/file-tree-reveal",
                "File tree",
                "A folder tree that opens one item at a time and highlights a file.",
                C::Ui,
                120,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/file-tree-reveal", "MIT")
                .look(NodeLook::of("lucide:folder-tree")),
        ),
        meta::<DragDropFlow>(
            element(
                ElementKind::Component,
                "remotion-ui/drag-drop-flow",
                "File upload",
                "A pointer drags a file into an upload area, which catches it and uploads it.",
                C::Ui,
                150,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/drag-drop-flow", "MIT")
                .look(NodeLook::of("lucide:cloud-upload")),
        ),
        meta::<CommitGraph>(
            element(
                ElementKind::Component,
                "remotion-ui/commit-graph",
                "Commit graph",
                "A version history graph that draws its branches and merges commit by commit.",
                C::Ui,
                120,
            )
                .source("RemotionUI", "https://remotionui.com/docs/components/commit-graph", "MIT")
                .look(NodeLook::of("lucide:git-graph").moves(NodeMotion::Draw)),
        ),
    ]
}
