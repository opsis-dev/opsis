import { moveCameraKey, sortedCameraKeys } from "@opsis/video/common/camera-path";
import { isRecord, isString } from "@opsis/video/common/guards";
import { MAX_SPEED, MAX_VOLUME, MIN_SPEED } from "@opsis/video/common/limits";
import { findById } from "@opsis/video/common/lists";
import { clamp, numberOr, round2 } from "@opsis/video/common/math";
import { framesToMs, msToFrames, wholeFrames } from "@opsis/video/common/time";
import { findHandle, slotIndex, slotLink } from "@/core/handles";
import { edgeId, nodeIdPrefix } from "@/core/ids";
import { CLIP_PORT, IN_PORTS, inPortIndex, nextInPort } from "@/core/nodes";
import { asSchema, schemaDefaults } from "@/core/schema";
import { formatNumberList, parseNumberList } from "@/core/text-input";
import { cameraPathSchema, type EditHandleKind } from "@/core/timeline";
import type { EdgeEnds } from "@/core/workflow";
import type {
  ParamRef,
  PortType,
  Workflow,
  WorkflowEdge,
  WorkflowNode,
  AudioBar,
  CameraKeyMark,
  ChainLink,
  VideoBar,
} from "@/generated/bindings";
import { defaultParams, getNodeDef, NO_TRANSITION, nodeNameById } from "@/store/catalog";
import { absPosition, mapNode, patchNodeParams, removeNodes } from "../workflow-edits";

export type EditContext = {
  workflow: Workflow;
  fps: number;
  sourceNodeId: string | null;
  makeId: (prefix: string) => string;
};

export type EditResult =
  | { ok: true; workflow: Workflow; message?: string; selectNodeId?: string }
  | { ok: false; error: string };

export const HALF_FRAME = 0.5;

const noMove = (delta: number) => Math.abs(delta) < HALF_FRAME;

const fail = (error: string): EditResult => ({ ok: false, error });

const NEEDS_A_FRAME = "A clip needs at least one frame.";
const PLAYHEAD_OUTSIDE_CLIP = "Put the playhead inside the clip.";

const notConnected = (doIt: string) =>
  fail(`Connect this clip to an export or a sequence to ${doIt} here.`);

const ok = (workflow: Workflow, message?: string, selectNodeId?: string): EditResult => ({
  ok: true,
  workflow,
  ...(message ? { message } : {}),
  ...(selectNodeId ? { selectNodeId } : {}),
});

const clipName = (wf: Workflow, id: string) => nodeNameById(wf.nodes, id, "this clip");

type ParamAt = Pick<ParamRef, "nodeId" | "param" | "index">;

const paramValue = (node: WorkflowNode, param: string): unknown =>
  node.params[param] ?? defaultParams(node.type)[param];

const numberListParam = (node: WorkflowNode, param: string): number[] => {
  const raw = paramValue(node, param);
  return parseNumberList(isString(raw) ? raw : undefined);
};

function readParam(wf: Workflow, ref: ParamAt): number {
  const node = findById(wf.nodes, ref.nodeId);
  if (!node) return 0;
  return ref.index === undefined
    ? numberOr(paramValue(node, ref.param), 0)
    : (numberListParam(node, ref.param)[ref.index] ?? 0);
}

function writeParam(wf: Workflow, ref: ParamAt, value: number): Workflow {
  const write = (node: WorkflowNode): unknown => {
    if (ref.index === undefined) return value;
    const parts = numberListParam(node, ref.param);
    while (parts.length <= ref.index) parts.push(0);
    parts[ref.index] = value;
    return formatNumberList(parts);
  };
  return mapNode(wf, ref.nodeId, (n) => ({ ...n, params: { ...n.params, [ref.param]: write(n) } }));
}

