import { z } from "zod";
import { MAX_SPEED, MAX_VOLUME, MIN_SPEED } from "../nodes/params";
import { getNodeDef, defaultParams, nodeNameById } from "../nodes/registry";
import { cutSchema, IN_PORTS } from "../nodes/timeline";
import type { ParamRef } from "../schema/timeline";
import {
  normalizeWorkflow,
  type PortType,
  type Workflow,
  type WorkflowEdge,
  type WorkflowNode,
} from "../schema/workflow";
import { findHandle, slotIndex, type AudioBar, type ChainLink, type VideoBar } from "./flatten";
import { clamp, finiteOr, round2 } from "../math";
import { nodeIdPrefix } from "../ids";
import { formatNumberList, parseNumberList } from "../format";
import { framesToMs, msToFrames } from "./time";

export type EditContext = {
  workflow: Workflow;
  fps: number;
  sourceNodeId: string | null;
  makeId: (prefix: string) => string;
};

export type EditResult =
  | { ok: true; workflow: Workflow; message?: string; selectNodeId?: string }
  | { ok: false; error: string };

const HALF_FRAME = 0.5;

const noMove = (delta: number) => Math.abs(delta) < HALF_FRAME;

const fail = (error: string): EditResult => ({ ok: false, error });

const notConnected = (doIt: string) =>
  fail(`Connect this clip to an export or a sequence to ${doIt} here.`);

const ok = (workflow: Workflow, message?: string, selectNodeId?: string): EditResult => ({
  ok: true,
  workflow,
  ...(message ? { message } : {}),
  ...(selectNodeId ? { selectNodeId } : {}),
});

const clipName = (wf: Workflow, id: string) => nodeNameById(wf, id, "this clip");

type ParamAt = Pick<ParamRef, "nodeId" | "param" | "index">;

const paramValue = (node: WorkflowNode, param: string): unknown =>
  node.params[param] ?? defaultParams(node.type)[param];

function readParam(wf: Workflow, ref: ParamAt): number {
  const node = wf.nodes.find((n) => n.id === ref.nodeId);
  if (!node) return 0;
  const raw = paramValue(node, ref.param);
  return ref.index === undefined ? finiteOr(raw, 0) : (parseNumberList(raw)[ref.index] ?? 0);
}

