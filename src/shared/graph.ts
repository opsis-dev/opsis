import { humanize } from "./format";
import { byCodeUnit, findById } from "./lists";
import { sessionNode } from "./nodes/browser";
import { loginOnceNode } from "./nodes/capture";
import type { PortSpec } from "./nodes/define";
import { parseParams } from "./nodes/params";
import { canConnect, SESSION_PORT } from "./nodes/ports";
import { getNodeDef, isOutputType, nodeName } from "./nodes/registry";
import { GROUP_TYPE, NOTE_TYPE } from "./nodes/utility";
import {
  PORT_WORDS,
  type EdgeEnds,
  type PortType,
  type Workflow,
  type WorkflowEdge,
  type WorkflowNode,
} from "./schema/workflow";
import { secretFileName } from "./secrets";
import { interpolate } from "./vars";

export type GraphIssue = {
  level: "error" | "warning";
  message: string;
  nodeId?: string;
  edgeId?: string;
};

export const isGroupNode = (node: Pick<WorkflowNode, "type">) => node.type === GROUP_TYPE;

export const isNoteNode = (node: Pick<WorkflowNode, "type">) => node.type === NOTE_TYPE;

export const isStructural = (node: WorkflowNode) => isGroupNode(node) || isNoteNode(node);

const nodesById = (nodes: readonly WorkflowNode[]): Map<string, WorkflowNode> =>
  new Map(nodes.map((n) => [n.id, n]));

const byPosition = (a: WorkflowNode, b: WorkflowNode) =>
  a.position.x - b.position.x || a.position.y - b.position.y || byCodeUnit(a.id, b.id);

type Vertex = { node: WorkflowNode; indegree: number; next: Vertex[] };