function shiftParam(
  wf: Workflow,
  ref: ParamRef,
  deltaContentFrames: number,
  fps: number,
): { workflow: Workflow; clamped: boolean } {
  const scale = ref.scale ?? 1;
  const shift = (r: ParamAt, sign: number) => {
    const current = readParam(wf, r);
    const frames = ref.unit === "ms" ? msToFrames(current, fps, 0) : current;
    const next = frames + sign * deltaContentFrames * scale;
    const clampedNext = Math.max(0, Math.round(next));
    return {
      value: ref.unit === "ms" ? framesToMs(clampedNext, fps) : clampedNext,
      clamped: next < -HALF_FRAME,
    };
  };
  const main = shift(ref, 1);
  let out = writeParam(wf, ref, main.value);
  let clamped = main.clamped;
  for (const link of ref.linked ?? []) {
    const r = { nodeId: ref.nodeId, param: link.param };
    if (link.ifNonZero && readParam(wf, r) === 0) continue;
    const s = shift(r, link.sign);
    out = writeParam(out, r, s.value);
    clamped ||= s.clamped;
  }
  return { workflow: out, clamped };
}

function shiftTo(
  wf: Workflow,
  ref: ParamRef,
  deltaContentFrames: number,
  fps: number,
  atLimit: string,
): EditResult {
  const { workflow, clamped } = shiftParam(wf, ref, deltaContentFrames, fps);
  return ok(workflow, clamped ? atLimit : undefined);
}

function setLength(
  wf: Workflow,
  ref: ParamRef,
  lengthContentFrames: number,
  fps: number,
): Workflow {
  const frames = wholeFrames(lengthContentFrames * (ref.scale ?? 1), 1);
  return writeParam(wf, ref, ref.unit === "ms" ? framesToMs(frames, fps) : frames);
}

const makeEdge = (e: EdgeEnds): WorkflowEdge => ({ ...e, id: edgeId(e) });

const NEW_NODE_OFFSET = { x: 60, y: 170 };
const NEW_NODE_STEP = { x: 30, y: 150 };

function placeNear(
  wf: Workflow,
  source: WorkflowNode | undefined,
  target: WorkflowNode | undefined,
  index = 0,
): Pick<WorkflowNode, "position" | "parentId"> {
  const anchor = source ?? target;
  if (!anchor) return { position: { x: 0, y: 0 } };
  const abs = absPosition(wf, anchor);
  const pos = {
    x: abs.x + NEW_NODE_OFFSET.x + index * NEW_NODE_STEP.x,
    y: abs.y + NEW_NODE_OFFSET.y + index * NEW_NODE_STEP.y,
  };
  const parentId =
    source?.parentId && target?.parentId === source.parentId ? source.parentId : undefined;
  const parent = findById(wf.nodes, parentId);
  if (!parent) return { position: pos };
  const p = absPosition(wf, parent);
  return { position: { x: pos.x - p.x, y: pos.y - p.y }, parentId: parent.id };
}

function newNode(
  ctx: EditContext,
  type: string,
  params: Record<string, unknown>,
  label: string,
  near: { source?: string; target?: string },
  index = 0,
): WorkflowNode {
  const wf = ctx.workflow;
  const source = findById(wf.nodes, near.source);
  const target = findById(wf.nodes, near.target);
  return {
    id: ctx.makeId(nodeIdPrefix(type)),
    type,
    label,
    params: { ...defaultParams(type), ...params },
    ...placeNear(wf, source, target, index),
  };
}

const clipEdge = ({
  source,
  sourceHandle = CLIP_PORT,
  target,
  targetHandle,
}: {
  source: string;
  sourceHandle?: string;
  target: string;
  targetHandle: string;
}): WorkflowEdge => makeEdge({ source, sourceHandle, target, targetHandle });

const ADDED_NODES = {
  "timeline.delay": { label: "Delay from timeline", where: "before" },
  "timeline.trim": { label: "Trim from timeline", where: "after" },
  "timeline.speed": { label: "Speed from timeline", where: "after" },
} as const;

type AddedNode = keyof typeof ADDED_NODES;

