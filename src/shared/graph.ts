import type { z } from "zod";
import { getNodeDef, nodeName } from "./nodes/registry";
import { canConnect } from "./nodes/ports";
import { isOutputType } from "./nodes/output";
import type { PortSpec } from "./nodes/define";
import { fieldMeta } from "./elements/introspect";
import type { PortType, Workflow, WorkflowEdge, WorkflowNode } from "./schema/workflow";
import { secretFileName } from "./secrets";
import { interpolate } from "./vars";
import { humanize } from "./format";
import { GROUP_TYPE, NOTE_TYPE } from "./nodes/utility";

export type GraphIssue = {
  level: "error" | "warning";
  message: string;
  nodeId?: string;
  edgeId?: string;
};

export const isGroupNode = (node: Pick<WorkflowNode, "type">) => node.type === GROUP_TYPE;

export const isNoteNode = (node: Pick<WorkflowNode, "type">) => node.type === NOTE_TYPE;

export const isStructural = (node: WorkflowNode) => isGroupNode(node) || isNoteNode(node);

const byPosition = (a: WorkflowNode, b: WorkflowNode) =>
  a.position.x - b.position.x || a.position.y - b.position.y || (a.id < b.id ? -1 : 1);

export function topoSort(wf: Workflow): { order: string[]; cyclic: string[] } {
  const nodes = wf.nodes.filter((n) => !isStructural(n));
  const byId: Record<string, WorkflowNode> = Object.fromEntries(nodes.map((n) => [n.id, n]));
  const indegree: Record<string, number> = Object.fromEntries(nodes.map((n) => [n.id, 0]));
  const next: Record<string, WorkflowNode[]> = Object.fromEntries(nodes.map((n) => [n.id, []]));
  for (const e of wf.edges) {
    if (!Object.hasOwn(byId, e.source) || !Object.hasOwn(byId, e.target)) continue;
    next[e.source].push(byId[e.target]);
    indegree[e.target] += 1;
  }
  const ready = nodes.filter((n) => indegree[n.id] === 0).sort(byPosition);
  const order: string[] = [];
  for (let node = ready.shift(); node; node = ready.shift()) {
    order.push(node.id);
    for (const after of next[node.id]) {
      indegree[after.id] -= 1;
      if (indegree[after.id] === 0) ready.push(after);
    }
    ready.sort(byPosition);
  }
  const ordered = new Set(order);
  return { order, cyclic: nodes.filter((n) => !ordered.has(n.id)).map((n) => n.id) };
}

function reach(wf: Workflow, nodeId: string, direction: "up" | "down"): Set<string> {
  const links = new Map<string, string[]>();
  for (const e of wf.edges) {
    const [from, to] = direction === "up" ? [e.target, e.source] : [e.source, e.target];
    const list = links.get(from);
    if (list) list.push(to);
    else links.set(from, [to]);
  }
  const out = new Set<string>();
  const stack = [nodeId];
  for (let id = stack.pop(); id !== undefined; id = stack.pop())
    for (const linked of links.get(id) ?? [])
      if (!out.has(linked)) {
        out.add(linked);
        stack.push(linked);
      }
  return out;
}

export const upstreamOf = (wf: Workflow, nodeId: string): Set<string> => reach(wf, nodeId, "up");

export const downstreamOf = (wf: Workflow, nodeId: string): Set<string> =>
  reach(wf, nodeId, "down");

export function exportedClipNodes(wf: Workflow): string[] {
  const out = new Set<string>();
  for (const node of wf.nodes) {
    if (getNodeDef(node.type)?.phase !== "render") continue;
    for (const id of upstreamOf(wf, node.id)) {
      const upstream = wf.nodes.find((n) => n.id === id);
      if (upstream && "clip" in (getNodeDef(upstream.type)?.outputs ?? {})) out.add(id);
    }
  }
  return [...out];
}

type ConnectionEnds = {
  source: string;
  sourceHandle: string;
  target: string;
  targetHandle: string;
};

type ConnectionProblem = { notFound: boolean; message: string };

export function portOf(
  wf: Workflow,
  nodeId: string,
  side: "in" | "out",
  handle: string,
): PortSpec | undefined {
  const node = wf.nodes.find((n) => n.id === nodeId);
  const def = node ? getNodeDef(node.type) : undefined;
  return def ? (side === "in" ? def.inputs : def.outputs)[handle] : undefined;
}