function writeParam(wf: Workflow, ref: ParamAt, value: number): Workflow {
  const write = (node: WorkflowNode): unknown => {
    if (ref.index === undefined) return value;
    const parts = parseNumberList(paramValue(node, ref.param));
    while (parts.length <= ref.index) parts.push(0);
    parts[ref.index] = value;
    return formatNumberList(parts);
  };
  return {
    ...wf,
    nodes: wf.nodes.map((n) =>
      n.id === ref.nodeId ? { ...n, params: { ...n.params, [ref.param]: write(n) } } : n,
    ),
  };
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

function setLength(
  wf: Workflow,
  ref: ParamRef,
  lengthContentFrames: number,
  fps: number,
): Workflow {
  const frames = Math.max(1, Math.round(lengthContentFrames * (ref.scale ?? 1)));
  return writeParam(wf, ref, ref.unit === "ms" ? framesToMs(frames, fps) : frames);
}

function absPosition(wf: Workflow, node: WorkflowNode): { x: number; y: number } {
  if (!node.parentId) return node.position;
  const parent = wf.nodes.find((n) => n.id === node.parentId);
  if (!parent) return node.position;
  const p = absPosition(wf, parent);
  return { x: p.x + node.position.x, y: p.y + node.position.y };
}

const edgeId = (e: Omit<WorkflowEdge, "id">) =>
  `${e.source}.${e.sourceHandle}->${e.target}.${e.targetHandle}`;

const makeEdge = (e: Omit<WorkflowEdge, "id">): WorkflowEdge => ({ ...e, id: edgeId(e) });

function placeNear(
  wf: Workflow,
  source: WorkflowNode | undefined,
  target: WorkflowNode | undefined,
  index = 0,
): Pick<WorkflowNode, "position" | "parentId"> {
  const anchor = source ?? target;
  if (!anchor) return { position: { x: 0, y: 0 } };
  const abs = absPosition(wf, anchor);
  const pos = { x: abs.x + 60 + index * 30, y: abs.y + 170 + index * 150 };
  const parentId =
    source?.parentId && target?.parentId === source.parentId ? source.parentId : undefined;
  if (!parentId) return { position: pos };
  const parent = wf.nodes.find((n) => n.id === parentId)!;
  const p = absPosition(wf, parent);
  return { position: { x: pos.x - p.x, y: pos.y - p.y }, parentId };
}

function newNode(
  ctx: EditContext,
  wf: Workflow,
  type: string,
  params: Record<string, unknown>,
  label: string,
  near: { source?: string; target?: string },
  index = 0,
): WorkflowNode {
  const source = wf.nodes.find((n) => n.id === near.source);
  const target = wf.nodes.find((n) => n.id === near.target);
  return {
    id: ctx.makeId(nodeIdPrefix(type)),
    type,
    label,
    params: { ...defaultParams(type), ...params },
    ...placeNear(wf, source, target, index),
  };
}

function insertOnEdge(
  ctx: EditContext,
  wf: Workflow,
  edge: WorkflowEdge,
  type: string,
  params: Record<string, unknown>,
  label: string,
): { workflow: Workflow; node: WorkflowNode } {
  const node = newNode(ctx, wf, type, params, label, { source: edge.source, target: edge.target });
  const def = getNodeDef(type)!;
  const inPort = Object.keys(def.inputs)[0];
  const outPort = Object.keys(def.outputs)[0];
  return {
    node,
    workflow: {
      ...wf,
      nodes: [...wf.nodes, node],
      edges: [
        ...wf.edges.filter((e) => e.id !== edge.id),
        makeEdge({
          source: edge.source,
          sourceHandle: edge.sourceHandle,
          target: node.id,
          targetHandle: inPort,
        }),
        makeEdge({
          source: node.id,
          sourceHandle: outPort,
          target: edge.target,
          targetHandle: edge.targetHandle,
        }),
      ],
    },
  };
}

const ROOT_PORT_TYPES = new Set<PortType>(["clip", "timeline"]);

function rootEdge(ctx: EditContext): WorkflowEdge | undefined {
  const source = ctx.workflow.nodes.find((n) => n.id === ctx.sourceNodeId);
  const def = source && getNodeDef(source.type);
  if (!def) return undefined;
  return ctx.workflow.edges.find((e) => {
    const type = def.inputs[e.targetHandle]?.type;
    return e.target === source.id && !!type && ROOT_PORT_TYPES.has(type);
  });
}

function slotEdge(ctx: EditContext, bar: Pick<VideoBar, "chain">): WorkflowEdge | undefined {
  const i = slotIndex(bar);
  if (i === -1) return rootEdge(ctx);
  const slot = bar.chain[i].slot!;
  return ctx.workflow.edges.find((e) => e.target === slot.nodeId && e.targetHandle === slot.port);
}

const slotLink = (bar: Pick<VideoBar, "chain">): ChainLink => {
  const i = slotIndex(bar);
  return bar.chain[i === -1 ? bar.chain.length - 1 : i];
};

const ROOT_LEVEL = { rate: 1, from: 0 };

const slotLevel = (bar: Pick<VideoBar, "chain">): { rate: number; from: number } =>
  slotIndex(bar) === -1 ? ROOT_LEVEL : slotLink(bar);

function pathEdge(wf: Workflow, from: string, to: string): WorkflowEdge | undefined {
  const seen = new Set<string>([from]);
  const queue: Array<{ id: string; first?: WorkflowEdge }> = [{ id: from }];
  while (queue.length) {
    const { id, first } = queue.shift()!;
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

function leafEdge(ctx: EditContext, bar: VideoBar): WorkflowEdge | undefined {
  const slot = bar.chain.find((l) => l.slot)?.slot;
  const next =
    bar.chain.slice(1).find((l) => l.nodeId && l.nodeId !== bar.nodeId)?.nodeId ??
    slot?.nodeId ??
    ctx.sourceNodeId ??
    undefined;
  if (!next) return undefined;
  if (slot && next === slot.nodeId) {
    const direct = ctx.workflow.edges.find(
      (e) => e.source === bar.nodeId && e.target === slot.nodeId && e.targetHandle === slot.port,
    );
    if (direct) return direct;
  }
  return pathEdge(ctx.workflow, bar.nodeId, next);
}

const owns = (link: ChainLink, wf: Workflow) =>
  !!link.handles &&
  Object.values(link.handles).some((h) => wf.nodes.some((n) => n.id === h?.nodeId));

function ownedHandle(
  ctx: EditContext,
  bar: VideoBar,
  kind: keyof NonNullable<ChainLink["handles"]>,
): { link: ChainLink; ref: ParamRef } | null {
  const owner = findHandle(bar, kind, "outer");
  const ref = owner?.link.handles?.[kind];
  return owner && ref && owns(owner.link, ctx.workflow) ? { link: owner.link, ref } : null;
}

const ADDED_NODES = {
  "timeline.delay": { label: "Delay from timeline", where: "before" },
  "timeline.trim": { label: "Trim from timeline", where: "after" },
  "timeline.speed": { label: "Speed from timeline", where: "after" },
} as const;

function addNode(
  ctx: EditContext,
  bar: VideoBar,
  edge: WorkflowEdge,
  type: keyof typeof ADDED_NODES,
  params: Record<string, unknown>,
): EditResult {
  const { label, where } = ADDED_NODES[type];
  const { workflow, node } = insertOnEdge(ctx, ctx.workflow, edge, type, params, label);
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

export function moveBar(ctx: EditContext, bar: VideoBar, delta: number): EditResult {
  if (noMove(delta)) return ok(ctx.workflow);
  const owned = ownedHandle(ctx, bar, "move");
  if (owned) {
    const content = Math.round(delta * owned.link.rate);
    const { workflow, clamped } = shiftParam(ctx.workflow, owned.ref, content, ctx.fps);
    return ok(workflow, clamped ? "It can't start any earlier." : undefined);
  }
  if (delta < 0) {
    const container = ctx.workflow.nodes.find((n) => n.id === containerOf(bar));
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
  if (owned) {
    const content = Math.round(delta * owned.link.rate);
    const { workflow, clamped } = shiftParam(ctx.workflow, owned.ref, content, ctx.fps);
    return ok(workflow, clamped ? "That's the start of the clip." : undefined);
  }
  if (delta < 0) return fail("That's the start of the clip.");
  const edge = leafEdge(ctx, bar);
  if (!edge) return notConnected("trim it");
  return addTrim(ctx, bar, edge, Math.round((bar.contentStart + delta) * bar.rate), 0);
}

function trimEnd(ctx: EditContext, bar: VideoBar, delta: number): EditResult {
  const owned = ownedHandle(ctx, bar, "trimEnd");
  if (owned) {
    const length = owned.link.end + delta * owned.link.rateEnd;
    if (length < 1) return fail("A clip needs at least one frame.");
    return ok(setLength(ctx.workflow, owned.ref, length, ctx.fps));
  }
  if (delta > 0) return fail("That's the end of the clip.");
  const edge = leafEdge(ctx, bar);
  if (!edge) return notConnected("trim it");
  const keep = Math.round(bar.contentEnd - bar.contentStart + delta * bar.rate);
  if (keep < 1) return fail("A clip needs at least one frame.");
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
    const { workflow, clamped } = shiftParam(wf, { ...owned.ref, linked: [] }, content, ctx.fps);
    return ok(workflow, clamped ? "That's the start of the clip." : undefined);
  }
  if (content < 0) return fail("That's the start of the clip.");
  const edge = leafEdge(ctx, bar);
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
  const a = trimBar(ctx, left, "end", delta);
  if (!a.ok) return a;
  const b = trimBar({ ...ctx, workflow: a.workflow }, right, "start", delta);
  if (!b.ok) return b;
  return ok(b.workflow, [a.message, b.message].filter(Boolean).join(" ") || undefined);
}

const clampRate = (rate: number) => clamp(round2(rate), MIN_SPEED, MAX_SPEED);

export function speedBar(ctx: EditContext, bar: VideoBar, newLength: number): EditResult {
  const owned = ownedHandle(ctx, bar, "speed");
  const span = findHandle(bar, "speed", "outer")?.link ?? slotLink(bar);
  const current = span.to - span.from;
  if (newLength < 1 || current < 1) return fail("A clip needs at least one frame.");
  const factor = current / newLength;
  if (owned) {
    const rate = readParam(ctx.workflow, owned.ref) || 1;
    return ok(writeParam(ctx.workflow, owned.ref, clampRate(rate * factor)));
  }
  const edge = slotEdge(ctx, bar);
  if (!edge) return notConnected("change its speed");
  return addNode(ctx, bar, edge, "timeline.speed", { rate: clampRate(factor) });
}

const cutsSchema = z.array(cutSchema);

function splitParts(ctx: EditContext, edge: WorkflowEdge, offset: number) {
  const near = { source: edge.source, target: edge.target };
  const head = newNode(
    ctx,
    ctx.workflow,
    "timeline.trim",
    { startMs: 0, durationMs: framesToMs(offset, ctx.fps) },
    "Split: first part",
    near,
    0,
  );
  const tail = newNode(
    ctx,
    ctx.workflow,
    "timeline.trim",
    { startMs: framesToMs(offset, ctx.fps), durationMs: 0 },
    "Split: second part",
    near,
    1,
  );
  const feeds = [head, tail].map((part) =>
    makeEdge({
      source: edge.source,
      sourceHandle: edge.sourceHandle,
      target: part.id,
      targetHandle: "clip",
    }),
  );
  return { head, tail, feeds };
}

type SplitParts = ReturnType<typeof splitParts>;

const SEQUENCE_PORTS: readonly string[] = IN_PORTS;

function splitInSequence(
  ctx: EditContext,
  edge: WorkflowEdge,
  container: WorkflowNode,
  { head, tail, feeds }: SplitParts,
): Workflow {
  const port = SEQUENCE_PORTS.indexOf(edge.targetHandle);
  const shiftPort = (handle: string) => {
    const k = SEQUENCE_PORTS.indexOf(handle);
    return k <= port ? handle : SEQUENCE_PORTS[k + 1];
  };
  const cuts = cutsSchema.parse(container.params.cuts ?? []).flatMap((c) => {
    const before = shiftPort(c.before);
    return before ? [{ ...c, before }] : [];
  });
  const transition = container.params.transition ?? defaultParams(container.type).transition;
  if (transition !== "none")
    cuts.push(cutSchema.parse({ before: SEQUENCE_PORTS[port + 1], transition: "none" }));
  const wf = ctx.workflow;
  const shifted = wf.edges.map((e) =>
    e.target === edge.target && SEQUENCE_PORTS.includes(e.targetHandle)
      ? makeEdge({ ...e, targetHandle: shiftPort(e.targetHandle) })
      : e,
  );
  return {
    ...wf,
    nodes: [
      ...wf.nodes.map((n) => (n.id === container.id ? { ...n, params: { ...n.params, cuts } } : n)),
      head,
      tail,
    ],
    edges: [
      ...shifted.filter((e) => e.id !== edge.id),
      ...feeds,
      makeEdge({
        source: head.id,
        sourceHandle: "clip",
        target: edge.target,
        targetHandle: SEQUENCE_PORTS[port],
      }),
      makeEdge({
        source: tail.id,
        sourceHandle: "clip",
        target: edge.target,
        targetHandle: SEQUENCE_PORTS[port + 1],
      }),
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
    wf,
    "timeline.sequence",
    { transition: "none" },
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
      makeEdge({ source: head.id, sourceHandle: "clip", target: join.id, targetHandle: "in1" }),
      makeEdge({ source: tail.id, sourceHandle: "clip", target: join.id, targetHandle: "in2" }),
      makeEdge({
        source: join.id,
        sourceHandle: "clip",
        target: edge.target,
        targetHandle: edge.targetHandle,
      }),
    ],
  };
}

function hasFreePort(wf: Workflow, container: WorkflowNode, edge: WorkflowEdge): boolean {
  if (container.type !== "timeline.sequence" || !SEQUENCE_PORTS.includes(edge.targetHandle))
    return false;
  const used = wf.edges.filter((e) => e.target === container.id).map((e) => e.targetHandle);
  const lastUsed = Math.max(-1, ...used.map((h) => SEQUENCE_PORTS.indexOf(h)));
  return lastUsed < SEQUENCE_PORTS.length - 1;
}

export function splitBar(ctx: EditContext, bar: VideoBar, at: number): EditResult {
  if (at <= bar.from + HALF_FRAME || at >= bar.to - HALF_FRAME)
    return fail("Put the playhead inside the clip.");
  const edge = slotEdge(ctx, bar);
  if (!edge) return notConnected("split it");
  const level = slotLevel(bar);
  const offset = Math.round((at - level.from) * level.rate);
  if (offset < 1) return fail("Put the playhead inside the clip.");
  const parts = splitParts(ctx, edge, offset);
  const container = ctx.workflow.nodes.find((n) => n.id === edge.target);
  const workflow =
    container && hasFreePort(ctx.workflow, container, edge)
      ? splitInSequence(ctx, edge, container, parts)
      : splitWithNewSequence(ctx, edge, parts);
  return ok(workflow, `Split ${clipName(ctx.workflow, bar.nodeId)} in two.`, parts.tail.id);
}

export function moveAudio(ctx: EditContext, bar: AudioBar, delta: number): EditResult {
  if (noMove(delta)) return ok(ctx.workflow);
  if (bar.bar) return moveBar(ctx, bar.bar, delta);
  const ref = bar.handles?.move;
  if (!ref) return fail("This sound can't be moved here.");
  const { workflow, clamped } = shiftParam(ctx.workflow, ref, Math.round(delta), ctx.fps);
  return ok(workflow, clamped ? "It can't start any earlier." : undefined);
}

export function trimAudio(
  ctx: EditContext,
  bar: AudioBar,
  edge: "start" | "end",
  delta: number,
): EditResult {
  if (noMove(delta)) return ok(ctx.workflow);
  if (bar.bar) return trimBar(ctx, bar.bar, edge, delta);
  if (edge === "start") {
    const ref = bar.handles?.trimStart;
    if (!ref) return fail("Sound effects play in full. Move it instead.");
    const { workflow, clamped } = shiftParam(ctx.workflow, ref, Math.round(delta), ctx.fps);
    return ok(workflow, clamped ? "That's the start of the sound." : undefined);
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
    "chain" in bar ? findHandle(bar, "move", "outer")?.link.handles?.move : bar.handles?.move;
  return ref ? handleKey(ref) : null;
}

export function containerOf(bar: Pick<VideoBar, "chain">): string | undefined {
  return bar.chain.find((l) => l.slot)?.slot?.nodeId;
}

export type Removal = { nodeIds: string[]; edgeIds: string[] };

export function removalFor(ctx: EditContext, bar: VideoBar | AudioBar): Removal {
  const wf = ctx.workflow;
  if (!("chain" in bar)) {
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
  const gone = new Set(removal.nodeIds);
  const edges = new Set(removal.edgeIds);
  return normalizeWorkflow({
    ...wf,
    nodes: wf.nodes.filter((n) => !gone.has(n.id)),
    edges: wf.edges.filter((e) => !edges.has(e.id) && !gone.has(e.source) && !gone.has(e.target)),
  });
}
