use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use indexmap::IndexMap;
use opsis_compile::cards::{CardSources, workflow_cards};
use opsis_compile::catalog::{EditorCatalog, editor_bundle};
use opsis_compile::compile::{CompileResult, PreviewTarget, compile_workflow, preview_spec};
use opsis_compile::graph::{GraphIssue, validate_workflow};
use opsis_compile::inspector::{InspectorContext, inspector_context};
use opsis_compile::mix::levels::apply_mix_levels;
use opsis_compile::mix::plan::{MixPlan, build_mix_plan};
use opsis_compile::node::Platform;
use opsis_compile::timeline::flatten::{BarClock, FlatTimeline, flatten_spec};
use opsis_store::secrets::list_secrets;
use opsis_types::cards::CardRow;
use opsis_types::timeline::TimelineSpec;
use opsis_types::workflow::Workflow;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::error::CommandResult;
use crate::events::Compiled;
use crate::state::{AppState, Editor};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditorCompile {
    pub seq: u64,
    pub result: CompileResult,
    pub issues: Vec<GraphIssue>,
    pub preview: Option<PreviewTarget>,
    pub flat: Option<FlatTimeline>,
    pub plan: Option<MixPlan>,
    pub levelled: Option<TimelineSpec>,
    pub cards: IndexMap<String, Vec<CardRow>>,
    pub inspector: Option<InspectorContext>,
    pub clocks: IndexMap<String, BarClock>,
}

static COMPILES: AtomicU64 = AtomicU64::new(0);

fn editor_compile(editor: &Editor, workflow: &Workflow) -> EditorCompile {
    let seq = COMPILES.fetch_add(1, Ordering::Relaxed) + 1;
    let result = compile_workflow(workflow, Some(&editor.results));
    let issues = validate_workflow(workflow, Some(&result));
    let preview = preview_spec(&workflow.settings, &result, editor.selected.as_deref());
    let flat = preview.as_ref().map(|p| flatten_spec(&p.spec));
    let film = preview.as_ref().zip(flat.as_ref());
    let plan = film.map(|(p, f)| build_mix_plan(&p.spec, workflow.mix.as_ref(), f));
    let levelled = film.map(|(p, f)| apply_mix_levels(&p.spec, workflow.mix.as_ref(), f));
    let secrets = editor
        .dir
        .as_deref()
        .map(|d| list_secrets(d).unwrap_or_default());
    let dir = editor.dir.clone();
    let gone = move |file: &str| {
        dir.as_deref()
            .is_some_and(|d: &Path| !d.join(file).exists())
    };
    let cards = workflow_cards(
        workflow,
        &CardSources {
            compiled: &result,
            results: Some(&editor.results),
            secrets: secrets.as_deref(),
            gone: &gone,
        },
    );
    let inspector = editor
        .inspected
        .as_deref()
        .and_then(|id| inspector_context(workflow, &result, &editor.results, id));
    let clocks = flat
        .iter()
        .flat_map(|f| f.bars.iter())
        .map(|b| (b.key.clone(), b.clock()))
        .collect();
    EditorCompile {
        seq,
        result,
        issues,
        preview,
        flat,
        plan,
        levelled,
        cards,
        inspector,
        clocks,
    }
}

pub fn recompile(app: &AppHandle) {
    let state = app.state::<AppState>();
    let compiled = {
        let editor = state.editor();
        let Some(workflow) = editor.workflow.as_ref() else {
            return;
        };
        editor_compile(&editor, workflow)
    };
    if let Err(error) = serde_json::to_value(&compiled)
        .map_err(tauri::Error::from)
        .and_then(|v| Compiled(v).emit(app))
    {
        log::warn!("sending the compile result: {error}");
    }
}

#[tauri::command]
#[specta::specta]
pub async fn compile(
    app: AppHandle,
    state: State<'_, AppState>,
    workflow: Workflow,
    selected: Option<String>,
    inspected: Option<String>,
) -> CommandResult<EditorCompile> {
    let compiled = {
        let mut editor = state.editor();
        editor.selected = selected;
        editor.inspected = inspected;
        let compiled = editor_compile(&editor, &workflow);
        editor.workflow = Some(workflow);
        compiled
    };
    if let Err(error) = serde_json::to_value(&compiled)
        .map_err(tauri::Error::from)
        .and_then(|v| Compiled(v).emit(&app))
    {
        log::warn!("sending the compile result: {error}");
    }
    Ok(compiled)
}

#[tauri::command]
#[specta::specta]
pub fn send_compiled(app: AppHandle) {
    recompile(&app);
}

#[tauri::command]
#[specta::specta]
pub fn editor_catalog() -> EditorCatalog {
    let platform = if cfg!(target_os = "macos") {
        Platform::Mac
    } else if cfg!(windows) {
        Platform::Windows
    } else {
        Platform::Linux
    };
    editor_bundle(platform)
}