function insertOnEdge(
  ctx: EditContext,
  edge: WorkflowEdge,
  type: AddedNode,
  params: Record<string, unknown>,
): { workflow: Workflow; node: WorkflowNode } {
  const wf = ctx.workflow;
  const { label } = ADDED_NODES[type];
  const node = newNode(ctx, type, params, label, { source: edge.source, target: edge.target });
  return {
    node,
    workflow: {
      ...wf,
      nodes: [...wf.nodes, node],
      edges: [
        ...wf.edges.filter((e) => e.id !== edge.id),
        clipEdge({
          source: edge.source,
          sourceHandle: edge.sourceHandle,
          target: node.id,
          targetHandle: CLIP_PORT,
        }),
        clipEdge({ source: node.id, target: edge.target, targetHandle: edge.targetHandle }),
      ],
    },
  };
}

const ROOT_PORT_TYPES = new Set<PortType>(["clip"]);

function rootEdge(ctx: EditContext): WorkflowEdge | undefined {
  const source = findById(ctx.workflow.nodes, ctx.sourceNodeId);
  const def = source && getNodeDef(source.type);
  if (!def) return undefined;
  return ctx.workflow.edges.find((e) => {
    const type = def.inputs[e.targetHandle]?.type;
    return e.target === source.id && !!type && ROOT_PORT_TYPES.has(type);
  });
}

function slotEdge(ctx: EditContext, bar: Pick<VideoBar, "chain">): WorkflowEdge | undefined {
  const slot = slotLink(bar)?.slot;
  if (!slot) return rootEdge(ctx);
  return ctx.workflow.edges.find((e) => e.target === slot.nodeId && e.targetHandle === slot.port);
}

const outermostLink = (bar: Pick<VideoBar, "chain">): ChainLink =>
  slotLink(bar) ?? bar.chain[bar.chain.length - 1];

const ROOT_LEVEL = { rate: 1, from: 0 };

const slotLevel = (bar: Pick<VideoBar, "chain">): { rate: number; from: number } =>
  slotLink(bar) ?? ROOT_LEVEL;

function firstEdgeOnPath(wf: Workflow, from: string, to: string): WorkflowEdge | undefined {
  const seen = new Set<string>([from]);
  const queue: Array<{ id: string; first?: WorkflowEdge }> = [{ id: from }];
  for (let i = 0; i < queue.length; i++) {
    const { id, first } = queue[i];
    for (const e of wf.edges) {
      if (e.source !== id || seen.has(e.target)) continue;
      const head = first ?? e;
      if (e.target === to) return head;
      seen.add(e.target);
      queue.push({ id: e.target, first: head });
    }
  }
  return undefined;
}

function nodeAfterClip(ctx: EditContext, bar: VideoBar): string | null {
  const wrapper = bar.chain.slice(1).find((l) => l.nodeId && l.nodeId !== bar.nodeId);
  return wrapper?.nodeId ?? slotLink(bar)?.slot?.nodeId ?? ctx.sourceNodeId;
}

function slotInputEdge(ctx: EditContext, bar: VideoBar, next: string): WorkflowEdge | undefined {
  const slot = slotLink(bar)?.slot;
  if (!slot || slot.nodeId !== next) return undefined;
  return ctx.workflow.edges.find(
    (e) => e.source === bar.nodeId && e.target === slot.nodeId && e.targetHandle === slot.port,
  );
}

function edgeOutOfClip(ctx: EditContext, bar: VideoBar): WorkflowEdge | undefined {
  const next = nodeAfterClip(ctx, bar);
  if (!next) return undefined;
  return slotInputEdge(ctx, bar, next) ?? firstEdgeOnPath(ctx.workflow, bar.nodeId, next);
}

function ownedHandle(
  ctx: EditContext,
  bar: VideoBar,
  kind: EditHandleKind,
): { link: ChainLink; ref: ParamRef } | null {
  const link = findHandle(bar, kind, "outer");
  const ref = link?.handles?.[kind];
  return link && ref && findById(ctx.workflow.nodes, ref.nodeId) ? { link, ref } : null;
}

function shiftOwned(
  ctx: EditContext,
  owned: { link: ChainLink; ref: ParamRef },
  delta: number,
  atLimit: string,
): EditResult {
  return shiftTo(ctx.workflow, owned.ref, Math.round(delta * owned.link.rate), ctx.fps, atLimit);
}