export function topoSort(wf: Workflow): { sorted: WorkflowNode[]; cyclic: string[] } {
  const nodes = wf.nodes.filter((n) => !isStructural(n));
  const vertices = nodes.map((node): Vertex => ({ node, indegree: 0, next: [] }));
  const byId = new Map(vertices.map((v) => [v.node.id, v]));
  for (const e of wf.edges) {
    const source = byId.get(e.source);
    const target = byId.get(e.target);
    if (!source || !target) continue;
    source.next.push(target);
    target.indegree++;
  }
  const byVertexPosition = (a: Vertex, b: Vertex) => byPosition(a.node, b.node);
  const ready = vertices.filter((v) => v.indegree === 0).sort(byVertexPosition);
  const sorted: WorkflowNode[] = [];
  for (let vertex = ready.shift(); vertex; vertex = ready.shift()) {
    sorted.push(vertex.node);
    for (const after of vertex.next) if (--after.indegree === 0) ready.push(after);
    ready.sort(byVertexPosition);
  }
  const ordered = new Set(sorted);
  return { sorted, cyclic: nodes.filter((n) => !ordered.has(n)).map((n) => n.id) };
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

type EdgeFault =
  | { fault: "node"; id: string }
  | { fault: "output" | "input"; node: WorkflowNode; handle: string }
  | { fault: "types"; from: PortType; to: PortType };

function edgeFault(c: EdgeEnds, byId: ReadonlyMap<string, WorkflowNode>): EdgeFault | null {
  const source = byId.get(c.source);
  if (!source) return { fault: "node", id: c.source };
  const target = byId.get(c.target);
  if (!target) return { fault: "node", id: c.target };
  const from = getNodeDef(source.type)?.outputs[c.sourceHandle];
  if (!from) return { fault: "output", node: source, handle: c.sourceHandle };
  const to = getNodeDef(target.type)?.inputs[c.targetHandle];
  if (!to) return { fault: "input", node: target, handle: c.targetHandle };
  return canConnect(from.type, to.type) ? null : { fault: "types", from: from.type, to: to.type };
}

function missingPortMessage(fault: Extract<EdgeFault, { fault: "output" | "input" }>): string {
  const def = getNodeDef(fault.node.type);
  const side = fault.fault;
  const ports = side === "output" ? def?.outputs : def?.inputs;
  return `“${fault.node.id}” has no ${side} “${fault.handle}”. Its ${side}s: ${portList(ports)}.`;
}

export function connectionProblem(wf: Workflow, c: EdgeEnds): ConnectionProblem | null {
  const fault = edgeFault(c, nodesById(wf.nodes));
  if (fault?.fault === "node") return { notFound: true, message: `There's no node “${fault.id}”.` };
  if (c.source === c.target) return { notFound: false, message: "A node can't connect to itself." };
  if (!fault) return null;
  const message =
    fault.fault === "types" ? `${portMismatch(fault.from, fault.to)}.` : missingPortMessage(fault);
  return { notFound: false, message };
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

type SessionWalk = { ok: true; chain: WorkflowNode[] } | { ok: false; problem: string };

function walkSessionChain(wf: Workflow, captureNodeId: string): SessionWalk {
  const byId = nodesById(wf.nodes);
  const chain: WorkflowNode[] = [];
  const seen = new Set<string>();
  const feedingEdge = (id: string) =>
    wf.edges.find((e) => e.target === id && e.targetHandle === SESSION_PORT);
  for (let edge = feedingEdge(captureNodeId); edge; edge = feedingEdge(edge.source)) {
    if (seen.has(edge.source))
      return { ok: false, problem: "The browser action chain loops back on itself." };
    seen.add(edge.source);
    const node = byId.get(edge.source);
    if (!node) return { ok: false, problem: `Missing node ${edge.source}` };
    chain.unshift(node);
    if (node.type === sessionNode.type) return { ok: true, chain };
  }
  const head = chain[0];
  return {
    ok: false,
    problem: head
      ? `The action chain starting at "${nodeName(head)}" is not connected to a Session node.`
      : "Connect a Session → actions chain to this node's session input.",
  };
}

export function sessionChain(wf: Workflow, captureNodeId: string): WorkflowNode[] {
  const walk = walkSessionChain(wf, captureNodeId);
  if (!walk.ok) throw new Error(walk.problem);
  return walk.chain;
}

export function loginDependencies(
  wf: Workflow,
  captureNodeId: string,
  vars: Record<string, string>,
): string[] {
  const param = (node: WorkflowNode, key: string) => {
    const value = node.params[key];
    return typeof value === "string" ? interpolate(value, vars).trim() : "";
  };
  const walk = walkSessionChain(wf, captureNodeId);
  const storageState = walk.ok ? param(walk.chain[0], "storageState") : "";
  if (!storageState) return [];
  const secret = secretFileName(storageState);
  return wf.nodes
    .filter(
      (n) =>
        n.type === loginOnceNode.type &&
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

function repeatedNodes(nodes: readonly WorkflowNode[]): Set<WorkflowNode> {
  const seen = new Set<string>();
  const repeated = new Set<WorkflowNode>();
  for (const node of nodes) {
    if (seen.has(node.id)) repeated.add(node);
    seen.add(node.id);
  }
  return repeated;
}

function definitionIssues(wf: Workflow, node: WorkflowNode): GraphIssue[] {
  const def = getNodeDef(node.type);
  if (!def) return [{ level: "error", nodeId: node.id, message: "This node type isn't available" }];
  if (node.disabled) return [];
  const parsed = parseParams(def.params, node.params);
  const paramIssues: GraphIssue[] = parsed.ok
    ? []
    : [{ level: "error", nodeId: node.id, message: parsed.problem }];
  return isStructural(node) ? paramIssues : [...paramIssues, ...portIssues(wf, node, def.inputs)];
}

function nodeIssues(wf: Workflow): GraphIssue[] {
  const repeated = repeatedNodes(wf.nodes);
  return wf.nodes.flatMap((node): GraphIssue[] => [
    ...(repeated.has(node)
      ? [{ level: "error" as const, nodeId: node.id, message: "Two nodes share this id" }]
      : []),
    ...definitionIssues(wf, node),
  ]);
}

function edgeIssue(e: WorkflowEdge, byId: ReadonlyMap<string, WorkflowNode>): GraphIssue | null {
  const fault = edgeFault(e, byId);
  if (!fault) return null;
  if (fault.fault === "node")
    return { level: "error", edgeId: e.id, message: "A connection points at a deleted node" };
  const message =
    fault.fault === "types"
      ? portMismatch(fault.from, fault.to)
      : "A connection uses a point that no longer exists";
  return { level: "error", edgeId: e.id, nodeId: e.target, message };
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
