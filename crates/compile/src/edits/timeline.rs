use std::collections::{HashSet, VecDeque};

use indexmap::IndexMap;
use opsis_types::edits::{EdgeEnds, Removal, TimelineEdit, TimelineReply};
use opsis_types::numbers::whole_signed;
use opsis_types::timeline::{CameraPath, ParamRef, ParamUnit, Sign};

use opsis_types::workflow::{PortType, Position, Workflow, WorkflowEdge, WorkflowNode};
use serde_json::{Map, Value, json};

use crate::edits::workflow::{absolute_position, remove_nodes};
use crate::graph::upstream_of;
use crate::ids::{Ids, node_id_prefix};
use crate::limits::{MAX_SPEED, MAX_VOLUME, MIN_SPEED};
use crate::nodes::timeline::Cut;
use crate::nodes::transitions::NO_TRANSITION;
use crate::ports::{IN_PORTS, in_port_index};
use crate::registry::{node_def, node_name};
use crate::time::{frames_to_ms, ms_to_exact_frames, whole_frames};
use crate::timeline::camera_path::sorted_camera_keys;
use crate::timeline::flatten::{AudioBar, CameraKeyMark, ChainLink, FlatTimeline, VideoBar};

pub const HALF_FRAME: f64 = 0.5;
const CLIP_PORT: &str = "clip";
const SEQUENCE_TYPE: &str = "timeline.sequence";
const DELAY_TYPE: &str = "timeline.delay";
const TRIM_TYPE: &str = "timeline.trim";
const SPEED_TYPE: &str = "timeline.speed";

const NEEDS_A_FRAME: &str = "A clip needs at least one frame.";
const PLAYHEAD_OUTSIDE_CLIP: &str = "Put the playhead inside the clip.";
const AT_START: &str = "That's the start of the clip.";
const TOO_EARLY: &str = "It can't start any earlier.";
const NEW_NODE_OFFSET: Position = Position { x: 60.0, y: 170.0 };
const NEW_NODE_STEP: Position = Position { x: 30.0, y: 150.0 };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleKind {
    Move,
    TrimStart,
    TrimEnd,
    Speed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Done {
    pub message: Option<String>,
}

pub type Outcome = Result<Done, String>;

fn done() -> Outcome {
    Ok(Done::default())
}

fn said(message: Option<String>) -> Outcome {
    Ok(Done { message })
}

fn not_connected(doing: &str) -> String {
    format!("Connect this clip to an export or a sequence to {doing} here.")
}

pub struct TimelineEdits<'a> {
    pub workflow: Workflow,
    pub fps: u32,
    pub source: Option<String>,
    pub ids: &'a mut dyn Ids,
}

fn no_move(delta: f64) -> bool {
    delta.abs() < HALF_FRAME
}

fn handle_of(link: &ChainLink, kind: HandleKind) -> Option<&ParamRef> {
    let handles = link.handles.as_ref()?;
    match kind {
        HandleKind::Move => handles.move_to.as_ref(),
        HandleKind::TrimStart => handles.trim_start.as_ref(),
        HandleKind::TrimEnd => handles.trim_end.as_ref(),
        HandleKind::Speed => handles.speed.as_ref(),
    }
}

pub fn slot_index(bar: &VideoBar) -> Option<usize> {
    bar.chain.iter().position(|l| l.slot.is_some())
}

pub fn slot_link(bar: &VideoBar) -> Option<&ChainLink> {
    slot_index(bar).map(|i| &bar.chain[i])
}

pub fn outer_handle(bar: &VideoBar, kind: HandleKind) -> Option<&ChainLink> {
    let end = slot_index(bar).map_or(bar.chain.len(), |i| i + 1);
    bar.chain[..end]
        .iter()
        .rev()
        .find(|l| handle_of(l, kind).is_some())
}

pub fn container_of(bar: &VideoBar) -> Option<&str> {
    slot_link(bar)
        .and_then(|l| l.slot.as_ref())
        .map(|s| s.node_id.as_str())
}

fn param_value(node: &WorkflowNode, param: &str) -> Option<Value> {
    node.params
        .get(param)
        .cloned()
        .or_else(|| node_def(&node.kind).and_then(|d| d.defaults().get(param).cloned()))
}

fn parse_number(text: &str) -> Option<f64> {
    let normal = text.trim().replacen(',', ".", 1);
    if normal.is_empty() {
        return None;
    }
    normal.parse::<f64>().ok().filter(|n| n.is_finite())
}

fn number_list(node: &WorkflowNode, param: &str) -> Vec<f64> {
    let text = param_value(node, param);
    text.as_ref()
        .and_then(Value::as_str)
        .unwrap_or_default()
        .split(',')
        .map(|part| parse_number(part).unwrap_or(0.0))
        .collect()
}