function addNode(
  ctx: EditContext,
  bar: VideoBar,
  edge: WorkflowEdge,
  type: AddedNode,
  params: Record<string, unknown>,
): EditResult {
  const { label, where } = ADDED_NODES[type];
  const { workflow, node } = insertOnEdge(ctx, edge, type, params);
  return ok(workflow, `Added “${label}” ${where} ${clipName(ctx.workflow, bar.nodeId)}.`, node.id);
}

const addTrim = (
  ctx: EditContext,
  bar: VideoBar,
  edge: WorkflowEdge,
  startFrames: number,
  lengthFrames: number,
) =>
  addNode(ctx, bar, edge, "timeline.trim", {
    startMs: framesToMs(startFrames, ctx.fps),
    durationMs: framesToMs(lengthFrames, ctx.fps),
  });

const AT_START = "That's the start of the clip.";
const TOO_EARLY = "It can't start any earlier.";

export function moveBar(ctx: EditContext, bar: VideoBar, delta: number): EditResult {
  if (noMove(delta)) return ok(ctx.workflow);
  const owned = ownedHandle(ctx, bar, "move");
  if (owned) return shiftOwned(ctx, owned, delta, TOO_EARLY);
  if (delta < 0) {
    const container = findById(ctx.workflow.nodes, containerOf(bar));
    return fail(
      container?.type === "timeline.sequence"
        ? "In a sequence a clip starts when the one before it ends. Shorten that one instead."
        : "It already starts at the beginning.",
    );
  }
  const edge = slotEdge(ctx, bar);
  if (!edge) return notConnected("move it");
  return addNode(ctx, bar, edge, "timeline.delay", {
    ms: framesToMs(Math.round(delta * slotLevel(bar).rate), ctx.fps),
  });
}

function trimStart(ctx: EditContext, bar: VideoBar, delta: number): EditResult {
  const owned = ownedHandle(ctx, bar, "trimStart");
  if (owned) return shiftOwned(ctx, owned, delta, AT_START);
  if (delta < 0) return fail(AT_START);
  const edge = edgeOutOfClip(ctx, bar);
  if (!edge) return notConnected("trim it");
  return addTrim(ctx, bar, edge, Math.round((bar.contentStart + delta) * bar.rate), 0);
}

function trimEnd(ctx: EditContext, bar: VideoBar, delta: number): EditResult {
  const owned = ownedHandle(ctx, bar, "trimEnd");
  if (owned) {
    const length = owned.link.end + delta * owned.link.rateEnd;
    if (length < 1) return fail(NEEDS_A_FRAME);
    return ok(setLength(ctx.workflow, owned.ref, length, ctx.fps));
  }
  if (delta > 0) return fail("That's the end of the clip.");
  const edge = edgeOutOfClip(ctx, bar);
  if (!edge) return notConnected("trim it");
  const keep = Math.round(bar.contentEnd - bar.contentStart + delta * bar.rate);
  if (keep < 1) return fail(NEEDS_A_FRAME);
  return addTrim(ctx, bar, edge, Math.round(bar.contentStart), keep);
}

export function trimBar(
  ctx: EditContext,
  bar: VideoBar,
  edge: "start" | "end",
  delta: number,
): EditResult {
  if (noMove(delta)) return ok(ctx.workflow);
  return edge === "start" ? trimStart(ctx, bar, delta) : trimEnd(ctx, bar, delta);
}

export function slipBar(ctx: EditContext, bar: VideoBar, delta: number): EditResult {
  if (noMove(delta)) return ok(ctx.workflow);
  const owned = ownedHandle(ctx, bar, "trimStart");
  const content = Math.round(delta * bar.rate);
  if (owned) {
    const { link } = owned;
    const length = Math.round((link.to - link.from) * link.rate);
    const lengthRef = link.handles?.trimEnd;
    const wf = lengthRef ? setLength(ctx.workflow, lengthRef, length, ctx.fps) : ctx.workflow;
    return shiftTo(wf, { ...owned.ref, linked: [] }, content, ctx.fps, AT_START);
  }
  if (content < 0) return fail(AT_START);
  const edge = edgeOutOfClip(ctx, bar);
  if (!edge) return notConnected("edit it");
  const length = Math.round(bar.contentEnd - bar.contentStart);
  return addTrim(ctx, bar, edge, Math.round(bar.contentStart) + content, length);
}

