import { MAX_SPEED } from "../nodes/params";
import { getNodeDef, defaultParams } from "../nodes/registry";
import type { ParamRef } from "../schema/timeline";
import type { Workflow, WorkflowEdge, WorkflowNode } from "../schema/workflow";
import { findHandle, slotIndex, type AudioBar, type ChainLink, type VideoBar } from "./flatten";

export type EditContext = {
  workflow: Workflow;
  fps: number;
  sourceNodeId: string | null;
  makeId: (prefix: string) => string;
};

export type EditResult =
  | { ok: true; workflow: Workflow; message?: string; selectNodeId?: string }
  | { ok: false; error: string };

const fail = (error: string): EditResult => ({ ok: false, error });

export const msFromFrames = (frames: number, fps: number) => Math.round((frames * 1000) / fps);
export const framesFromMs = (ms: number, fps: number) =>
  ms ? Math.max(1, Math.round((ms / 1000) * fps)) : 0;

const nodeName = (wf: Workflow, id: string) => {
  const n = wf.nodes.find((x) => x.id === id);
  if (!n) return "this clip";
  return n.label?.trim() || getNodeDef(n.type)?.title || "this clip";
};

export function readParam(wf: Workflow, ref: Pick<ParamRef, "nodeId" | "param" | "index">): number {
  const node = wf.nodes.find((n) => n.id === ref.nodeId);
  const raw = node?.params[ref.param] ?? defaultParams(node?.type ?? "")[ref.param];
  if (ref.index !== undefined) {
    const parts = String(raw ?? "").split(",");
    const n = Number(parts[ref.index]?.trim());
    return Number.isFinite(n) ? n : 0;
  }
  const n = Number(raw);
  return Number.isFinite(n) ? n : 0;
}

function writeParam(
  wf: Workflow,
  ref: Pick<ParamRef, "nodeId" | "param" | "index">,
  value: number,
): Workflow {
  return {
    ...wf,
    nodes: wf.nodes.map((n) => {
      if (n.id !== ref.nodeId) return n;
      if (ref.index === undefined) return { ...n, params: { ...n.params, [ref.param]: value } };
      const parts = String(n.params[ref.param] ?? defaultParams(n.type)[ref.param] ?? "")
        .split(",")
        .map((s) => Number(s.trim()) || 0);
      while (parts.length <= ref.index) parts.push(0);
      parts[ref.index] = value;
      while (parts.length > 1 && parts.at(-1) === 0) parts.pop();
      const text = parts.every((p) => p === 0) ? "" : parts.join(",");
      return { ...n, params: { ...n.params, [ref.param]: text } };
    }),
  };
}

