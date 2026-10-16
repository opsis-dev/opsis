import { getNodeDef } from "./nodes/registry";
import { canConnect } from "./nodes/ports";
import { fieldMeta } from "./elements/introspect";
import type { PortType, Workflow, WorkflowEdge, WorkflowNode } from "./schema/workflow";
import { interpolate } from "./vars";

export type GraphIssue = {
  level: "error" | "warning";
  message: string;
  nodeId?: string;
  edgeId?: string;
};

export const isStructural = (node: WorkflowNode) =>
  node.type === "core.group" || node.type === "core.note";

export function incomingEdges(wf: Workflow, nodeId: string): WorkflowEdge[] {
  return wf.edges.filter((e) => e.target === nodeId);
}

export function outgoingEdges(wf: Workflow, nodeId: string): WorkflowEdge[] {
  return wf.edges.filter((e) => e.source === nodeId);
}

export function topoSort(wf: Workflow): { order: string[]; cyclic: string[] } {
  const nodes = wf.nodes.filter((n) => !isStructural(n));
  const ids = new Set(nodes.map((n) => n.id));
  const indegree = new Map<string, number>(nodes.map((n) => [n.id, 0]));
  const adj = new Map<string, string[]>(nodes.map((n) => [n.id, []]));
  for (const e of wf.edges) {
    if (!ids.has(e.source) || !ids.has(e.target)) continue;
    adj.get(e.source)!.push(e.target);
    indegree.set(e.target, (indegree.get(e.target) ?? 0) + 1);
  }
  const pos = new Map(nodes.map((n) => [n.id, n.position]));
  const byPosition = (a: string, b: string) => {
    const pa = pos.get(a)!;
    const pb = pos.get(b)!;
    return pa.x - pb.x || pa.y - pb.y || (a < b ? -1 : 1);
  };
  const ready = nodes.filter((n) => indegree.get(n.id) === 0).map((n) => n.id);
  ready.sort(byPosition);
  const order: string[] = [];
  while (ready.length) {
    const id = ready.shift()!;
    order.push(id);
    for (const next of adj.get(id)!) {
      const d = indegree.get(next)! - 1;
      indegree.set(next, d);
      if (d === 0) {
        ready.push(next);
        ready.sort(byPosition);
      }
    }
  }
  const cyclic = nodes.map((n) => n.id).filter((id) => !order.includes(id));
  return { order, cyclic };
}

export function upstreamOf(wf: Workflow, nodeId: string): Set<string> {
  const out = new Set<string>();
  const stack = [nodeId];
  while (stack.length) {
    const id = stack.pop()!;
    for (const e of wf.edges) {
      if (e.target === id && !out.has(e.source)) {
        out.add(e.source);
        stack.push(e.source);
      }
    }
  }
  return out;
}

export function downstreamOf(wf: Workflow, nodeId: string): Set<string> {
  const out = new Set<string>();
  const stack = [nodeId];
  while (stack.length) {
    const id = stack.pop()!;
    for (const e of wf.edges) {
      if (e.source === id && !out.has(e.target)) {
        out.add(e.target);
        stack.push(e.target);
      }
    }
  }
  return out;
}

export function runTargets(wf: Workflow): string[] {
  const executes = (n: WorkflowNode | undefined) =>
    !!n && !n.disabled && !isStructural(n) && !!getNodeDef(n.type)?.executes;
  const byId = new Map(wf.nodes.map((n) => [n.id, n]));
  const outputs = wf.nodes.filter((n) => executes(n) && n.type.startsWith("output."));
  if (!outputs.length) return wf.nodes.filter(executes).map((n) => n.id);
  const targets = new Set(outputs.map((n) => n.id));
  for (const out of outputs)
    for (const id of downstreamOf(wf, out.id)) if (executes(byId.get(id))) targets.add(id);
  return [...targets];
}

export function sessionChain(wf: Workflow, captureNodeId: string): WorkflowNode[] {
  const byId = new Map(wf.nodes.map((n) => [n.id, n]));
  const chain: WorkflowNode[] = [];
  let current = captureNodeId;
  const seen = new Set<string>();
  for (;;) {
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
      ? `The action chain starting at "${head.label ?? head.type}" is not connected to a Session node.`
      : "Connect a Session → actions chain to this node's session input.",
  );
}