const portList = (ports: Record<string, PortSpec> | undefined) => {
  const ids = Object.keys(ports ?? {});
  return ids.length ? ids.join(", ") : "none";
};

export function connectionProblem(wf: Workflow, c: ConnectionEnds): ConnectionProblem | null {
  const source = wf.nodes.find((n) => n.id === c.source);
  const target = wf.nodes.find((n) => n.id === c.target);
  if (!source) return { notFound: true, message: `There's no node “${c.source}”.` };
  if (!target) return { notFound: true, message: `There's no node “${c.target}”.` };
  if (source.id === target.id)
    return { notFound: false, message: "A node can't connect to itself." };
  const from = portOf(wf, c.source, "out", c.sourceHandle);
  const to = portOf(wf, c.target, "in", c.targetHandle);
  if (!from)
    return {
      notFound: false,
      message: `“${c.source}” has no output “${c.sourceHandle}”. Its outputs: ${portList(getNodeDef(source.type)?.outputs)}.`,
    };
  if (!to)
    return {
      notFound: false,
      message: `“${c.target}” has no input “${c.targetHandle}”. Its inputs: ${portList(getNodeDef(target.type)?.inputs)}.`,
    };
  if (!canConnect(from.type, to.type))
    return {
      notFound: false,
      message: `A ${from.type} output can't connect to a ${to.type} input.`,
    };
  return null;
}

export function runTargets(wf: Workflow): string[] {
  const executes = (n: WorkflowNode | undefined) =>
    !!n && !n.disabled && !isStructural(n) && !!getNodeDef(n.type)?.executes;
  const byId = new Map(wf.nodes.map((n) => [n.id, n]));
  const outputs = wf.nodes.filter((n) => executes(n) && isOutputType(n.type));
  if (!outputs.length) return wf.nodes.filter(executes).map((n) => n.id);
  const targets = new Set(outputs.map((n) => n.id));
  for (const out of outputs)
    for (const id of downstreamOf(wf, out.id)) if (executes(byId.get(id))) targets.add(id);
  return [...targets];
}

type SessionWalk = { chain: WorkflowNode[]; problem: null } | { chain: null; problem: string };

function walkSession(wf: Workflow, captureNodeId: string): SessionWalk {
  const byId = new Map(wf.nodes.map((n) => [n.id, n]));
  const chain: WorkflowNode[] = [];
  const seen = new Set<string>();
  const fail = (problem: string): SessionWalk => ({ chain: null, problem });
  for (let current = captureNodeId; ;) {
    const edge = wf.edges.find((e) => e.target === current && e.targetHandle === "session");
    if (!edge) break;
    if (seen.has(edge.source)) return fail("The browser action chain loops back on itself.");
    seen.add(edge.source);
    const node = byId.get(edge.source);
    if (!node) return fail(`Missing node ${edge.source}`);
    chain.unshift(node);
    if (node.type === "browser.session") return { chain, problem: null };
    current = node.id;
  }
  const head = chain[0];
  return fail(
    head
      ? `The action chain starting at "${nodeName(head)}" is not connected to a Session node.`
      : "Connect a Session → actions chain to this node's session input.",
  );
}

export function sessionChain(wf: Workflow, captureNodeId: string): WorkflowNode[] {
  const walk = walkSession(wf, captureNodeId);
  if (walk.problem !== null) throw new Error(walk.problem);
  return walk.chain;
}

export function loginDependencies(
  wf: Workflow,
  captureNodeId: string,
  vars: Record<string, string>,
): string[] {
  const param = (node: WorkflowNode, key: string) => {
    const value = interpolate(node.params, vars)[key];
    return typeof value === "string" ? value.trim() : "";
  };
  const { chain } = walkSession(wf, captureNodeId);
  const storageState = chain ? param(chain[0], "storageState") : "";
  if (!storageState) return [];
  const secret = secretFileName(storageState);
  return wf.nodes
    .filter(
      (n) =>
        n.type === "capture.loginOnce" &&
        !n.disabled &&
        secretFileName(param(n, "secretName")) === secret,
    )
    .map((n) => n.id);
}