fn format_number_list(parts: &[f64]) -> String {
    let mut trimmed = parts.to_vec();
    while trimmed.len() > 1 && trimmed.last() == Some(&0.0) {
        trimmed.pop();
    }
    if trimmed.iter().all(|p| *p == 0.0) {
        return String::new();
    }
    trimmed
        .iter()
        .map(|p| json!(p).to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn number(value: f64) -> Value {
    if value.fract() == 0.0 && value.abs() <= f64::from(i32::MAX) {
        json!(whole_signed(value))
    } else {
        json!(value)
    }
}

#[derive(Debug, Clone)]
struct ParamAt {
    node_id: String,
    param: String,
    index: Option<u32>,
}

impl From<&ParamRef> for ParamAt {
    fn from(r: &ParamRef) -> Self {
        Self {
            node_id: r.node_id.clone(),
            param: r.param.clone(),
            index: r.index,
        }
    }
}

fn read_param(wf: &Workflow, at: &ParamAt) -> f64 {
    let Some(node) = wf.node(&at.node_id) else {
        return 0.0;
    };
    match at.index {
        None => param_value(node, &at.param)
            .as_ref()
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
        Some(index) => number_list(node, &at.param)
            .get(index as usize)
            .copied()
            .unwrap_or(0.0),
    }
}

fn write_param(wf: &mut Workflow, at: &ParamAt, value: f64) {
    let Some(node) = wf.nodes.iter_mut().find(|n| n.id == at.node_id) else {
        return;
    };
    let written = match at.index {
        None => number(value),
        Some(index) => {
            let mut parts = number_list(node, &at.param);
            let index = index as usize;
            if parts.len() <= index {
                parts.resize(index + 1, 0.0);
            }
            parts[index] = value;
            Value::String(format_number_list(&parts))
        }
    };
    node.params.insert(at.param.clone(), written);
}

fn sign_of(sign: Sign) -> f64 {
    match sign {
        Sign::Same => 1.0,
        Sign::Opposite => -1.0,
    }
}

fn shift_param(wf: &mut Workflow, r: &ParamRef, delta_content: f64, fps: u32) -> bool {
    let scale = r.scale.unwrap_or(1.0);
    let in_ms = r.unit == ParamUnit::Ms;
    let shift = |wf: &Workflow, at: &ParamAt, sign: f64| {
        let current = read_param(wf, at);
        let frames = if in_ms {
            ms_to_exact_frames(current, fps).round().max(0.0)
        } else {
            current
        };
        let next = frames + sign * delta_content * scale;
        let clamped = next.round().max(0.0);
        let value = if in_ms {
            frames_to_ms(clamped, fps)
        } else {
            clamped
        };
        (value, next < -HALF_FRAME)
    };
    let main = ParamAt::from(r);
    let (value, mut clamped) = shift(wf, &main, 1.0);
    let base = wf.clone();
    write_param(wf, &main, value);
    for link in r.linked.iter().flatten() {
        let at = ParamAt {
            node_id: r.node_id.clone(),
            param: link.param.clone(),
            index: None,
        };
        if link.if_non_zero == Some(true) && read_param(&base, &at) == 0.0 {
            continue;
        }
        let (value, hit) = shift(&base, &at, sign_of(link.sign));
        write_param(wf, &at, value);
        clamped |= hit;
    }
    clamped
}

fn shift_to(wf: &mut Workflow, r: &ParamRef, delta: f64, fps: u32, at_limit: &str) -> Outcome {
    let clamped = shift_param(wf, r, delta, fps);
    said(clamped.then(|| at_limit.to_owned()))
}

fn set_length(wf: &mut Workflow, r: &ParamRef, length_content: f64, fps: u32) {
    let frames = f64::from(whole_frames(length_content * r.scale.unwrap_or(1.0), 1));
    let value = if r.unit == ParamUnit::Ms {
        frames_to_ms(frames, fps)
    } else {
        frames
    };
    write_param(wf, &ParamAt::from(r), value);
}

fn clip_edge(source: &str, source_handle: &str, target: &str, target_handle: &str) -> WorkflowEdge {
    EdgeEnds {
        source: source.to_owned(),
        source_handle: source_handle.to_owned(),
        target: target.to_owned(),
        target_handle: target_handle.to_owned(),
    }
    .edge()
}

fn place_near(
    wf: &Workflow,
    source: Option<&WorkflowNode>,
    target: Option<&WorkflowNode>,
    index: f64,
) -> (Position, Option<String>) {
    let Some(anchor) = source.or(target) else {
        return (Position::default(), None);
    };
    let at = absolute_position(wf, anchor);
    let pos = Position {
        x: at.x + NEW_NODE_OFFSET.x + index * NEW_NODE_STEP.x,
        y: at.y + NEW_NODE_OFFSET.y + index * NEW_NODE_STEP.y,
    };
    let shared = source
        .and_then(|s| s.parent_id.as_deref())
        .filter(|p| target.and_then(|t| t.parent_id.as_deref()) == Some(*p));
    let Some(parent) = shared.and_then(|p| wf.node(p)) else {
        return (pos, None);
    };
    let p = absolute_position(wf, parent);
    (
        Position {
            x: pos.x - p.x,
            y: pos.y - p.y,
        },
        Some(parent.id.clone()),
    )
}

#[derive(Clone, Copy)]
enum Added {
    Delay,
    Trim,
    Speed,
}

impl Added {
    fn kind(self) -> &'static str {
        match self {
            Self::Delay => DELAY_TYPE,
            Self::Trim => TRIM_TYPE,
            Self::Speed => SPEED_TYPE,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Delay => "Delay from timeline",
            Self::Trim => "Trim from timeline",
            Self::Speed => "Speed from timeline",
        }
    }

    fn place(self) -> &'static str {
        match self {
            Self::Delay => "before",
            Self::Trim | Self::Speed => "after",
        }
    }
}

fn values(pairs: &[(&str, Value)]) -> Map<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), v.clone()))
        .collect()
}

