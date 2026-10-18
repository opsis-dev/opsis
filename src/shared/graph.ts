import { getNodeDef, nodeName } from "./nodes/registry";
import { canConnect } from "./nodes/ports";
import { isOutputType } from "./nodes/output";
import type { PortSpec } from "./nodes/define";
import { parseParams } from "./nodes/params";
import {
  PORT_WORDS,
  type PortType,
  type Workflow,
  type WorkflowEdge,
  type WorkflowNode,
} from "./schema/workflow";
import { secretFileName } from "./secrets";
import { interpolate } from "./vars";
import { humanize } from "./format";
import { GROUP_TYPE, NOTE_TYPE } from "./nodes/utility";
import { byCodeUnit, findById } from "./lists";

export type GraphIssue = {
  level: "error" | "warning";
  message: string;
  nodeId?: string;
  edgeId?: string;
};

export const isGroupNode = (node: Pick<WorkflowNode, "type">) => node.type === GROUP_TYPE;

export const isNoteNode = (node: Pick<WorkflowNode, "type">) => node.type === NOTE_TYPE;

export const isStructural = (node: WorkflowNode) => isGroupNode(node) || isNoteNode(node);

export const nodesById = (nodes: readonly WorkflowNode[]): Map<string, WorkflowNode> =>
  new Map(nodes.map((n) => [n.id, n]));

const edgeEnds = (e: WorkflowEdge, byId: ReadonlyMap<string, WorkflowNode>) => {
  const source = byId.get(e.source);
  const target = byId.get(e.target);
  return source && target ? { source, target } : null;
};

const byPosition = (a: WorkflowNode, b: WorkflowNode) =>
  a.position.x - b.position.x || a.position.y - b.position.y || byCodeUnit(a.id, b.id);

export function topoSort(wf: Workflow): {
  order: string[];
  sorted: WorkflowNode[];
  cyclic: string[];
} {
  const nodes = wf.nodes.filter((n) => !isStructural(n));
  const byId = nodesById(nodes);
  const indegree = new Map(nodes.map((n) => [n.id, 0]));
  const next = new Map(nodes.map((n): [string, WorkflowNode[]] => [n.id, []]));
  for (const e of wf.edges) {
    const ends = edgeEnds(e, byId);
    if (!ends) continue;
    next.get(ends.source.id)?.push(ends.target);
    indegree.set(ends.target.id, (indegree.get(ends.target.id) ?? 0) + 1);
  }
  const ready = nodes.filter((n) => indegree.get(n.id) === 0).sort(byPosition);
  const sorted: WorkflowNode[] = [];
  for (let node = ready.shift(); node; node = ready.shift()) {
    sorted.push(node);
    for (const after of next.get(node.id) ?? []) {
      const left = (indegree.get(after.id) ?? 0) - 1;
      indegree.set(after.id, left);
      if (left === 0) ready.push(after);
    }
    ready.sort(byPosition);
  }
  const ordered = new Set(sorted);
  return {
    order: sorted.map((n) => n.id),
    sorted,
    cyclic: nodes.filter((n) => !ordered.has(n)).map((n) => n.id),
  };
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
  const byId = nodesById(wf.nodes);
  const out = new Set<string>();
  for (const node of wf.nodes) {
    if (getNodeDef(node.type)?.phase !== "render") continue;
    for (const id of upstreamOf(wf, node.id)) {
      const upstream = byId.get(id);
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
  const node = findById(wf.nodes, nodeId);
  const def = node ? getNodeDef(node.type) : undefined;
  return def ? (side === "in" ? def.inputs : def.outputs)[handle] : undefined;
}

const portMismatch = (from: PortType, to: PortType) =>
  `A ${PORT_WORDS[from]} can't connect to a ${PORT_WORDS[to]} input`;

const portList = (ports: Record<string, PortSpec> | undefined) => {
  const ids = Object.keys(ports ?? {});
  return ids.length ? ids.join(", ") : "none";
};

export function connectionProblem(wf: Workflow, c: ConnectionEnds): ConnectionProblem | null {
  const source = findById(wf.nodes, c.source);
  const target = findById(wf.nodes, c.target);
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
    return { notFound: false, message: `${portMismatch(from.type, to.type)}.` };
  return null;
}

export function runTargets(wf: Workflow): string[] {
  const executes = (n: WorkflowNode | undefined) =>
    !!n && !n.disabled && !isStructural(n) && !!getNodeDef(n.type)?.executes;
  const byId = nodesById(wf.nodes);
  const outputs = wf.nodes.filter((n) => executes(n) && isOutputType(n.type));
  if (!outputs.length) return wf.nodes.filter(executes).map((n) => n.id);
  const targets = new Set(outputs.map((n) => n.id));
  for (const out of outputs)
    for (const id of downstreamOf(wf, out.id)) if (executes(byId.get(id))) targets.add(id);
  return [...targets];
}

export function sessionChain(wf: Workflow, captureNodeId: string): WorkflowNode[] {
  const byId = nodesById(wf.nodes);
  const chain: WorkflowNode[] = [];
  const seen = new Set<string>();
  for (let current = captureNodeId; ;) {
    const edge = wf.edges.find((e) => e.target === current && e.targetHandle === "session");
    if (!edge) break;
    if (seen.has(edge.source)) throw new Error("The browser action chain loops back on itself.");
    seen.add(edge.source);
    const node = byId.get(edge.source);
    if (!node) throw new Error(`Missing node ${edge.source}`);
    chain.unshift(node);
    if (node.type === "browser.session") return chain;
    current = node.id;
  }
  const head = chain[0];
  throw new Error(
    head
      ? `The action chain starting at "${nodeName(head)}" is not connected to a Session node.`
      : "Connect a Session → actions chain to this node's session input.",
  );
}

function sessionNodeOf(wf: Workflow, captureNodeId: string): WorkflowNode | undefined {
  try {
    return sessionChain(wf, captureNodeId)[0];
  } catch {
    return undefined;
  }
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
  const session = sessionNodeOf(wf, captureNodeId);
  const storageState = session ? param(session, "storageState") : "";
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
    const parsed = parseParams(def.params, node.params);
    if (!parsed.ok) issues.push({ level: "error", nodeId: node.id, message: parsed.problem });
    return [...issues, ...portIssues(wf, node, def.inputs)];
  });
}

function edgeIssue(e: WorkflowEdge, byId: ReadonlyMap<string, WorkflowNode>): GraphIssue | null {
  const ends = edgeEnds(e, byId);
  if (!ends)
    return { level: "error", edgeId: e.id, message: "A connection points at a deleted node" };
  const t = ends.target;
  const sp = getNodeDef(ends.source.type)?.outputs[e.sourceHandle];
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
      message: portMismatch(sp.type, tp.type),
    };
  return null;
}

function edgeIssues(wf: Workflow): GraphIssue[] {
  const byId = nodesById(wf.nodes);
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