export function rollCut(
  ctx: EditContext,
  left: VideoBar,
  right: VideoBar,
  delta: number,
): EditResult {
  if (noMove(delta)) return ok(ctx.workflow);
  const a = trimEnd(ctx, left, delta);
  if (!a.ok) return a;
  const b = trimStart({ ...ctx, workflow: a.workflow }, right, delta);
  if (!b.ok) return b;
  return ok(b.workflow, [a.message, b.message].filter(Boolean).join(" ") || undefined);
}

const clampRate = (rate: number) => clamp(round2(rate), MIN_SPEED, MAX_SPEED);

export function speedBar(ctx: EditContext, bar: VideoBar, newLength: number): EditResult {
  const owned = ownedHandle(ctx, bar, "speed");
  const span = findHandle(bar, "speed", "outer") ?? outermostLink(bar);
  const current = span.to - span.from;
  if (newLength < 1 || current < 1) return fail(NEEDS_A_FRAME);
  const factor = current / newLength;
  if (owned) {
    const rate = readParam(ctx.workflow, owned.ref) || 1;
    return ok(writeParam(ctx.workflow, owned.ref, clampRate(rate * factor)));
  }
  const edge = slotEdge(ctx, bar);
  if (!edge) return notConnected("change its speed");
  return addNode(ctx, bar, edge, "timeline.speed", { rate: clampRate(factor) });
}

type Cut = Record<string, unknown> & { before: string };

const isCutList = (value: unknown): value is Cut[] =>
  Array.isArray(value) && value.every((c) => isRecord(c) && isString(c.before));

const newCut = (before: string): Cut => {
  const sequence = asSchema(getNodeDef("timeline.sequence")?.params);
  const item = asSchema(asSchema(sequence.properties?.cuts).items);
  return { ...schemaDefaults(item), before, transition: NO_TRANSITION };
};

function splitParts(ctx: EditContext, edge: WorkflowEdge, offset: number) {
  const near = { source: edge.source, target: edge.target };
  const cut = framesToMs(offset, ctx.fps);
  const part = (label: string, params: Record<string, unknown>, index: number) =>
    newNode(ctx, "timeline.trim", params, label, near, index);
  const head = part("Split: first part", { startMs: 0, durationMs: cut }, 0);
  const tail = part("Split: second part", { startMs: cut, durationMs: 0 }, 1);
  const feeds = [head, tail].map((p) =>
    clipEdge({
      source: edge.source,
      sourceHandle: edge.sourceHandle,
      target: p.id,
      targetHandle: CLIP_PORT,
    }),
  );
  return { head, tail, feeds };
}

type SplitParts = ReturnType<typeof splitParts>;

function splitInSequence(
  ctx: EditContext,
  edge: WorkflowEdge,
  container: WorkflowNode,
  next: string,
  { head, tail, feeds }: SplitParts,
): Workflow {
  const port = inPortIndex(edge.targetHandle);
  const shiftPort = (handle: string): string | undefined =>
    inPortIndex(handle) <= port ? handle : nextInPort(handle);
  const saved = isCutList(container.params.cuts) ? container.params.cuts : [];
  const cuts = saved.flatMap((c) => {
    const before = shiftPort(c.before);
    return before ? [{ ...c, before }] : [];
  });
  if (paramValue(container, "transition") !== NO_TRANSITION) cuts.push(newCut(next));
  const wf = ctx.workflow;
  const shifted = wf.edges.flatMap((e) => {
    if (e.target !== edge.target || inPortIndex(e.targetHandle) === -1) return [e];
    const targetHandle = shiftPort(e.targetHandle);
    return targetHandle ? [makeEdge({ ...e, targetHandle })] : [];
  });
  return {
    ...wf,
    nodes: [...patchNodeParams(wf, container.id, { cuts }).nodes, head, tail],
    edges: [
      ...shifted.filter((e) => e.id !== edge.id),
      ...feeds,
      clipEdge({ source: head.id, target: edge.target, targetHandle: edge.targetHandle }),
      clipEdge({ source: tail.id, target: edge.target, targetHandle: next }),
    ],
  };
}