fn ms_value(frames: f64, fps: u32) -> Value {
    number(frames_to_ms(frames, fps))
}

impl TimelineEdits<'_> {
    fn clip_name(&self, id: &str) -> String {
        self.workflow
            .node(id)
            .map_or_else(|| "this clip".to_owned(), node_name)
    }

    fn new_node(
        &mut self,
        kind: &str,
        params: Map<String, Value>,
        label: &str,
        near: (&str, &str),
        index: f64,
    ) -> WorkflowNode {
        let (position, parent_id) = place_near(
            &self.workflow,
            self.workflow.node(near.0),
            self.workflow.node(near.1),
            index,
        );
        let mut all = node_def(kind).map(|d| d.defaults()).unwrap_or_default();
        all.extend(params);
        WorkflowNode {
            id: self.ids.id(node_id_prefix(kind)),
            kind: kind.to_owned(),
            position,
            params: all,
            label: Some(label.to_owned()),
            disabled: None,
            parent_id,
            size: None,
        }
    }

    fn insert_on_edge(&mut self, edge: &WorkflowEdge, added: Added, params: Map<String, Value>) {
        let node = self.new_node(
            added.kind(),
            params,
            added.label(),
            (&edge.source, &edge.target),
            0.0,
        );
        let wf = &mut self.workflow;
        let id = node.id.clone();
        wf.nodes.push(node);
        wf.edges.retain(|e| e.id != edge.id);
        wf.edges
            .push(clip_edge(&edge.source, &edge.source_handle, &id, CLIP_PORT));
        wf.edges
            .push(clip_edge(&id, CLIP_PORT, &edge.target, &edge.target_handle));
    }

    fn root_edge(&self) -> Option<WorkflowEdge> {
        let source = self.workflow.node(self.source.as_deref()?)?;
        let def = node_def(&source.kind)?;
        self.workflow
            .edges
            .iter()
            .find(|e| {
                e.target == source.id
                    && def
                        .input(&e.target_handle)
                        .is_some_and(|p| p.kind == PortType::Clip)
            })
            .cloned()
    }

    fn slot_edge(&self, bar: &VideoBar) -> Option<WorkflowEdge> {
        let Some(slot) = slot_link(bar).and_then(|l| l.slot.as_ref()) else {
            return self.root_edge();
        };
        self.workflow
            .edges
            .iter()
            .find(|e| e.target == slot.node_id && e.target_handle == slot.port)
            .cloned()
    }

    fn node_after_clip(&self, bar: &VideoBar) -> Option<String> {
        bar.chain
            .iter()
            .skip(1)
            .find_map(|l| l.node_id.clone().filter(|id| *id != bar.node_id))
            .or_else(|| {
                slot_link(bar)
                    .and_then(|l| l.slot.as_ref())
                    .map(|s| s.node_id.clone())
            })
            .or_else(|| self.source.clone())
    }

    fn edge_out_of_clip(&self, bar: &VideoBar) -> Option<WorkflowEdge> {
        let next = self.node_after_clip(bar)?;
        let slot = slot_link(bar).and_then(|l| l.slot.as_ref());
        if let Some(slot) = slot.filter(|s| s.node_id == next)
            && let Some(edge) = self.workflow.edges.iter().find(|e| {
                e.source == bar.node_id && e.target == slot.node_id && e.target_handle == slot.port
            })
        {
            return Some(edge.clone());
        }
        first_edge_on_path(&self.workflow, &bar.node_id, &next)
    }

    fn owned<'b>(&self, bar: &'b VideoBar, kind: HandleKind) -> Option<(&'b ChainLink, ParamRef)> {
        let link = outer_handle(bar, kind)?;
        let r = handle_of(link, kind)?;
        self.workflow.node(&r.node_id).map(|_| (link, r.clone()))
    }

    fn shift_owned(
        &mut self,
        link: &ChainLink,
        r: &ParamRef,
        delta: f64,
        at_limit: &str,
    ) -> Outcome {
        let fps = self.fps;
        shift_to(
            &mut self.workflow,
            r,
            (delta * link.rate).round(),
            fps,
            at_limit,
        )
    }

    fn add(
        &mut self,
        bar: &VideoBar,
        edge: &WorkflowEdge,
        added: Added,
        params: Map<String, Value>,
    ) -> Outcome {
        let name = self.clip_name(&bar.node_id);
        self.insert_on_edge(edge, added, params);
        Ok(Done {
            message: Some(format!(
                "Added “{}” {} {name}.",
                added.label(),
                added.place()
            )),
        })
    }

    fn add_trim(
        &mut self,
        bar: &VideoBar,
        edge: &WorkflowEdge,
        start: f64,
        length: f64,
    ) -> Outcome {
        let fps = self.fps;
        self.add(
            bar,
            edge,
            Added::Trim,
            values(&[
                ("startMs", ms_value(start, fps)),
                ("durationMs", ms_value(length, fps)),
            ]),
        )
    }

    pub fn move_bar(&mut self, bar: &VideoBar, delta: f64) -> Outcome {
        if no_move(delta) {
            return done();
        }
        if let Some((link, r)) = self.owned(bar, HandleKind::Move) {
            return self.shift_owned(link, &r, delta, TOO_EARLY);
        }
        if delta < 0.0 {
            let in_sequence = container_of(bar)
                .and_then(|id| self.workflow.node(id))
                .is_some_and(|n| n.kind == SEQUENCE_TYPE);
            return Err(if in_sequence {
                "In a sequence a clip starts when the one before it ends. Shorten that one instead."
            } else {
                "It already starts at the beginning."
            }
            .to_owned());
        }
        let edge = self
            .slot_edge(bar)
            .ok_or_else(|| not_connected("move it"))?;
        let rate = slot_link(bar).map_or(1.0, |l| l.rate);
        let ms = ms_value((delta * rate).round(), self.fps);
        self.add(bar, &edge, Added::Delay, values(&[("ms", ms)]))
    }

    fn trim_start(&mut self, bar: &VideoBar, delta: f64) -> Outcome {
        if let Some((link, r)) = self.owned(bar, HandleKind::TrimStart) {
            return self.shift_owned(link, &r, delta, AT_START);
        }
        if delta < 0.0 {
            return Err(AT_START.to_owned());
        }
        let edge = self
            .edge_out_of_clip(bar)
            .ok_or_else(|| not_connected("trim it"))?;
        self.add_trim(
            bar,
            &edge,
            ((bar.content_start + delta) * bar.rate).round(),
            0.0,
        )
    }

    fn trim_end(&mut self, bar: &VideoBar, delta: f64) -> Outcome {
        if let Some((link, r)) = self.owned(bar, HandleKind::TrimEnd) {
            let length = link.end + delta * link.rate_end;
            if length < 1.0 {
                return Err(NEEDS_A_FRAME.to_owned());
            }
            set_length(&mut self.workflow, &r, length, self.fps);
            return done();
        }
        if delta > 0.0 {
            return Err("That's the end of the clip.".to_owned());
        }
        let edge = self
            .edge_out_of_clip(bar)
            .ok_or_else(|| not_connected("trim it"))?;
        let keep = (bar.content_end - bar.content_start + delta * bar.rate).round();
        if keep < 1.0 {
            return Err(NEEDS_A_FRAME.to_owned());
        }
        self.add_trim(bar, &edge, bar.content_start.round(), keep)
    }

    pub fn trim_bar(&mut self, bar: &VideoBar, at_start: bool, delta: f64) -> Outcome {
        if no_move(delta) {
            return done();
        }
        if at_start {
            self.trim_start(bar, delta)
        } else {
            self.trim_end(bar, delta)
        }
    }

    pub fn slip_bar(&mut self, bar: &VideoBar, delta: f64) -> Outcome {
        if no_move(delta) {
            return done();
        }
        let content = (delta * bar.rate).round();
        if let Some((link, r)) = self.owned(bar, HandleKind::TrimStart) {
            let length = ((link.to - link.from) * link.rate).round();
            if let Some(length_ref) = handle_of(link, HandleKind::TrimEnd) {
                set_length(&mut self.workflow, length_ref, length, self.fps);
            }
            let alone = ParamRef {
                linked: Some(Vec::new()),
                ..r
            };
            let fps = self.fps;
            return shift_to(&mut self.workflow, &alone, content, fps, AT_START);
        }
        if content < 0.0 {
            return Err(AT_START.to_owned());
        }
        let edge = self
            .edge_out_of_clip(bar)
            .ok_or_else(|| not_connected("edit it"))?;
        let length = (bar.content_end - bar.content_start).round();
        self.add_trim(bar, &edge, bar.content_start.round() + content, length)
    }

    pub fn roll_cut(&mut self, left: &VideoBar, right: &VideoBar, delta: f64) -> Outcome {
        if no_move(delta) {
            return done();
        }
        let a = self.trim_end(left, delta)?;
        let b = self.trim_start(right, delta)?;
        let joined: Vec<String> = [a.message, b.message].into_iter().flatten().collect();
        said((!joined.is_empty()).then(|| joined.join(" ")))
    }

    pub fn speed_bar(&mut self, bar: &VideoBar, new_length: f64) -> Outcome {
        let owned = self.owned(bar, HandleKind::Speed);
        let span = outer_handle(bar, HandleKind::Speed)
            .or_else(|| slot_link(bar))
            .or_else(|| bar.chain.last());
        let current = span.map_or(0.0, |s| s.to - s.from);
        if new_length < 1.0 || current < 1.0 {
            return Err(NEEDS_A_FRAME.to_owned());
        }
        let factor = current / new_length;
        if let Some((_, r)) = owned {
            let at = ParamAt::from(&r);
            let rate = match read_param(&self.workflow, &at) {
                0.0 => 1.0,
                rate => rate,
            };
            write_param(&mut self.workflow, &at, clamp_rate(rate * factor));
            return done();
        }
        let edge = self
            .slot_edge(bar)
            .ok_or_else(|| not_connected("change its speed"))?;
        self.add(
            bar,
            &edge,
            Added::Speed,
            values(&[("rate", number(clamp_rate(factor)))]),
        )
    }

    fn split_parts(&mut self, edge: &WorkflowEdge, offset: f64) -> SplitParts {
        let cut = ms_value(offset, self.fps);
        let near = (edge.source.as_str(), edge.target.as_str());
        let head = self.new_node(
            TRIM_TYPE,
            values(&[("startMs", json!(0)), ("durationMs", cut.clone())]),
            "Split: first part",
            near,
            0.0,
        );
        let tail = self.new_node(
            TRIM_TYPE,
            values(&[("startMs", cut), ("durationMs", json!(0))]),
            "Split: second part",
            near,
            1.0,
        );
        let feeds = [&head, &tail]
            .iter()
            .map(|p| clip_edge(&edge.source, &edge.source_handle, &p.id, CLIP_PORT))
            .collect();
        SplitParts { head, tail, feeds }
    }

    fn split_in_sequence(
        &mut self,
        edge: &WorkflowEdge,
        container: &WorkflowNode,
        next: &str,
        parts: SplitParts,
    ) {
        let port = in_port_index(&edge.target_handle).unwrap_or(0);
        let shift_port = |handle: &str| -> Option<String> {
            let index = in_port_index(handle)?;
            if index <= port {
                Some(handle.to_owned())
            } else {
                IN_PORTS.get(index + 1).map(|p| (*p).to_owned())
            }
        };
        let saved = container
            .params
            .get("cuts")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mut cuts: Vec<Value> = saved
            .into_iter()
            .filter_map(|mut cut| {
                let before = shift_port(cut.get("before")?.as_str()?)?;
                cut["before"] = Value::String(before);
                Some(cut)
            })
            .collect();
        let transition = param_value(container, "transition");
        if transition.as_ref().and_then(Value::as_str) != Some(NO_TRANSITION) {
            let mut cut = serde_json::to_value(Cut::default()).unwrap_or_else(|_| json!({}));
            cut["before"] = Value::String(next.to_owned());
            cut["transition"] = Value::String(NO_TRANSITION.to_owned());
            cuts.push(cut);
        }
        let wf = &mut self.workflow;
        let mut edges: Vec<WorkflowEdge> = wf
            .edges
            .iter()
            .filter(|e| e.id != edge.id)
            .filter_map(|e| {
                if e.target != edge.target || in_port_index(&e.target_handle).is_none() {
                    return Some(e.clone());
                }
                let target_handle = shift_port(&e.target_handle)?;
                Some(clip_edge(
                    &e.source,
                    &e.source_handle,
                    &e.target,
                    &target_handle,
                ))
            })
            .collect();
        edges.extend(parts.feeds);
        edges.push(clip_edge(
            &parts.head.id,
            CLIP_PORT,
            &edge.target,
            &edge.target_handle,
        ));
        edges.push(clip_edge(&parts.tail.id, CLIP_PORT, &edge.target, next));
        if let Some(node) = wf.nodes.iter_mut().find(|n| n.id == container.id) {
            node.params.insert("cuts".to_owned(), Value::Array(cuts));
        }
        wf.nodes.push(parts.head);
        wf.nodes.push(parts.tail);
        wf.edges = edges;
    }

    fn split_with_new_sequence(&mut self, edge: &WorkflowEdge, parts: SplitParts) {
        let join = self.new_node(
            SEQUENCE_TYPE,
            values(&[("transition", json!(NO_TRANSITION))]),
            "Split",
            (&edge.source, &edge.target),
            2.0,
        );
        let wf = &mut self.workflow;
        wf.edges.retain(|e| e.id != edge.id);
        wf.edges.extend(parts.feeds);
        wf.edges
            .push(clip_edge(&parts.head.id, CLIP_PORT, &join.id, IN_PORTS[0]));
        wf.edges
            .push(clip_edge(&parts.tail.id, CLIP_PORT, &join.id, IN_PORTS[1]));
        wf.edges.push(clip_edge(
            &join.id,
            CLIP_PORT,
            &edge.target,
            &edge.target_handle,
        ));
        wf.nodes.push(parts.head);
        wf.nodes.push(parts.tail);
        wf.nodes.push(join);
    }

    pub fn split_bar(&mut self, bar: &VideoBar, at: f64) -> Outcome {
        if at <= bar.from + HALF_FRAME || at >= bar.to - HALF_FRAME {
            return Err(PLAYHEAD_OUTSIDE_CLIP.to_owned());
        }
        let edge = self
            .slot_edge(bar)
            .ok_or_else(|| not_connected("split it"))?;
        let (from, rate) = slot_link(bar).map_or((0.0, 1.0), |l| (l.from, l.rate));
        let offset = ((at - from) * rate).round();
        if offset < 1.0 {
            return Err(PLAYHEAD_OUTSIDE_CLIP.to_owned());
        }
        let name = self.clip_name(&bar.node_id);
        let parts = self.split_parts(&edge, offset);
        let container = self.workflow.node(&edge.target).cloned();
        match container
            .as_ref()
            .and_then(|c| free_port_after(&self.workflow, c, &edge))
        {
            Some(next) => {
                let container = container.expect("a container with a free port");
                self.split_in_sequence(&edge, &container, &next, parts);
            }
            None => self.split_with_new_sequence(&edge, parts),
        }
        Ok(Done {
            message: Some(format!("Split {name} in two.")),
        })
    }

    pub fn move_audio(&mut self, bar: &AudioBar, delta: f64) -> Outcome {
        if let Some(video) = &bar.bar {
            return self.move_bar(video, delta);
        }
        if no_move(delta) {
            return done();
        }
        let r = bar
            .handles
            .as_ref()
            .and_then(|h| h.move_to.as_ref())
            .ok_or("This sound can't be moved here.")?;
        let fps = self.fps;
        shift_to(&mut self.workflow, r, delta.round(), fps, TOO_EARLY)
    }

    pub fn trim_audio(&mut self, bar: &AudioBar, at_start: bool, delta: f64) -> Outcome {
        if let Some(video) = &bar.bar {
            return self.trim_bar(video, at_start, delta);
        }
        if no_move(delta) {
            return done();
        }
        let handles = bar.handles.as_ref();
        let fps = self.fps;
        if at_start {
            let r = handles
                .and_then(|h| h.trim_start.as_ref())
                .ok_or("Sound effects play in full. Move it instead.")?;
            return shift_to(
                &mut self.workflow,
                r,
                delta.round(),
                fps,
                "That's the start of the sound.",
            );
        }
        let r = handles
            .and_then(|h| h.trim_end.as_ref())
            .ok_or("Sound effects play in full.")?;
        let length = bar.to - bar.from + delta;
        if length < 1.0 {
            return Err("A sound needs at least one frame.".to_owned());
        }
        set_length(&mut self.workflow, r, length, fps);
        done()
    }

    pub fn set_audio_volume(&mut self, bar: &AudioBar, volume: f64) -> Outcome {
        let v = round2(volume.clamp(0.0, MAX_VOLUME));
        if let Some(video) = &bar.bar {
            let at = ParamAt {
                node_id: video.node_id.clone(),
                param: "volume".to_owned(),
                index: None,
            };
            write_param(&mut self.workflow, &at, v);
            return done();
        }
        let r = bar
            .handles
            .as_ref()
            .and_then(|h| h.volume.as_ref())
            .ok_or("This sound's volume is set in the mixer.")?;
        write_param(&mut self.workflow, &ParamAt::from(r), v);
        done()
    }

    pub fn ripple_targets(
        &self,
        flat: &FlatTimeline,
        after: f64,
        exclude_container: Option<&str>,
        exclude: &HashSet<String>,
    ) -> Vec<RippleTarget> {
        let mut out: IndexMap<String, RippleTarget> = IndexMap::new();
        for bar in &flat.bars {
            if bar.from < after - HALF_FRAME {
                continue;
            }
            if exclude_container.is_some() && container_of(bar) == exclude_container {
                continue;
            }
            let Some((link, r)) = self.owned(bar, HandleKind::Move) else {
                continue;
            };
            let key = handle_key(&r);
            if exclude.contains(&key) {
                continue;
            }
            out.insert(
                key,
                RippleTarget {
                    rate: link.rate,
                    param: r,
                },
            );
        }
        for audio in &flat.audio {
            if audio.bar.is_some() || audio.from < after - HALF_FRAME {
                continue;
            }
            let Some(r) = audio.handles.as_ref().and_then(|h| h.move_to.as_ref()) else {
                continue;
            };
            let key = handle_key(r);
            if exclude.contains(&key) {
                continue;
            }
            out.insert(
                key,
                RippleTarget {
                    rate: 1.0,
                    param: r.clone(),
                },
            );
        }
        out.into_values().collect()
    }

    pub fn apply_ripple(&mut self, targets: &[RippleTarget], delta: f64) {
        for t in targets {
            shift_param(
                &mut self.workflow,
                &t.param,
                (delta * t.rate).round(),
                self.fps,
            );
        }
    }

    pub fn removal_for_video(&self, bar: &VideoBar) -> Removal {
        let end = slot_index(bar).unwrap_or(bar.chain.len());
        let mut ids: Vec<String> = Vec::new();
        for link in &bar.chain[..end] {
            let Some(id) = link.node_id.as_ref() else {
                continue;
            };
            if ids.contains(id) || Some(id) == self.source.as_ref() {
                continue;
            }
            if self
                .workflow
                .edges
                .iter()
                .filter(|e| &e.source == id)
                .count()
                > 1
            {
                break;
            }
            ids.push(id.clone());
        }
        if ids.is_empty() {
            return Removal {
                node_ids: Vec::new(),
                edge_ids: self.slot_edge(bar).map(|e| e.id).into_iter().collect(),
            };
        }
        Removal {
            node_ids: ids,
            edge_ids: Vec::new(),
        }
    }

    pub fn removal_for_audio(&self, bar: &AudioBar) -> Removal {
        match (&bar.bar, &bar.node_id) {
            (Some(video), _) => self.removal_for_video(video),
            (None, node) => Removal {
                node_ids: node.iter().cloned().collect(),
                edge_ids: Vec::new(),
            },
        }
    }

    pub fn move_camera_key(&mut self, mark: &CameraKeyMark, delta: f64) -> Outcome {
        if no_move(delta) {
            return done();
        }
        let unreadable =
            || "This node's camera keys can't be read. Check them in the Inspector.".to_owned();
        let node = self.workflow.node(&mark.node_id).ok_or_else(unreadable)?;
        let path: CameraPath = node
            .params
            .get("camera")
            .cloned()
            .and_then(|v| serde_json::from_value(v).ok())
            .ok_or_else(unreadable)?;
        let mut keys = sorted_camera_keys(Some(&path));
        let index = mark.index as usize;
        let key = keys.get_mut(index).ok_or("That camera key is gone.")?;
        key.frame = (key.frame + delta).round().max(0.0);
        keys.sort_by(|a, b| a.frame.total_cmp(&b.frame));
        let camera = serde_json::to_value(CameraPath { keys }).map_err(|e| e.to_string())?;
        if let Some(node) = self
            .workflow
            .nodes
            .iter_mut()
            .find(|n| n.id == mark.node_id)
        {
            node.params.insert("camera".to_owned(), camera);
        }
        done()
    }
}