export function shiftParam(
  wf: Workflow,
  ref: ParamRef,
  deltaContentFrames: number,
  fps: number,
): { workflow: Workflow; clamped: boolean } {
  const scale = ref.scale ?? 1;
  const shift = (r: Pick<ParamRef, "nodeId" | "param" | "index">, sign: number) => {
    const current = readParam(wf, r);
    const frames = ref.unit === "ms" ? framesFromMs(current, fps) : current;
    const next = frames + sign * deltaContentFrames * scale;
    const clampedNext = Math.max(0, Math.round(next));
    return {
      value: ref.unit === "ms" ? msFromFrames(clampedNext, fps) : clampedNext,
      clamped: next < -0.5,
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

export function setLength(
  wf: Workflow,
  ref: ParamRef,
  lengthContentFrames: number,
  fps: number,
): Workflow {
  const frames = Math.max(1, Math.round(lengthContentFrames * (ref.scale ?? 1)));
  return writeParam(wf, ref, ref.unit === "ms" ? msFromFrames(frames, fps) : frames);
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
    id: ctx.makeId(type.split(".")[0]),
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

function rootEdge(ctx: EditContext): WorkflowEdge | undefined {
  if (!ctx.sourceNodeId) return undefined;
  const def = getNodeDef(ctx.workflow.nodes.find((n) => n.id === ctx.sourceNodeId)?.type ?? "");
  if (!def) return undefined;
  return ctx.workflow.edges.find(
    (e) =>
      e.target === ctx.sourceNodeId &&
      ["clip", "timeline"].includes(def.inputs[e.targetHandle]?.type ?? ""),
  );
}

export function slotEdge(ctx: EditContext, bar: Pick<VideoBar, "chain">): WorkflowEdge | undefined {
  const i = slotIndex(bar);
  if (i === -1) return rootEdge(ctx);
  const slot = bar.chain[i].slot!;
  return ctx.workflow.edges.find((e) => e.target === slot.nodeId && e.targetHandle === slot.port);
}

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
  const next =
    bar.chain.slice(1).find((l) => l.nodeId && l.nodeId !== bar.nodeId)?.nodeId ??
    bar.chain.find((l) => l.slot)?.slot?.nodeId ??
    ctx.sourceNodeId ??
    undefined;
  if (!next) return undefined;
  const slot = bar.chain.find((l) => l.slot)?.slot;
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

const ok = (workflow: Workflow, message?: string, selectNodeId?: string): EditResult => ({
  ok: true,
  workflow,
  ...(message ? { message } : {}),
  ...(selectNodeId ? { selectNodeId } : {}),
});

export function moveBar(ctx: EditContext, bar: VideoBar, delta: number): EditResult {
  if (Math.abs(delta) < 0.5) return ok(ctx.workflow);
  const owner = findHandle(bar, "move", "outer");
  if (owner && owns(owner.link, ctx.workflow)) {
    const { workflow, clamped } = shiftParam(
      ctx.workflow,
      owner.link.handles!.move!,
      Math.round(delta * owner.link.rate),
      ctx.fps,
    );
    return ok(workflow, clamped ? "It can't start any earlier." : undefined);
  }
  if (delta < 0) {
    const slot = bar.chain.find((l) => l.slot)?.slot;
    const container = ctx.workflow.nodes.find((n) => n.id === slot?.nodeId);
    return fail(
      container?.type === "timeline.sequence"
        ? "In a sequence a clip starts when the one before it ends. Shorten that one instead."
        : "It already starts at the beginning.",
    );
  }
  const edge = slotEdge(ctx, bar);
  if (!edge) return fail("Connect this clip to an export or a sequence to move it here.");
  const level = bar.chain[Math.max(0, slotIndex(bar))]?.rate ?? 1;
  const ms = msFromFrames(Math.round(delta * level), ctx.fps);
  const { workflow, node } = insertOnEdge(
    ctx,
    ctx.workflow,
    edge,
    "timeline.delay",
    { ms },
    "Delay from timeline",
  );
  return ok(
    workflow,
    `Added “Delay from timeline” before ${nodeName(ctx.workflow, bar.nodeId)}.`,
    node.id,
  );
}

export function trimBar(
  ctx: EditContext,
  bar: VideoBar,
  edge: "start" | "end",
  delta: number,
): EditResult {
  if (Math.abs(delta) < 0.5) return ok(ctx.workflow);
  if (edge === "start") {
    const owner = findHandle(bar, "trimStart", "outer");
    if (owner && owns(owner.link, ctx.workflow)) {
      const { workflow, clamped } = shiftParam(
        ctx.workflow,
        owner.link.handles!.trimStart!,
        Math.round(delta * owner.link.rate),
        ctx.fps,
      );
      return ok(workflow, clamped ? "That's the start of the clip." : undefined);
    }
    if (delta < 0) return fail("That's the start of the clip.");
    const target = leafEdge(ctx, bar);
    if (!target) return fail("Connect this clip to an export or a sequence to trim it here.");
    const start = Math.round((bar.contentStart + delta) * bar.rate);
    const { workflow, node } = insertOnEdge(
      ctx,
      ctx.workflow,
      target,
      "timeline.trim",
      { startMs: msFromFrames(start, ctx.fps), durationMs: 0 },
      "Trim from timeline",
    );
    return ok(
      workflow,
      `Added “Trim from timeline” after ${nodeName(ctx.workflow, bar.nodeId)}.`,
      node.id,
    );
  }
  const owner = findHandle(bar, "trimEnd", "outer");
  if (owner && owns(owner.link, ctx.workflow)) {
    const length = owner.link.end + delta * owner.link.rateEnd;
    if (length < 1) return fail("A clip needs at least one frame.");
    return ok(setLength(ctx.workflow, owner.link.handles!.trimEnd!, length, ctx.fps));
  }
  if (delta > 0) return fail("That's the end of the clip.");
  const target = leafEdge(ctx, bar);
  if (!target) return fail("Connect this clip to an export or a sequence to trim it here.");
  const keep = Math.round(bar.contentEnd - bar.contentStart + delta * bar.rate);
  if (keep < 1) return fail("A clip needs at least one frame.");
  const { workflow, node } = insertOnEdge(
    ctx,
    ctx.workflow,
    target,
    "timeline.trim",
    {
      startMs: msFromFrames(Math.round(bar.contentStart), ctx.fps),
      durationMs: msFromFrames(keep, ctx.fps),
    },
    "Trim from timeline",
  );
  return ok(
    workflow,
    `Added “Trim from timeline” after ${nodeName(ctx.workflow, bar.nodeId)}.`,
    node.id,
  );
}

export function slipBar(ctx: EditContext, bar: VideoBar, delta: number): EditResult {
  if (Math.abs(delta) < 0.5) return ok(ctx.workflow);
  const owner = findHandle(bar, "trimStart", "outer");
  const content = Math.round(delta * bar.rate);
  if (owner && owns(owner.link, ctx.workflow)) {
    const ref = owner.link.handles!.trimStart!;
    const length = Math.round((owner.link.to - owner.link.from) * owner.link.rate);
    const lengthRef = owner.link.handles!.trimEnd;
    let wf = ctx.workflow;
    if (lengthRef) wf = setLength(wf, lengthRef, length, ctx.fps);
    const { workflow, clamped } = shiftParam(wf, { ...ref, linked: [] }, content, ctx.fps);
    return ok(workflow, clamped ? "That's the start of the clip." : undefined);
  }
  if (content < 0) return fail("That's the start of the clip.");
  const target = leafEdge(ctx, bar);
  if (!target) return fail("Connect this clip to an export or a sequence to edit it here.");
  const length = Math.round((bar.contentEnd - bar.contentStart) * 1);
  const { workflow, node } = insertOnEdge(
    ctx,
    ctx.workflow,
    target,
    "timeline.trim",
    {
      startMs: msFromFrames(Math.round(bar.contentStart) + content, ctx.fps),
      durationMs: msFromFrames(length, ctx.fps),
    },
    "Trim from timeline",
  );
  return ok(
    workflow,
    `Added “Trim from timeline” after ${nodeName(ctx.workflow, bar.nodeId)}.`,
    node.id,
  );
}

export function rollCut(
  ctx: EditContext,
  left: VideoBar,
  right: VideoBar,
  delta: number,
): EditResult {
  if (Math.abs(delta) < 0.5) return ok(ctx.workflow);
  const a = trimBar(ctx, left, "end", delta);
  if (!a.ok) return a;
  const b = trimBar({ ...ctx, workflow: a.workflow }, right, "start", delta);
  if (!b.ok) return b;
  return ok(b.workflow, [a.message, b.message].filter(Boolean).join(" ") || undefined);
}

export function speedBar(ctx: EditContext, bar: VideoBar, newLength: number): EditResult {
  const owner = findHandle(bar, "speed", "outer");
  const i = slotIndex(bar);
  const span = owner?.link ?? bar.chain[i === -1 ? bar.chain.length - 1 : i];
  const current = span.to - span.from;
  if (newLength < 1 || current < 1) return fail("A clip needs at least one frame.");
  const factor = current / newLength;
  if (owner && owns(owner.link, ctx.workflow)) {
    const ref = owner.link.handles!.speed!;
    const rate = readParam(ctx.workflow, ref) || 1;
    const next = Math.min(MAX_SPEED, Math.max(0.1, Math.round(rate * factor * 100) / 100));
    return ok(writeParam(ctx.workflow, ref, next));
  }
  const edge = slotEdge(ctx, bar);
  if (!edge) return fail("Connect this clip to an export or a sequence to change its speed here.");
  const rate = Math.min(MAX_SPEED, Math.max(0.1, Math.round(factor * 100) / 100));
  const { workflow, node } = insertOnEdge(
    ctx,
    ctx.workflow,
    edge,
    "timeline.speed",
    { rate },
    "Speed from timeline",
  );
  return ok(
    workflow,
    `Added “Speed from timeline” after ${nodeName(ctx.workflow, bar.nodeId)}.`,
    node.id,
  );
}

const SEQ_PORTS = ["in1", "in2", "in3", "in4", "in5", "in6"];

export function splitBar(ctx: EditContext, bar: VideoBar, at: number): EditResult {
  if (at <= bar.from + 0.5 || at >= bar.to - 0.5) return fail("Put the playhead inside the clip.");
  const edge = slotEdge(ctx, bar);
  if (!edge) return fail("Connect this clip to an export or a sequence to split it here.");
  const i = slotIndex(bar);
  const level = i === -1 ? null : bar.chain[i];
  const rate = level?.rate ?? 1;
  const offset = Math.round((at - (level?.from ?? 0)) * rate);
  if (offset < 1) return fail("Put the playhead inside the clip.");
  let wf = ctx.workflow;
  const head = newNode(
    ctx,
    wf,
    "timeline.trim",
    { startMs: 0, durationMs: msFromFrames(offset, ctx.fps) },
    "Split: first part",
    { source: edge.source, target: edge.target },
    0,
  );
  const tail = newNode(
    ctx,
    wf,
    "timeline.trim",
    { startMs: msFromFrames(offset, ctx.fps), durationMs: 0 },
    "Split: second part",
    { source: edge.source, target: edge.target },
    1,
  );
  const feed = (target: string) =>
    makeEdge({
      source: edge.source,
      sourceHandle: edge.sourceHandle,
      target,
      targetHandle: "clip",
    });
  const container = wf.nodes.find((n) => n.id === edge.target);
  const used = new Set(wf.edges.filter((e) => e.target === edge.target).map((e) => e.targetHandle));
  const port = SEQ_PORTS.indexOf(edge.targetHandle);
  const lastUsed = Math.max(-1, ...[...used].map((h) => SEQ_PORTS.indexOf(h)));
  const inPlace =
    container?.type === "timeline.sequence" && port !== -1 && lastUsed < SEQ_PORTS.length - 1;
  if (inPlace) {
    const shiftPort = (handle: string) => {
      const k = SEQ_PORTS.indexOf(handle);
      return k <= port ? handle : SEQ_PORTS[k + 1];
    };
    const cuts: Array<Record<string, unknown>> = (
      (container.params.cuts ?? []) as Array<Record<string, unknown>>
    ).flatMap((c) => {
      const before = shiftPort(String(c.before));
      return before ? [{ ...c, before }] : [];
    });
    if ((container.params.transition ?? "core/fade") !== "none")
      cuts.push({
        before: SEQ_PORTS[port + 1],
        transition: "none",
        transitionDuration: 15,
        keepWhole: false,
      });
    const shifted = wf.edges.map((e) =>
      e.target === edge.target && SEQ_PORTS.includes(e.targetHandle)
        ? makeEdge({ ...e, targetHandle: shiftPort(e.targetHandle) })
        : e,
    );
    wf = {
      ...wf,
      nodes: [
        ...wf.nodes.map((n) =>
          n.id === container.id ? { ...n, params: { ...n.params, cuts } } : n,
        ),
        head,
        tail,
      ],
      edges: [
        ...shifted.filter((e) => e.id !== edge.id),
        feed(head.id),
        feed(tail.id),
        makeEdge({
          source: head.id,
          sourceHandle: "clip",
          target: edge.target,
          targetHandle: SEQ_PORTS[port],
        }),
        makeEdge({
          source: tail.id,
          sourceHandle: "clip",
          target: edge.target,
          targetHandle: SEQ_PORTS[port + 1],
        }),
      ],
    };
    return ok(wf, `Split ${nodeName(ctx.workflow, bar.nodeId)} in two.`, tail.id);
  }
  const join = newNode(
    ctx,
    wf,
    "timeline.sequence",
    { transition: "none" },
    "Split",
    { source: edge.source, target: edge.target },
    2,
  );
  wf = {
    ...wf,
    nodes: [...wf.nodes, head, tail, join],
    edges: [
      ...wf.edges.filter((e) => e.id !== edge.id),
      feed(head.id),
      feed(tail.id),
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
  return ok(wf, `Split ${nodeName(ctx.workflow, bar.nodeId)} in two.`, tail.id);
}

export function moveAudio(ctx: EditContext, bar: AudioBar, delta: number): EditResult {
  if (Math.abs(delta) < 0.5) return ok(ctx.workflow);
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
  if (Math.abs(delta) < 0.5) return ok(ctx.workflow);
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
  const v = Math.round(Math.min(2, Math.max(0, volume)) * 100) / 100;
  if (bar.bar) {
    const id = bar.bar.nodeId;
    return ok({
      ...ctx.workflow,
      nodes: ctx.workflow.nodes.map((n) =>
        n.id === id ? { ...n, params: { ...n.params, volume: v } } : n,
      ),
    });
  }
  const ref = bar.handles?.volume;
  if (!ref) return fail("This sound's volume is set in the mixer.");
  return ok(writeParam(ctx.workflow, ref, v));
}

export type RippleTarget = { ref: ParamRef; rate: number; key: string };

export function rippleTargets(
  ctx: EditContext,
  bars: VideoBar[],
  audio: AudioBar[],
  after: number,
  exclude: { container?: string; refs: Set<string> },
): RippleTarget[] {
  const out = new Map<string, RippleTarget>();
  const keyOf = (r: ParamRef) => `${r.nodeId}:${r.param}:${r.index ?? ""}`;
  for (const bar of bars) {
    if (bar.from < after - 0.5) continue;
    const slot = bar.chain.find((l) => l.slot)?.slot;
    if (slot && slot.nodeId === exclude.container) continue;
    const owner = findHandle(bar, "move", "outer");
    const ref = owner?.link.handles?.move;
    if (!ref || !owns(owner.link, ctx.workflow)) continue;
    const key = keyOf(ref);
    if (exclude.refs.has(key)) continue;
    out.set(key, { ref, rate: owner.link.rate, key });
  }
  for (const a of audio) {
    if (a.bar || a.from < after - 0.5 || !a.handles?.move) continue;
    const key = keyOf(a.handles.move);
    if (exclude.refs.has(key)) continue;
    out.set(key, { ref: a.handles.move, rate: 1, key });
  }
  return [...out.values()];
}

export function applyRipple(
  ctx: EditContext,
  wf: Workflow,
  targets: RippleTarget[],
  delta: number,
): Workflow {
  let out = wf;
  for (const t of targets)
    out = shiftParam(out, t.ref, Math.round(delta * t.rate), ctx.fps).workflow;
  return out;
}

export const handleKey = (r: ParamRef) => `${r.nodeId}:${r.param}:${r.index ?? ""}`;

export function moveHandleKey(bar: VideoBar | AudioBar): string | null {
  if ("chain" in bar) {
    const owner = findHandle(bar, "move", "outer");
    return owner?.link.handles?.move ? handleKey(owner.link.handles.move) : null;
  }
  return bar.handles?.move ? handleKey(bar.handles.move) : null;
}

export function containerOf(bar: VideoBar): string | undefined {
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
  const channels = wf.mix
    ? Object.fromEntries(Object.entries(wf.mix.channels).filter(([id]) => !gone.has(id)))
    : undefined;
  return {
    ...wf,
    nodes: wf.nodes.filter((n) => !gone.has(n.id)),
    edges: wf.edges.filter((e) => !edges.has(e.id) && !gone.has(e.source) && !gone.has(e.target)),
    ...(wf.mix && channels ? { mix: { ...wf.mix, channels } } : {}),
  };
}
