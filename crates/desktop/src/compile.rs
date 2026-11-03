use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use indexmap::IndexMap;
use opsis_compile::asset_ref::project_refs_in;
use opsis_compile::cards::{CardSources, workflow_cards};
use opsis_compile::catalog::{CatalogPort, EditorCatalog, editor_bundle, node_inputs};
use opsis_compile::compile::{CompileResult, FilmTarget, compile_workflow, picked_film};
use opsis_compile::edits::timeline::Film;
use opsis_compile::graph::{GraphIssue, output_conversions, validate_workflow};
use opsis_compile::inspector::{InspectorContext, inspector_context};
use opsis_compile::mix::levels::{apply_mix_levels, solo_active};
use opsis_compile::mix::plan::{MixPlan, build_mix_plan};
use opsis_compile::node::Platform;
use opsis_compile::timeline::flatten::{FlatTimeline, LocalFrames, flatten_spec};
use opsis_store::secrets::list_secrets;
use opsis_types::cards::CardRow;
use opsis_types::mix::Mix;
use opsis_types::timeline::TimelineSpec;
use opsis_types::values::NodeResult;
use opsis_types::workflow::{PortType, Workflow};
use serde::Serialize;
use serde_json::json;
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::error::CommandResult;
use crate::events::Compiled;
use crate::node_results::{check_media_files, held, refresh};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditorCompile {
    pub seq: u64,
    pub result: CompileResult,
    pub issues: Vec<GraphIssue>,
    pub film: Option<FilmTarget>,
    pub flat: Option<FlatTimeline>,
    pub plan: Option<MixPlan>,
    pub levelled: Option<TimelineSpec>,
    pub cards: IndexMap<String, Vec<CardRow>>,
    pub inputs: IndexMap<String, IndexMap<String, CatalogPort>>,
    pub inspector: Option<InspectorContext>,
    pub local_frames: IndexMap<String, LocalFrames>,
    pub mix: Mix,
    pub solo: bool,
    pub outdated: Vec<String>,
    pub converts: IndexMap<String, IndexMap<String, Vec<PortType>>>,
}

static COMPILES: AtomicU64 = AtomicU64::new(0);

struct CompileInput {
    dir: PathBuf,
    workflow: Workflow,
    results: IndexMap<String, NodeResult>,
    selected: Option<String>,
    inspected: Option<String>,
    gesturing: bool,
}

fn compile_input(state: &AppState) -> Option<CompileInput> {
    let editor = state.editor();
    let doc = editor.document.as_ref()?;
    Some(CompileInput {
        dir: doc.dir().to_path_buf(),
        workflow: doc.workflow().clone(),
        results: editor.results.clone(),
        selected: editor.selected.clone(),
        inspected: editor.inspected.clone(),
        gesturing: doc.gesture_start().is_some(),
    })
}

fn editor_compile(input: &CompileInput) -> EditorCompile {
    let workflow = &input.workflow;
    let seq = COMPILES.fetch_add(1, Ordering::Relaxed) + 1;
    let result = compile_workflow(workflow, Some(&input.results));
    let issues = validate_workflow(workflow, Some(&result));
    let picked = picked_film(workflow, &result, input.selected.as_deref());
    let flat = picked.as_ref().map(|p| flatten_spec(&p.spec));
    let film = picked.as_ref().zip(flat.as_ref());
    let plan = film.map(|(p, f)| build_mix_plan(&p.spec, workflow.mix.as_ref(), f));
    let levelled = film.map(|(p, f)| apply_mix_levels(&p.spec, workflow.mix.as_ref(), f));
    let secrets = list_secrets(&input.dir).unwrap_or_else(|error| {
        log::warn!("couldn't list the saved sign-ins: {error}");
        Vec::new()
    });
    let gone = |file: &str| !input.dir.join(file).exists();
    let cards = workflow_cards(
        workflow,
        &CardSources {
            compiled: &result,
            results: Some(&input.results),
            secrets: Some(&secrets),
            gone: &gone,
        },
    );
    let inspector = input
        .inspected
        .as_deref()
        .and_then(|id| inspector_context(workflow, &result, &input.results, id));
    let local_frames = flat
        .iter()
        .flat_map(|f| f.bars.iter())
        .map(|b| (b.key.clone(), b.local_frames()))
        .collect();
    let mix = workflow.mix.clone().unwrap_or_default();
    let converts = output_conversions(&result);
    EditorCompile {
        converts,
        solo: solo_active(&mix),
        mix,
        seq,
        result,
        issues,
        outdated: Vec::new(),
        film: picked,
        flat,
        plan,
        levelled,
        cards,
        inputs: node_inputs(workflow),
        inspector,
        local_frames,
    }
}

fn refreshed_compile(app: &AppHandle, input: &mut CompileInput) -> EditorCompile {
    let refreshed = if input.gesturing {
        held(app, &input.dir, &input.results)
    } else {
        refresh(app, &input.dir, &input.workflow, &mut input.results)
    };
    let mut compiled = editor_compile(input);
    compiled.outdated = refreshed.outdated;
    compiled.issues.extend(refreshed.problems);
    compiled
}

fn film_of(compiled: &EditorCompile) -> Option<Film> {
    Some(Film {
        flat: compiled.flat.clone()?,
        source: compiled.film.as_ref().map(|p| p.source_node_id.clone()),
    })
}

fn emit(app: &AppHandle, compiled: &EditorCompile) {
    if let Err(error) = serde_json::to_value(compiled)
        .map_err(tauri::Error::from)
        .and_then(|v| Compiled(v).emit(app))
    {
        log::warn!("sending the compile result: {error}");
    }
}

pub fn request(app: &AppHandle) {
    let state = app.state::<AppState>();
    if state.compiles.waiting.swap(true, Ordering::AcqRel) {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let _turn = state.compiles.turn.lock().await;
        state.compiles.waiting.store(false, Ordering::Release);
        let Some(mut input) = compile_input(&state) else {
            return;
        };
        let emitter = app.clone();
        let compiled = tauri::async_runtime::spawn_blocking(move || {
            let compiled = refreshed_compile(&emitter, &mut input);
            emitter.state::<AppState>().editor().film = film_of(&compiled);
            emit(&emitter, &compiled);
            if !input.gesturing {
                check_media_files(
                    &emitter,
                    &input.dir,
                    &input.workflow,
                    &compiled.result,
                    &input.results,
                );
            }
            let refs = project_refs_in(&json!([
                compiled.result.values,
                input.workflow.settings.fonts.families
            ]));
            crate::document::follow_loaded_files(&emitter, input.dir, refs);
        })
        .await;
        if let Err(error) = compiled {
            log::warn!("working out the film: {error}");
        }
    });
}

#[tauri::command]
#[specta::specta]
pub fn set_compile_focus(
    app: AppHandle,
    state: State<'_, AppState>,
    selected: Option<String>,
    inspected: Option<String>,
) {
    {
        let mut editor = state.editor();
        if editor.selected == selected && editor.inspected == inspected {
            return;
        }
        editor.selected = selected;
        editor.inspected = inspected;
    }
    request(&app);
}

#[tauri::command]
#[specta::specta]
pub async fn latest_compile(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<Option<EditorCompile>> {
    let Some(mut input) = compile_input(&state) else {
        return Ok(None);
    };
    Ok(Some(
        tauri::async_runtime::spawn_blocking(move || refreshed_compile(&app, &mut input)).await?,
    ))
}

#[tauri::command]
#[specta::specta]
pub fn editor_catalog() -> EditorCatalog {
    editor_bundle(Platform::current())
}
