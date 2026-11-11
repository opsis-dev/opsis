#![allow(
    dead_code,
    reason = "each test binary uses part of the shared fake host"
)]

use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use opsis_compile::node::Platform;
use opsis_document::Document;
use opsis_script::host::{HostResult, ProjectInfo, RunEvents, RunNext, ScriptHost};
use opsis_script::{Ending, Script, run};
use opsis_types::edits::{Edited, TimelineEdit, TimelineReply, WorkflowEdit};
use opsis_types::events::{RunEvent, RunTargets};
use opsis_types::script::ScriptLineKind;
use opsis_types::shell::PlaybackCall;
use opsis_types::workflow::Workflow;
use serde_json::{Value, json};

pub const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/todomvc/project.opsis.json"
);

#[derive(Debug, Clone, PartialEq)]
pub enum Asked {
    Edit(WorkflowEdit),
    Timeline(TimelineEdit),
    Run(RunTargets),
    Playback(PlaybackCall),
    Select(Vec<String>),
    StopRun,
    ClearResults,
    ClearLog,
    Save,
    UseFont(String),
}

pub struct FakeHost {
    pub document: RefCell<Document>,
    pub asked: RefCell<Vec<Asked>>,
    pub events: RefCell<Vec<RunEvent>>,
    pub running: std::cell::Cell<bool>,
}

impl FakeHost {
    pub fn new() -> Rc<Self> {
        let text = std::fs::read_to_string(EXAMPLE).unwrap();
        let workflow: Workflow = serde_json::from_str(&text).unwrap();
        Rc::new(Self {
            document: RefCell::new(Document::open(
                Path::new("/project"),
                workflow,
                "v0".into(),
                Platform::Linux,
            )),
            asked: RefCell::default(),
            events: RefCell::default(),
            running: std::cell::Cell::new(false),
        })
    }

    fn note(&self, asked: Asked) {
        self.asked.borrow_mut().push(asked);
    }
}

struct Queued(VecDeque<RunEvent>);

impl RunEvents for Queued {
    fn next(&mut self, _wait: Duration) -> RunNext {
        self.0
            .pop_front()
            .map_or(RunNext::Idle, |event| RunNext::Event(Box::new(event)))
    }
}

impl ScriptHost for FakeHost {
    fn version(&self) -> String {
        "9.9.9".into()
    }

    fn project(&self) -> HostResult<ProjectInfo> {
        let doc = self.document.borrow();
        Ok(ProjectInfo {
            name: doc.workflow().name.clone(),
            dir: "/project".into(),
            revision: doc.revision(),
            saved: !doc.is_dirty(),
            settings: doc.workflow().settings.clone(),
        })
    }

    fn workflow(&self) -> HostResult<Workflow> {
        Ok(self.document.borrow().workflow().clone())
    }

    fn edit(&self, edit: WorkflowEdit) -> HostResult<Edited> {
        self.note(Asked::Edit(edit.clone()));
        self.document
            .borrow_mut()
            .apply(edit)
            .map_err(|e| e.to_string())
    }

    fn edit_timeline(&self, edit: TimelineEdit) -> HostResult<TimelineReply> {
        self.note(Asked::Timeline(edit));
        Ok(TimelineReply::default())
    }

    fn save(&self) -> HostResult<bool> {
        self.note(Asked::Save);
        Ok(true)
    }

    fn problems(&self) -> HostResult<Value> {
        Ok(json!([]))
    }

    fn catalog(&self) -> Value {
        json!([{ "type": "browser.open", "title": "Open page" }])
    }

    fn compile(&self) -> HostResult<Value> {
        Ok(
            json!({ "issues": [], "outdated": [], "flat": { "fps": 30 }, "levelled": null, "mix": {}, "plan": null }),
        )
    }

    fn validate(&self, workflow: &Value) -> Value {
        let checked = opsis_compile::check::check_workflow(workflow);
        json!({ "ok": checked.ok, "problems": checked.problems })
    }

    fn fonts(&self) -> HostResult<Value> {
        Ok(json!([{ "id": "inter", "family": "Inter" }]))
    }

    fn use_font(&self, id: &str) -> HostResult<String> {
        self.note(Asked::UseFont(id.to_owned()));
        Ok("Inter".into())
    }

    fn start_run(&self, targets: RunTargets) -> HostResult<(String, Box<dyn RunEvents>)> {
        self.note(Asked::Run(targets));
        Ok((
            "r1".into(),
            Box::new(Queued(self.events.borrow().iter().cloned().collect())),
        ))
    }

    fn stop_run(&self) {
        self.note(Asked::StopRun);
    }

    fn active_run(&self) -> Option<String> {
        self.running.get().then(|| "r1".to_owned())
    }

    fn run_state(&self) -> Value {
        json!({ "view": {}, "logs": [] })
    }

    fn results(&self) -> Value {
        json!({ "render": { "artifacts": [{ "kind": "video", "path": ".opsis/out/film.mp4" }] } })
    }

    fn clear_results(&self) -> HostResult<()> {
        self.note(Asked::ClearResults);
        Ok(())
    }

    fn clear_log(&self) {
        self.note(Asked::ClearLog);
    }

    fn playback(&self, call: PlaybackCall) -> HostResult<()> {
        self.note(Asked::Playback(call));
        Ok(())
    }

    fn shared(&self) -> Value {
        json!({ "selection": { "selectedIds": ["a"], "activeIds": [], "pinnedSource": null }, "playhead": null })
    }

    fn select(&self, ids: Vec<String>) -> HostResult<()> {
        self.note(Asked::Select(ids));
        Ok(())
    }
}

pub struct Ran {
    pub ending: Ending,
    pub lines: Vec<(ScriptLineKind, String)>,
}

pub fn run_with(root: &Path, host: Rc<FakeHost>, source: &str, stop: Arc<AtomicBool>) -> Ran {
    let lines = Rc::new(RefCell::new(Vec::new()));
    let kept = Rc::clone(&lines);
    let ending = run(
        &Script {
            file: "scripts/test.lua".into(),
            source: source.into(),
        },
        root,
        host,
        Rc::new(move |kind, text: &str| kept.borrow_mut().push((kind, text.to_owned()))),
        stop,
    );
    let lines = lines.borrow().clone();
    Ran { ending, lines }
}

pub fn run_in(root: &Path, source: &str) -> Ran {
    run_with(
        root,
        FakeHost::new(),
        source,
        Arc::new(AtomicBool::new(false)),
    )
}