const TYPE_WORDS: Record<PortType, string> = {
  session: "browser action",
  recording: "recording",
  clip: "clip",
  scene: "clip",
  timeline: "clip",
  audio: "sound",
  captions: "captions",
  image: "image",
  media: "file",
  file: "file",
};

export function describeParamIssue(shape: z.ZodObject["shape"], issue: z.core.$ZodIssue): string {
  const key = String(issue.path[0] ?? "");
  const field: z.ZodType | undefined = shape[key];
  const label =
    (field ? fieldMeta(field).description : undefined) || (key ? humanize(key) : "A setting");
  if (issue.code === "too_small" && issue.minimum === 1 && /string|character/i.test(issue.message))
    return `${label} is empty`;
  if (issue.code === "too_small") return `${label} is too small`;
  if (issue.code === "too_big") return `${label} is too large`;
  if (issue.code === "invalid_type")
    return /undefined/.test(issue.message) ? `${label} is missing` : `${label} isn't valid`;
  if (issue.code === "invalid_value") return `${label} has a value that isn't allowed`;
  if (issue.code === "custom" || !/expected|invalid/i.test(issue.message)) return issue.message;
  return `${label} isn't valid`;
}

function portIssues(
  wf: Workflow,
  node: WorkflowNode,
  inputs: Record<string, PortSpec>,
): GraphIssue[] {
  const issues: GraphIssue[] = [];
  for (const [portId, spec] of Object.entries(inputs)) {
    const edges = wf.edges.filter((e) => e.target === node.id && e.targetHandle === portId);
    const name = spec.label ?? humanize(portId);
    if (edges.length === 0 && !spec.optional)
      issues.push({ level: "warning", nodeId: node.id, message: `Connect something to “${name}”` });
    if (edges.length > 1 && !spec.many)
      issues.push({
        level: "error",
        nodeId: node.id,
        message: `“${name}” can only take one connection`,
      });
  }
  return issues;
}

function nodeIssues(wf: Workflow): GraphIssue[] {
  const seen = new Set<string>();
  return wf.nodes.flatMap((node): GraphIssue[] => {
    const duplicate = seen.has(node.id);
    seen.add(node.id);
    const issues: GraphIssue[] = duplicate
      ? [{ level: "error", nodeId: node.id, message: "Two nodes share this id" }]
      : [];
    if (isStructural(node)) return issues;
    const def = getNodeDef(node.type);
    if (!def)
      return [
        ...issues,
        { level: "error", nodeId: node.id, message: "This node type isn't available" },
      ];
    if (node.disabled) return issues;
    const parsed = def.params.safeParse(node.params);
    if (!parsed.success)
      issues.push({
        level: "error",
        nodeId: node.id,
        message: describeParamIssue(def.params.shape, parsed.error.issues[0]),
      });
    return [...issues, ...portIssues(wf, node, def.inputs)];
  });
}

function edgeIssue(e: WorkflowEdge, byId: Map<string, WorkflowNode>): GraphIssue | null {
  const s = byId.get(e.source);
  const t = byId.get(e.target);
  if (!s || !t)
    return { level: "error", edgeId: e.id, message: "A connection points at a deleted node" };
  const sp = getNodeDef(s.type)?.outputs[e.sourceHandle];
  const tp = getNodeDef(t.type)?.inputs[e.targetHandle];
  if (!sp || !tp)
    return {
      level: "error",
      edgeId: e.id,
      nodeId: t.id,
      message: "A connection uses a point that no longer exists",
    };
  if (!canConnect(sp.type, tp.type))
    return {
      level: "error",
      edgeId: e.id,
      nodeId: t.id,
      message: `A ${TYPE_WORDS[sp.type]} can't connect to a ${TYPE_WORDS[tp.type]} input`,
    };
  return null;
}

function edgeIssues(wf: Workflow): GraphIssue[] {
  const byId = new Map(wf.nodes.map((n) => [n.id, n]));
  return wf.edges.flatMap((e) => edgeIssue(e, byId) ?? []);
}

const cycleIssues = (wf: Workflow): GraphIssue[] =>
  topoSort(wf).cyclic.map((id) => ({
    level: "error",
    nodeId: id,
    message: "This node loops back into itself",
  }));

export const validateWorkflow = (wf: Workflow): GraphIssue[] => [
  ...nodeIssues(wf),
  ...edgeIssues(wf),
  ...cycleIssues(wf),
];