struct SplitParts {
    head: WorkflowNode,
    tail: WorkflowNode,
    feeds: Vec<WorkflowEdge>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RippleTarget {
    pub param: ParamRef,
    pub rate: f64,
}

pub fn apply_removal(removal: &Removal, wf: &mut Workflow) {
    remove_nodes(wf, &removal.node_ids);
    wf.edges.retain(|e| !removal.edge_ids.contains(&e.id));
}

pub fn handle_key(r: &ParamRef) -> String {
    format!(
        "{}:{}:{}",
        r.node_id,
        r.param,
        r.index.map(|i| i.to_string()).unwrap_or_default()
    )
}

pub fn move_handle_key(bar: &VideoBar) -> Option<String> {
    outer_handle(bar, HandleKind::Move)
        .and_then(|l| handle_of(l, HandleKind::Move))
        .map(handle_key)
}

pub fn audio_move_handle_key(bar: &AudioBar) -> Option<String> {
    bar.handles
        .as_ref()
        .and_then(|h| h.move_to.as_ref())
        .map(handle_key)
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn clamp_rate(rate: f64) -> f64 {
    round2(rate).clamp(MIN_SPEED, MAX_SPEED)
}

fn first_edge_on_path(wf: &Workflow, from: &str, to: &str) -> Option<WorkflowEdge> {
    let mut seen: HashSet<&str> = HashSet::from([from]);
    let mut queue: VecDeque<(&str, Option<&WorkflowEdge>)> = VecDeque::from([(from, None)]);
    while let Some((id, first)) = queue.pop_front() {
        for e in wf.edges.iter().filter(|e| e.source == id) {
            if seen.contains(e.target.as_str()) {
                continue;
            }
            let head = first.unwrap_or(e);
            if e.target == to {
                return Some(head.clone());
            }
            seen.insert(&e.target);
            queue.push_back((&e.target, Some(head)));
        }
    }
    None
}

fn free_port_after(wf: &Workflow, container: &WorkflowNode, edge: &WorkflowEdge) -> Option<String> {
    if container.kind != SEQUENCE_TYPE {
        return None;
    }
    let index = in_port_index(&edge.target_handle)?;
    let last_used = wf
        .edges
        .iter()
        .filter(|e| e.target == container.id)
        .filter_map(|e| in_port_index(&e.target_handle))
        .max();
    if last_used.is_some_and(|last| last >= IN_PORTS.len() - 1) {
        return None;
    }
    IN_PORTS.get(index + 1).map(|p| (*p).to_owned())
}

const CUT_TOLERANCE_FRAMES: f64 = 1.0;

#[derive(Debug, Clone)]
pub struct Film {
    pub flat: FlatTimeline,
    pub source: Option<String>,
}

fn video_bar<'a>(flat: &'a FlatTimeline, key: &str) -> Result<&'a VideoBar, String> {
    flat.bars
        .iter()
        .find(|b| b.key == key)
        .ok_or_else(|| "That clip is no longer on the timeline.".to_owned())
}

