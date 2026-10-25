import { compileWorkflow, type CompileResult } from "./compile/compile";
import { nonBlankString } from "./guards";
import { byCodeUnit, findById } from "./lists";
import { ACTION_NODE_TYPES, navigateNode, setStateNode, signInNode } from "./nodes/browser";
import type { PortSpec } from "./nodes/define";
import {
  exportBaseName,
  renderNode,
  stillFileNames,
  stillOutputNode,
  videoFileNames,
} from "./nodes/output";
import { parseParams } from "./nodes/params";
import { canConnect, portLabel, SESSION_PORT } from "./nodes/ports";
import { getNodeDef, isOutputType, nodeName } from "./nodes/registry";
import { GROUP_TYPE, NOTE_TYPE } from "./nodes/utility";
import { asValue, type BrowserSessionValue } from "./schema/values";
import {
  PORT_WORDS,
  type EdgeEnds,
  type PortType,
  type Workflow,
  type WorkflowEdge,
  type WorkflowNode,
} from "./schema/workflow";
import { secretFileName, secretName } from "./secrets";
import { interpolate, unknownVariables, workflowVars } from "./vars";

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

export function sessionInto(
  wf: Workflow,
  values: CompileResult["values"],
  nodeId: string,
): BrowserSessionValue | undefined {
  const edge = wf.edges.find((e) => e.target === nodeId && e.targetHandle === SESSION_PORT);
  return edge && asValue(values[edge.source]?.[edge.sourceHandle], "browser-session");
}

function portIssues(
  wf: Workflow,
  node: WorkflowNode,
  inputs: Record<string, PortSpec>,
): GraphIssue[] {
  const issues: GraphIssue[] = [];
  const byId = nodesById(wf.nodes);
  for (const [portId, spec] of Object.entries(inputs)) {
    const edges = wf.edges.filter(
      (e) => e.target === node.id && e.targetHandle === portId && !edgeFault(e, byId),
    );
    const name = portLabel(spec);
    if (edges.length === 0 && !spec.optional)
      issues.push({ level: "error", nodeId: node.id, message: `Connect something to “${name}”` });
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
  const missingVar = unknownVariables(node.params, workflowVars(wf))[0];
  const paramIssues: GraphIssue[] = [
    ...(parsed.ok ? [] : [{ level: "error" as const, nodeId: node.id, message: parsed.problem }]),
    ...(missingVar === undefined
      ? []
      : [
          {
            level: "error" as const,
            nodeId: node.id,
            message: `“${missingVar}” isn't one of the project's variables. Add it under Variables in the project settings, or change this setting.`,
          },
        ]),
  ];
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

const anchorIssues = (wf: Workflow): GraphIssue[] => {
  const actions = new Set(wf.nodes.filter((n) => ACTION_NODE_TYPES.has(n.type)).map((n) => n.id));
  return wf.nodes
    .filter((n) => !n.disabled && typeof n.params.atAction === "string" && n.params.atAction)
    .filter((n) => !actions.has(String(n.params.atAction)))
    .map((n) => ({
      level: "error",
      nodeId: n.id,
      message: "The action this sound plays on is gone. Pick another, or play it at a time.",
    }));
};

function exportFiles(node: WorkflowNode, vars: Record<string, string>): string[] {
  const params = interpolate(node.params, vars);
  if (node.type === renderNode.type) {
    const p = renderNode.params.safeParse(params);
    return p.success
      ? videoFileNames(exportBaseName(p.data.fileName), p.data.codec, p.data.variants)
      : [];
  }
  if (node.type === stillOutputNode.type) {
    const p = stillOutputNode.params.safeParse(params);
    return p.success
      ? stillFileNames(exportBaseName(p.data.fileName), p.data.presets, p.data.format)
      : [];
  }
  return [];
}

function signInClashes(wf: Workflow): GraphIssue[] {
  const vars = workflowVars(wf);
  const saver = new Map<string, WorkflowNode>();
  const issues: GraphIssue[] = [];
  for (const node of wf.nodes.filter((n) => n.type === signInNode.type && !n.disabled)) {
    const name = secretName(nonBlankString(interpolate(node.params.secretName, vars)) ?? "");
    const key = secretFileName(name);
    const first = saver.get(key);
    if (!first) saver.set(key, node);
    else
      issues.push({
        level: "error",
        nodeId: node.id,
        message: `“${nodeName(first)}” also saves the sign-in “${name}”. Give one of them another name.`,
      });
  }
  return issues;
}

function exportClashes(wf: Workflow): GraphIssue[] {
  const vars = workflowVars(wf);
  const writer = new Map<string, WorkflowNode>();
  const issues: GraphIssue[] = [];
  for (const node of wf.nodes.filter((n) => !n.disabled))
    for (const file of exportFiles(node, vars)) {
      const first = writer.get(file);
      if (!first) writer.set(file, node);
      else if (first.id !== node.id)
        issues.push({
          level: "error",
          nodeId: node.id,
          message: `“${nodeName(first)}” also saves “${file}”. Give one of them another file name.`,
        });
    }
  return issues;
}

function passesNothing(wf: Workflow, skipped: WorkflowNode, handle: string): boolean {
  const def = getNodeDef(skipped.type);
  const type = def?.outputs[handle]?.type;
  if (!def || !type) return false;
  const byId = nodesById(wf.nodes);
  return !wf.edges.some(
    (e) =>
      e.target === skipped.id && def.inputs[e.targetHandle]?.type === type && !edgeFault(e, byId),
  );
}

function skippedSourceIssues(wf: Workflow): GraphIssue[] {
  const byId = nodesById(wf.nodes);
  return wf.edges.flatMap((e): GraphIssue[] => {
    const source = byId.get(e.source);
    const target = byId.get(e.target);
    if (!source?.disabled || !target || target.disabled) return [];
    if (!passesNothing(wf, source, e.sourceHandle)) return [];
    return [
      {
        level: "warning",
        nodeId: target.id,
        message: `“${nodeName(source)}” is skipped and has nothing to pass on, so this gets nothing from it.`,
      },
    ];
  });
}

const runsBeforeFirstPage = (
  wf: Workflow,
  values: CompileResult["values"],
  nodeId: string,
): boolean => {
  const session = sessionInto(wf, values, nodeId);
  return !!session && !session.steps.some((step) => step.type === navigateNode.type);
};

const reloadIssues = (wf: Workflow, compiled: CompileResult): GraphIssue[] =>
  wf.nodes
    .filter(
      (n) =>
        n.type === setStateNode.type &&
        !n.disabled &&
        n.params.reload === true &&
        runsBeforeFirstPage(wf, compiled.values, n.id),
    )
    .map((n) => ({
      level: "warning",
      nodeId: n.id,
      message:
        "Before Go to page the script runs as the site first opens, so there's nothing to reload. Turn off Reload, or move this after Go to page.",
    }));

export const validateWorkflow = (
  wf: Workflow,
  compiled: CompileResult = compileWorkflow(wf),
): GraphIssue[] => [
  ...nodeIssues(wf),
  ...edgeIssues(wf),
  ...cycleIssues(wf),
  ...reloadIssues(wf, compiled),
  ...anchorIssues(wf),
  ...exportClashes(wf),
  ...signInClashes(wf),
  ...skippedSourceIssues(wf),
];