export function loginDependencies(
  wf: Workflow,
  captureNodeId: string,
  vars: Record<string, string>,
): string[] {
  const param = (node: WorkflowNode, key: string) => {
    const value = (interpolate(node.params ?? {}, vars) as Record<string, unknown>)[key];
    return typeof value === "string" ? value.trim() : "";
  };
  let head: WorkflowNode | undefined;
  try {
    head = sessionChain(wf, captureNodeId)[0];
  } catch {
    return [];
  }
  const secret = head ? param(head, "storageState") : "";
  if (!secret) return [];
  return wf.nodes
    .filter(
      (n) =>
        n.type === "capture.loginOnce" &&
        !n.disabled &&
        (param(n, "secretName") || "default") === secret,
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

const humanKey = (key: string) =>
  key.replace(/([a-z])([A-Z])/g, "$1 $2").replace(/^./, (c) => c.toUpperCase());

type ZodIssueLike = {
  code: string;
  message: string;
  path: PropertyKey[];
  minimum?: unknown;
  maximum?: unknown;
};

export function describeParamIssue(shape: Record<string, unknown>, issue: ZodIssueLike): string {
  const key = String(issue.path[0] ?? "");
  const field = shape[key];
  const label =
    (field ? fieldMeta(field as Parameters<typeof fieldMeta>[0]).description : undefined) ||
    (key ? humanKey(key) : "A setting");
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

export function validateWorkflow(wf: Workflow): GraphIssue[] {
  const issues: GraphIssue[] = [];
  const byId = new Map(wf.nodes.map((n) => [n.id, n]));

  for (const node of wf.nodes) {
    if (isStructural(node)) continue;
    const def = getNodeDef(node.type);
    if (!def) {
      issues.push({ level: "error", nodeId: node.id, message: "This node type isn't available" });
      continue;
    }
    if (node.disabled) continue;
    const parsed = def.params.safeParse(node.params ?? {});
    if (!parsed.success) {
      const first = parsed.error.issues[0];
      issues.push({
        level: "error",
        nodeId: node.id,
        message: describeParamIssue(def.params.shape, first as ZodIssueLike),
      });
    }
    for (const [portId, spec] of Object.entries(def.inputs)) {
      const edges = wf.edges.filter((e) => e.target === node.id && e.targetHandle === portId);
      if (edges.length === 0 && !spec.optional) {
        issues.push({
          level: "warning",
          nodeId: node.id,
          message: `Connect something to “${spec.label ?? humanKey(portId)}”`,
        });
      }
      if (edges.length > 1 && !spec.many) {
        issues.push({
          level: "error",
          nodeId: node.id,
          message: `“${spec.label ?? humanKey(portId)}” can only take one connection`,
        });
      }
    }
  }

  for (const e of wf.edges) {
    const s = byId.get(e.source);
    const t = byId.get(e.target);
    if (!s || !t) {
      issues.push({
        level: "error",
        edgeId: e.id,
        message: "A connection points at a deleted node",
      });
      continue;
    }
    const sp = getNodeDef(s.type)?.outputs[e.sourceHandle];
    const tp = getNodeDef(t.type)?.inputs[e.targetHandle];
    if (!sp || !tp) {
      issues.push({
        level: "error",
        edgeId: e.id,
        nodeId: t.id,
        message: "A connection uses a point that no longer exists",
      });
      continue;
    }
    if (!canConnect(sp.type, tp.type)) {
      issues.push({
        level: "error",
        edgeId: e.id,
        nodeId: t.id,
        message: `A ${TYPE_WORDS[sp.type]} can't connect to a ${TYPE_WORDS[tp.type]} input`,
      });
    }
  }

  const { cyclic } = topoSort(wf);
  for (const id of cyclic)
    issues.push({ level: "error", nodeId: id, message: "This node loops back into itself" });
  return issues;
}
