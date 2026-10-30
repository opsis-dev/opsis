use opsis_golden::typed;
use serde_json::Value;

pub fn fixture(name: &str) -> Value {
    opsis_golden::read(&format!("compile/{name}"))
}

pub fn template(id: &str) -> Value {
    fixture(&format!("workflows/template-{id}.json"))["workflow"].clone()
}

use indexmap::IndexMap;
use opsis_compile::compile::{compile_workflow, film_of, picked_film};
use opsis_compile::registry::node_def;
use opsis_compile::timeline::flatten::{FlatTimeline, flatten_spec};
use opsis_types::actions::{Action, ActionLog};
use opsis_types::timeline::TimelineSpec;
use opsis_types::values::NodeResult;
use opsis_types::workflow::Workflow;
use serde_json::json;

pub const TITLE: &str = "fx.title";
pub const CLIP: &str = "clip";
pub const SESSION: &str = "session";

#[derive(Default)]
pub struct Graph {
    nodes: Vec<Value>,
    edges: Vec<Value>,
}

pub fn defaults_with(kind: &str, params: Value) -> Value {
    let mut all = node_def(kind)
        .map(|d| Value::Object(d.defaults()))
        .unwrap_or_else(|| json!({}));
    if let (Some(all), Value::Object(given)) = (all.as_object_mut(), params) {
        all.extend(given);
    }
    all
}

impl Graph {
    pub fn node(&mut self, id: &str, kind: &str, params: Value) -> String {
        self.nodes.push(json!({ "id": id, "type": kind, "position": { "x": 0, "y": 0 }, "params": defaults_with(kind, params) }));
        id.to_owned()
    }

    pub fn labelled(&mut self, id: &str, kind: &str, params: Value, label: &str) -> String {
        self.node(id, kind, params);
        self.nodes.last_mut().unwrap()["label"] = json!(label);
        id.to_owned()
    }

    pub fn link_ports(
        &mut self,
        source: &str,
        target: &str,
        source_handle: &str,
        target_handle: &str,
    ) -> &mut Self {
        let id = format!("{source}.{source_handle}->{target}.{target_handle}");
        self.edges.push(json!({ "id": id, "source": source, "sourceHandle": source_handle, "target": target, "targetHandle": target_handle }));
        self
    }

    pub fn link(&mut self, source: &str, target: &str) -> &mut Self {
        self.link_ports(source, target, CLIP, CLIP)
    }

    pub fn chain(&mut self, ids: &[String]) -> &mut Self {
        for pair in ids.windows(2) {
            self.link_ports(&pair[0], &pair[1], SESSION, SESSION);
        }
        self
    }

    pub fn record_take(&mut self, steps: &[(&str, &str, Value)], camera: Value) -> String {
        let mut steps: Vec<(&str, &str, Value)> = steps.to_vec();
        if let Some(page) = steps.iter().position(|s| s.1 == "browser.navigate") {
            let mut at = page + 1;
            while steps.get(at).is_some_and(|s| s.1 == "browser.wait") {
                at += 1;
            }
            steps.insert(at, ("start", "browser.startRecording", json!({})));
        }
        let mut ids: Vec<String> = steps
            .iter()
            .map(|(id, kind, params)| self.node(id, kind, params.clone()))
            .collect();
        let rec = self.node("record", "capture.record", json!({}));
        ids.push(rec.clone());
        self.chain(&ids);
        let cam = self.node("camera", "camera.autoZoom", camera);
        self.link_ports(&rec, &cam, "recording", "recording");
        cam
    }

    pub fn recorded_take(&mut self, extra: &[(&str, &str, Value)]) -> String {
        let mut steps = vec![
            ("session", "browser.open", json!({})),
            (
                "open",
                "browser.navigate",
                json!({ "url": "https://example.com" }),
            ),
        ];
        steps.extend(extra.iter().cloned());
        self.record_take(&steps, json!({ "speed": 1 }))
    }

    pub fn build(&self) -> Workflow {
        serde_json::from_value(json!({ "version": 1, "id": "wf", "name": "Test", "nodes": self.nodes, "edges": self.edges })).unwrap()
    }
}

pub fn film(make: impl FnOnce(&mut Graph) -> String) -> Workflow {
    let mut g = Graph::default();
    let last = make(&mut g);
    g.node("out", "output.render", json!({}));
    g.link(&last, "out");
    g.build()
}

pub fn film_spec(wf: &Workflow) -> TimelineSpec {
    picked_film(wf, &compile_workflow(wf, None), None)
        .expect("a film")
        .spec
}

pub fn export_film(
    wf: &Workflow,
    results: Option<&IndexMap<String, NodeResult>>,
    id: &str,
) -> TimelineSpec {
    film_of(wf, &compile_workflow(wf, results), id).expect("a film")
}

pub fn flat_film(wf: &Workflow) -> FlatTimeline {
    flatten_spec(&film_spec(wf))
}

pub fn make_log(actions: Vec<Action>, duration_ms: f64) -> ActionLog {
    serde_json::from_value(json!({
        "viewport": { "width": 1920, "height": 1080 }, "dpr": 1, "t0": 0, "durationMs": duration_ms,
        "actions": actions, "cursorTrack": [{ "t": 0, "x": 960, "y": 540 }]
    }))
    .unwrap()
}

pub fn node_result(fields: Value) -> NodeResult {
    let mut base =
        json!({ "hash": "h", "outputs": {}, "artifacts": [], "finishedAt": 0, "durationMs": 0 });
    base.as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    serde_json::from_value(base).unwrap()
}

