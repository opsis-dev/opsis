import { sameFamily } from "./fonts";
import type { FontFamilyDef } from "./fonts/types";
import { isGroupNode } from "./graph";
import { edgeId, nodeIdFor } from "./ids";
import { defaultParams, getNodeDef } from "./nodes/registry";
import { GROUP_TYPE } from "./nodes/utility";
import {
  normalizeWorkflow,
  type EdgeEnds,
  type Workflow,
  type WorkflowEdge,
  type WorkflowNode,
} from "./schema/workflow";
import { renameVariable } from "./vars";
import { findById } from "./lists";

type Point = { x: number; y: number };
type Size = { width: number; height: number };

export const GROUP_PAD = 40;
export const GROUP_TITLE_HEIGHT = 24;

export function absPosition(wf: Workflow, node: WorkflowNode): Point {
  if (!node.parentId) return node.position;
  const parent = findById(wf.nodes, node.parentId);
  if (!parent) return node.position;
  const p = absPosition(wf, parent);
  return { x: p.x + node.position.x, y: p.y + node.position.y };
}

function detachFromParent(node: WorkflowNode, parent: Point | undefined): WorkflowNode {
  const { parentId: _parentId, ...rest } = node;
  return {
    ...rest,
    position: { x: node.position.x + (parent?.x ?? 0), y: node.position.y + (parent?.y ?? 0) },
  };
}

export function removeNodes(wf: Workflow, ids: readonly string[]): Workflow {
  const gone = new Set(ids);
  const positions = new Map(wf.nodes.map((n) => [n.id, n.position]));
  return normalizeWorkflow({
    ...wf,
    nodes: wf.nodes
      .filter((n) => !gone.has(n.id))
      .map((n) =>
        n.parentId && gone.has(n.parentId) ? detachFromParent(n, positions.get(n.parentId)) : n,
      ),
    edges: wf.edges.filter((e) => !gone.has(e.source) && !gone.has(e.target)),
  });
}

const sameEnds = (a: EdgeEnds, b: EdgeEnds) =>
  a.source === b.source &&
  a.sourceHandle === b.sourceHandle &&
  a.target === b.target &&
  a.targetHandle === b.targetHandle;

export function connect(wf: Workflow, edge: EdgeEnds): Workflow {
  const target = findById(wf.nodes, edge.target);
  const many = !!(target && getNodeDef(target.type)?.inputs[edge.targetHandle]?.many);
  const replaced = (e: WorkflowEdge) =>
    many ? sameEnds(e, edge) : e.target === edge.target && e.targetHandle === edge.targetHandle;
  return {
    ...wf,
    edges: [...wf.edges.filter((e) => !replaced(e)), { ...edge, id: edgeId(edge) }],
  };
}

export function pasteNodes(
  wf: Workflow,
  nodes: readonly WorkflowNode[],
  edges: readonly WorkflowEdge[],
  offset: Point,
): { workflow: Workflow; ids: string[] } {
  const copies = nodes.map((node) => ({ node, id: nodeIdFor(node.type) }));
  const ids = new Map(copies.map((c) => [c.node.id, c.id]));
  const fresh = copies.map(({ node, id }): WorkflowNode => {
    const { parentId: copiedParent, ...rest } = node;
    const parentId = copiedParent && ids.get(copiedParent);
    if (parentId) return { ...rest, id, parentId };
    const at = absPosition(wf, node);
    return { ...rest, id, position: { x: at.x + offset.x, y: at.y + offset.y } };
  });
  const freshEdges = edges.flatMap((e) => {
    const source = ids.get(e.source);
    const target = ids.get(e.target);
    if (!source || !target) return [];
    const ends = { ...e, source, target };
    return [{ ...ends, id: edgeId(ends) }];
  });
  return {
    workflow: { ...wf, nodes: [...wf.nodes, ...fresh], edges: [...wf.edges, ...freshEdges] },
    ids: fresh.map((n) => n.id),
  };
}

export function groupNodes(
  wf: Workflow,
  ids: readonly string[],
  group: { id: string; title: string; sizeOf: (node: WorkflowNode) => Size },
): Workflow {
  const members = wf.nodes.filter((n) => ids.includes(n.id) && !n.parentId && !isGroupNode(n));
  if (members.length < 2) return wf;
  const x = Math.min(...members.map((n) => n.position.x)) - GROUP_PAD;
  const y = Math.min(...members.map((n) => n.position.y)) - GROUP_PAD - GROUP_TITLE_HEIGHT;
  const right = Math.max(...members.map((n) => n.position.x + group.sizeOf(n).width));
  const bottom = Math.max(...members.map((n) => n.position.y + group.sizeOf(n).height));
  const memberIds = new Set(members.map((m) => m.id));
  return {
    ...wf,
    nodes: [
      {
        id: group.id,
        type: GROUP_TYPE,
        position: { x, y },
        params: { ...defaultParams(GROUP_TYPE), title: group.title },
        size: { width: right - x + GROUP_PAD, height: bottom - y + GROUP_PAD },
      },
      ...wf.nodes.map((n) =>
        memberIds.has(n.id)
          ? { ...n, parentId: group.id, position: { x: n.position.x - x, y: n.position.y - y } }
          : n,
      ),
    ],
  };
}

export function ungroup(wf: Workflow, groupId: string): Workflow {
  const group = findById(wf.nodes, groupId);
  if (!group) return wf;
  return {
    ...wf,
    nodes: wf.nodes
      .filter((n) => n.id !== groupId)
      .map((n) => (n.parentId === groupId ? detachFromParent(n, group.position) : n)),
  };
}

export const addProjectFont = (wf: Workflow, def: FontFamilyDef): Workflow => ({
  ...wf,
  settings: {
    ...wf.settings,
    fonts: {
      ...wf.settings.fonts,
      families: [
        ...wf.settings.fonts.families.filter((f) => !sameFamily(f.family, def.family)),
        def,
      ],
    },
  },
});

export function renameWorkflowVariable(wf: Workflow, from: string, to: string): Workflow {
  const name = to.trim();
  if (!name || name === from || name in wf.variables || !(from in wf.variables)) return wf;
  return {
    ...wf,
    variables: Object.fromEntries(
      Object.entries(wf.variables).map(([k, v]) => [k === from ? name : k, v]),
    ),
    nodes: wf.nodes.map((n) => ({ ...n, params: renameVariable(n.params, from, name) })),
  };
}