fn audio_bar<'a>(flat: &'a FlatTimeline, key: &str) -> Result<&'a AudioBar, String> {
    flat.audio
        .iter()
        .find(|b| b.key == key)
        .ok_or_else(|| "That sound is no longer on the timeline.".to_owned())
}

fn neighbour<'a>(flat: &'a FlatTimeline, bar: &VideoBar, right: bool) -> Option<&'a VideoBar> {
    let container = container_of(bar);
    flat.bars.iter().find(|b| {
        b.key != bar.key
            && container_of(b) == container
            && (if right {
                b.from - bar.to
            } else {
                b.to - bar.from
            })
            .abs()
                < CUT_TOLERANCE_FRAMES
    })
}

struct RippleScope<'a> {
    after: f64,
    container: Option<&'a str>,
    exclude: Option<String>,
}

fn moved_by_edit(before: &Workflow, after: &Workflow) -> HashSet<String> {
    let mut moved = HashSet::new();
    for node in &after.nodes {
        if before
            .node(&node.id)
            .is_none_or(|was| was.params != node.params)
        {
            moved.insert(node.id.clone());
            moved.extend(upstream_of(after, &node.id));
        }
    }
    moved
}

fn bar_under<'a>(flat: &'a FlatTimeline, at: f64, selected: &[String]) -> Option<&'a VideoBar> {
    let under: Vec<&VideoBar> = flat
        .bars
        .iter()
        .filter(|b| b.from < at && b.to > at)
        .collect();
    under
        .iter()
        .find(|b| selected.contains(&b.node_id))
        .or_else(|| under.iter().max_by_key(|b| b.lane))
        .copied()
}