function splitWithNewSequence(
  ctx: EditContext,
  edge: WorkflowEdge,
  { head, tail, feeds }: SplitParts,
): Workflow {
  const wf = ctx.workflow;
  const join = newNode(
    ctx,
    "timeline.sequence",
    { transition: NO_TRANSITION },
    "Split",
    { source: edge.source, target: edge.target },
    2,
  );
  return {
    ...wf,
    nodes: [...wf.nodes, head, tail, join],
    edges: [
      ...wf.edges.filter((e) => e.id !== edge.id),
      ...feeds,
      clipEdge({ source: head.id, target: join.id, targetHandle: "in1" }),
      clipEdge({ source: tail.id, target: join.id, targetHandle: "in2" }),
      clipEdge({ source: join.id, target: edge.target, targetHandle: edge.targetHandle }),
    ],
  };
}

function freePortAfter(
  wf: Workflow,
  container: WorkflowNode | undefined,
  edge: WorkflowEdge,
): string | undefined {
  if (container?.type !== "timeline.sequence" || inPortIndex(edge.targetHandle) === -1)
    return undefined;
  const used = wf.edges.filter((e) => e.target === container.id).map((e) => e.targetHandle);
  const lastUsed = Math.max(-1, ...used.map((h) => inPortIndex(h)));
  return lastUsed < IN_PORTS.length - 1 ? nextInPort(edge.targetHandle) : undefined;
}

export function splitBar(ctx: EditContext, bar: VideoBar, at: number): EditResult {
  if (at <= bar.from + HALF_FRAME || at >= bar.to - HALF_FRAME) return fail(PLAYHEAD_OUTSIDE_CLIP);
  const edge = slotEdge(ctx, bar);
  if (!edge) return notConnected("split it");
  const level = slotLevel(bar);
  const offset = Math.round((at - level.from) * level.rate);
  if (offset < 1) return fail(PLAYHEAD_OUTSIDE_CLIP);
  const parts = splitParts(ctx, edge, offset);
  const container = findById(ctx.workflow.nodes, edge.target);
  const next = freePortAfter(ctx.workflow, container, edge);
  const workflow =
    container && next
      ? splitInSequence(ctx, edge, container, next, parts)
      : splitWithNewSequence(ctx, edge, parts);
  return ok(workflow, `Split ${clipName(ctx.workflow, bar.nodeId)} in two.`, parts.tail.id);
}

export function moveAudio(ctx: EditContext, bar: AudioBar, delta: number): EditResult {
  if (bar.bar) return moveBar(ctx, bar.bar, delta);
  if (noMove(delta)) return ok(ctx.workflow);
  const ref = bar.handles?.move;
  if (!ref) return fail("This sound can't be moved here.");
  return shiftTo(ctx.workflow, ref, Math.round(delta), ctx.fps, TOO_EARLY);
}

export function trimAudio(
  ctx: EditContext,
  bar: AudioBar,
  edge: "start" | "end",
  delta: number,
): EditResult {
  if (bar.bar) return trimBar(ctx, bar.bar, edge, delta);
  if (noMove(delta)) return ok(ctx.workflow);
  if (edge === "start") {
    const ref = bar.handles?.trimStart;
    if (!ref) return fail("Sound effects play in full. Move it instead.");
    return shiftTo(ctx.workflow, ref, Math.round(delta), ctx.fps, "That's the start of the sound.");
  }
  const ref = bar.handles?.trimEnd;
  if (!ref) return fail("Sound effects play in full.");
  const length = bar.to - bar.from + delta;
  if (length < 1) return fail("A sound needs at least one frame.");
  return ok(setLength(ctx.workflow, ref, length, ctx.fps));
}

export function setAudioVolume(ctx: EditContext, bar: AudioBar, volume: number): EditResult {
  const v = round2(clamp(volume, 0, MAX_VOLUME));
  if (bar.bar) return ok(writeParam(ctx.workflow, { nodeId: bar.bar.nodeId, param: "volume" }, v));
  const ref = bar.handles?.volume;
  if (!ref) return fail("This sound's volume is set in the mixer.");
  return ok(writeParam(ctx.workflow, ref, v));
}