pub fn record_result(log: &ActionLog) -> NodeResult {
    node_result(json!({ "outputs": { "recording": {
        "type": "recording", "src": "project:.opsis/cache/h/recording.mp4", "log": log,
        "durationMs": log.duration_ms, "viewport": log.viewport, "nodeId": "record", "style": []
    } } }))
}

pub fn action(value: Value) -> Action {
    serde_json::from_value(value).unwrap()
}

use opsis_compile::node::{CardInputs, Ctx};
use opsis_compile::vars::Vars;
use opsis_types::cards::CardRow;
use opsis_types::values::{ClipValue, CompileValue, PortValues};
use opsis_types::workflow::WorkflowSettings;

pub fn compile_node_with(
    kind: &str,
    params: Value,
    inputs: Value,
    result: Option<&NodeResult>,
) -> Result<PortValues, String> {
    compile_node_in(kind, params, inputs, result, &Vars::new())
}

pub fn compile_node_in(
    kind: &str,
    params: Value,
    inputs: Value,
    result: Option<&NodeResult>,
    vars: &Vars,
) -> Result<PortValues, String> {
    let def = node_def(kind).unwrap_or_else(|| panic!("no node {kind}"));
    let values = def
        .parse(defaults_with(kind, params).as_object().unwrap(), None)
        .map_err(|e| e.to_string())?;
    let inputs: PortValues = typed(&inputs);
    let wf: Workflow =
        typed(&json!({ "version": 1, "id": "wf", "name": "Test", "nodes": [], "edges": [] }));
    let settings = WorkflowSettings::default();
    let ctx = Ctx {
        node_id: "n",
        kind,
        label: None,
        params: (),
        values: &values,
        inputs: &inputs,
        settings: &settings,
        result,
        vars,
        workflow: &wf,
    };
    Ok(def
        .run_compile(&ctx)
        .map_err(|e| e.to_string())?
        .unwrap_or_default())
}

pub fn compile_node(kind: &str, params: Value, inputs: Value) -> PortValues {
    compile_node_with(kind, params, inputs, None).unwrap_or_else(|e| panic!("{kind}: {e}"))
}

pub fn compile_clip(kind: &str, params: Value, inputs: Value) -> ClipValue {
    match compile_node(kind, params, inputs).shift_remove("clip") {
        Some(CompileValue::Clip(clip)) => clip,
        other => panic!("no clip from {kind}: {other:?}"),
    }
}

pub fn node_card(kind: &str, params: Value, inputs: CardInputs) -> Vec<CardRow> {
    node_card_with(kind, params, inputs, None)
}

pub fn node_card_with(
    kind: &str,
    params: Value,
    inputs: CardInputs,
    result: Option<&NodeResult>,
) -> Vec<CardRow> {
    let def = node_def(kind).unwrap_or_else(|| panic!("no node {kind}"));
    def.card_for(
        defaults_with(kind, params).as_object().unwrap(),
        &Vars::new(),
        &WorkflowSettings::default(),
        result,
        &inputs,
    )
}

pub fn card_texts(kind: &str, params: Value, inputs: CardInputs) -> Vec<String> {
    node_card(kind, params, inputs)
        .into_iter()
        .map(|r| r.text)
        .collect()
}

pub fn parse_params(kind: &str, params: Value) -> Result<serde_json::Map<String, Value>, String> {
    node_def(kind)
        .unwrap()
        .parse(defaults_with(kind, params).as_object().unwrap(), None)
        .map_err(|e| e.to_string())
}

pub fn clip_value(clip: &ClipValue) -> Value {
    serde_json::to_value(CompileValue::Clip(clip.clone())).unwrap()
}

pub fn take_recording(log: Option<ActionLog>) -> Value {
    let log = log.unwrap_or_else(|| make_log(Vec::new(), 3000.0));
    json!({ "type": "recording", "src": "project:rec.mp4", "log": log, "durationMs": log.duration_ms, "viewport": log.viewport, "nodeId": "rec", "style": [] })
}

pub fn style_entries(steps: &[(&str, Value)], prefix: &str) -> Value {
    let entries: Vec<Value> = steps
        .iter()
        .enumerate()
        .map(|(i, (kind, params))| json!({ "nodeId": format!("{prefix}{i}"), "type": kind, "params": parse_params(kind, params.clone()).unwrap() }))
        .collect();
    Value::Array(entries)
}

pub fn zoomed(
    style: Option<&[(&str, Value)]>,
    own: &[(&str, Value)],
    log: Option<ActionLog>,
    camera: Value,
) -> ClipValue {
    let mut recording = take_recording(log);
    recording["style"] = style_entries(own, "t");
    let mut inputs = json!({ "recording": recording });
    if let Some(style) = style {
        inputs["style"] =
            json!({ "type": "recording-style", "entries": style_entries(style, "s") });
    }
    compile_clip("camera.autoZoom", camera, inputs)
}

pub fn near(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() <= tolerance
}

pub fn problem(kind: &str, params: Value) -> Option<String> {
    parse_params(kind, params).err()
}

pub fn group_value(clip: &ClipValue) -> Value {
    match &clip.tracks[0].items[0] {
        opsis_types::timeline::TimelineItem::Group { props, .. } => {
            serde_json::to_value(props).unwrap()
        }
        other => panic!("{other:?}"),
    }
}

pub fn todo_mvc() -> Workflow {
    serde_json::from_value(template("todomvc")).unwrap()
}