impl TimelineEdits<'_> {
    fn rippled(
        &mut self,
        flat: &FlatTimeline,
        before: &Workflow,
        outcome: Outcome,
        scope: RippleScope<'_>,
        delta: f64,
    ) -> Outcome {
        let done = outcome?;
        let exclude: HashSet<String> = scope.exclude.into_iter().collect();
        let moved = moved_by_edit(before, &self.workflow);
        let targets: Vec<RippleTarget> = self
            .ripple_targets(flat, scope.after, scope.container, &exclude)
            .into_iter()
            .filter(|t| !moved.contains(&t.param.node_id))
            .collect();
        self.apply_ripple(&targets, delta);
        Ok(done)
    }

    pub fn apply(&mut self, edit: TimelineEdit, flat: &FlatTimeline) -> Outcome {
        match edit {
            TimelineEdit::Move { bar, delta, ripple } => {
                let bar = video_bar(flat, &bar)?;
                let before = self.workflow.clone();
                let moved = self.move_bar(bar, delta);
                if !ripple {
                    return moved;
                }
                self.rippled(
                    flat,
                    &before,
                    moved,
                    RippleScope {
                        after: bar.from + HALF_FRAME,
                        container: container_of(bar),
                        exclude: move_handle_key(bar),
                    },
                    delta,
                )
            }
            TimelineEdit::TrimStart { bar, delta } => {
                let bar = video_bar(flat, &bar)?;
                self.trim_bar(bar, true, delta)
            }
            TimelineEdit::TrimEnd { bar, delta, ripple } => {
                let bar = video_bar(flat, &bar)?;
                let before = self.workflow.clone();
                let trimmed = self.trim_bar(bar, false, delta);
                if !ripple {
                    return trimmed;
                }
                self.rippled(
                    flat,
                    &before,
                    trimmed,
                    RippleScope {
                        after: bar.to - HALF_FRAME,
                        container: container_of(bar),
                        exclude: None,
                    },
                    delta,
                )
            }
            TimelineEdit::Slip { bar, delta } => self.slip_bar(video_bar(flat, &bar)?, delta),
            TimelineEdit::Speed { bar, delta } => {
                let bar = video_bar(flat, &bar)?;
                let length = f64::from(whole_frames(bar.to - bar.from + delta, 1));
                self.speed_bar(bar, length)
            }
            TimelineEdit::RollStart { bar, delta } => {
                let bar = video_bar(flat, &bar)?;
                match neighbour(flat, bar, false) {
                    Some(left) => self.roll_cut(left, bar, delta),
                    None => self.trim_bar(bar, true, delta),
                }
            }
            TimelineEdit::RollEnd { bar, delta } => {
                let bar = video_bar(flat, &bar)?;
                match neighbour(flat, bar, true) {
                    Some(right) => self.roll_cut(bar, right, delta),
                    None => self.trim_bar(bar, false, delta),
                }
            }
            TimelineEdit::AudioMove { bar, delta, ripple } => {
                let bar = audio_bar(flat, &bar)?;
                let before = self.workflow.clone();
                let moved = self.move_audio(bar, delta);
                if !ripple {
                    return moved;
                }
                let key = match &bar.bar {
                    Some(video) => move_handle_key(video),
                    None => audio_move_handle_key(bar),
                };
                self.rippled(
                    flat,
                    &before,
                    moved,
                    RippleScope {
                        after: bar.from + HALF_FRAME,
                        container: None,
                        exclude: key,
                    },
                    delta,
                )
            }
            TimelineEdit::AudioTrimStart { bar, delta } => {
                self.trim_audio(audio_bar(flat, &bar)?, true, delta)
            }
            TimelineEdit::AudioTrimEnd { bar, delta } => {
                self.trim_audio(audio_bar(flat, &bar)?, false, delta)
            }
            TimelineEdit::Volume { bar, volume } => {
                self.set_audio_volume(audio_bar(flat, &bar)?, volume)
            }
            TimelineEdit::Split { at, selected } => {
                let bar = bar_under(flat, at, &selected)
                    .ok_or("Put the playhead over a clip to split it.")?;
                self.split_bar(bar, at)
            }
            TimelineEdit::SplitBar { bar, at } => self.split_bar(video_bar(flat, &bar)?, at),
            TimelineEdit::CameraKey { key, delta } => {
                let mark = flat
                    .camera_keys
                    .iter()
                    .find(|m| m.key == key)
                    .ok_or("That camera key is gone.")?;
                self.move_camera_key(mark, delta)
            }
            TimelineEdit::Keyframes { edit } => {
                super::keyframes::apply_keyframe_edit(&mut self.workflow, edit)
                    .map_err(|e| e.to_string())?;
                done()
            }
        }
    }

    pub fn selection_removal(&self, flat: &FlatTimeline, selected: &[String]) -> Option<Removal> {
        let mut nodes: Vec<String> = Vec::new();
        let mut edges: Vec<String> = Vec::new();
        let mut add = |removal: Removal| {
            for id in removal.node_ids {
                if !nodes.contains(&id) {
                    nodes.push(id);
                }
            }
            for id in removal.edge_ids {
                if !edges.contains(&id) {
                    edges.push(id);
                }
            }
        };
        for bar in flat.bars.iter().filter(|b| selected.contains(&b.node_id)) {
            add(self.removal_for_video(bar));
        }
        for audio in flat.audio.iter().filter(|a| {
            a.bar.is_none() && a.node_id.as_ref().is_some_and(|id| selected.contains(id))
        }) {
            add(self.removal_for_audio(audio));
        }
        (!nodes.is_empty() || !edges.is_empty()).then_some(Removal {
            node_ids: nodes,
            edge_ids: edges,
        })
    }
}

pub fn timeline_reply(outcome: &Outcome) -> TimelineReply {
    match outcome {
        Ok(done) => TimelineReply {
            error: None,
            message: done.message.clone(),
        },
        Err(error) => TimelineReply {
            error: Some(error.clone()),
            ..TimelineReply::default()
        },
    }
}