const handleKey = (r: ParamRef) => `${r.nodeId}:${r.param}:${r.index ?? ""}`;

type RippleTarget = { ref: ParamRef; rate: number; key: string };

export function rippleTargets(
  ctx: EditContext,
  bars: VideoBar[],
  audio: AudioBar[],
  after: number,
  exclude: { container?: string; refs: Set<string> },
): RippleTarget[] {
  const out = new Map<string, RippleTarget>();
  for (const bar of bars) {
    if (bar.from < after - HALF_FRAME) continue;
    const container = containerOf(bar);
    if (container && container === exclude.container) continue;
    const owned = ownedHandle(ctx, bar, "move");
    if (!owned) continue;
    const key = handleKey(owned.ref);
    if (exclude.refs.has(key)) continue;
    out.set(key, { ref: owned.ref, rate: owned.link.rate, key });
  }
  for (const a of audio) {
    if (a.bar || a.from < after - HALF_FRAME || !a.handles?.move) continue;
    const key = handleKey(a.handles.move);
    if (exclude.refs.has(key)) continue;
    out.set(key, { ref: a.handles.move, rate: 1, key });
  }
  return [...out.values()];
}

export function applyRipple(
  wf: Workflow,
  targets: RippleTarget[],
  delta: number,
  fps: number,
): Workflow {
  let out = wf;
  for (const t of targets) out = shiftParam(out, t.ref, Math.round(delta * t.rate), fps).workflow;
  return out;
}

export function moveHandleKey(bar: VideoBar | AudioBar): string | null {
  const ref =
    bar.type === "video" ? findHandle(bar, "move", "outer")?.handles?.move : bar.handles?.move;
  return ref ? handleKey(ref) : null;
}

export function containerOf(bar: Pick<VideoBar, "chain">): string | undefined {
  return slotLink(bar)?.slot.nodeId;
}

export type Removal = { nodeIds: string[]; edgeIds: string[] };

export function removalFor(ctx: EditContext, bar: VideoBar | AudioBar): Removal {
  const wf = ctx.workflow;
  if (bar.type === "audio") {
    if (bar.bar) return removalFor(ctx, bar.bar);
    return { nodeIds: bar.nodeId ? [bar.nodeId] : [], edgeIds: [] };
  }
  const slot = slotIndex(bar);
  const ids: string[] = [];
  const end = slot === -1 ? bar.chain.length : slot;
  for (let i = 0; i < end; i++) {
    const id = bar.chain[i].nodeId;
    if (!id || ids.includes(id) || id === ctx.sourceNodeId) continue;
    if (wf.edges.filter((e) => e.source === id).length > 1) break;
    ids.push(id);
  }
  if (ids.length === 0) {
    const edge = slotEdge(ctx, bar);
    return { nodeIds: [], edgeIds: edge ? [edge.id] : [] };
  }
  return { nodeIds: ids, edgeIds: [] };
}

export function applyRemoval(wf: Workflow, removal: Removal): Workflow {
  const without = removeNodes(wf, removal.nodeIds);
  const edges = new Set(removal.edgeIds);
  return { ...without, edges: without.edges.filter((e) => !edges.has(e.id)) };
}

export function moveCameraKeyMark(
  ctx: EditContext,
  mark: CameraKeyMark,
  delta: number,
): EditResult {
  if (noMove(delta)) return { ok: true, workflow: ctx.workflow };
  const node = findById(ctx.workflow.nodes, mark.nodeId);
  const path = cameraPathSchema.safeParse(node?.params.camera);
  if (!node || !path.success)
    return fail("This node's camera keys can't be read. Check them in the Inspector.");
  const key = sortedCameraKeys(path.data)[mark.index];
  if (!key) return fail("That camera key is gone.");
  const camera = moveCameraKey(path.data, mark.index, key.frame + delta);
  return { ok: true, workflow: patchNodeParams(ctx.workflow, mark.nodeId, { camera }) };
}
