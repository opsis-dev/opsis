import { edgeId, randomId } from "../ids";
import { CLIP_PORT } from "../nodes/define";
import { SESSION_PORT } from "../nodes/ports";
import { defaultParams, getNodeDef } from "../nodes/registry";
import { CARD_GAP, CARD_WIDTH, COLUMN_WIDTH, estimateCardHeight, ROW_HEIGHT } from "../nodes/size";
import {
  DEFAULT_WORKFLOW_SETTINGS,
  type Workflow,
  type WorkflowEdge,
  type WorkflowNode,
  type WorkflowSettings,
} from "../schema/workflow";

type Placed = { node: WorkflowNode; height: number };

export type TakeStep = {
  id: string;
  type: string;
  params?: Record<string, unknown>;
  label?: string;
};

type TakeOptions = { camera?: Record<string, unknown>; row?: number; prefix?: string };

const overlaps = (p: Placed, x: number, y: number, height: number) =>
  Math.abs(p.node.position.x - x) < CARD_WIDTH + CARD_GAP &&
  y < p.node.position.y + p.height + CARD_GAP &&
  y + height + CARD_GAP > p.node.position.y;

function nextFreeY(placed: Placed[], x: number, y: number, height: number): number {
  const below = placed.find((p) => overlaps(p, x, y, height));
  return below ? nextFreeY(placed, x, below.node.position.y + below.height + CARD_GAP, height) : y;
}

function unstack(nodes: readonly WorkflowNode[]): WorkflowNode[] {
  const placed: Placed[] = [];
  const moved = new Map<WorkflowNode, WorkflowNode>();
  const order = [...nodes].sort(
    (a, b) => a.position.y - b.position.y || a.position.x - b.position.x,
  );
  for (const node of order) {
    const height = estimateCardHeight(getNodeDef(node.type));
    const y = nextFreeY(placed, node.position.x, node.position.y, height);
    const next = { ...node, position: { ...node.position, y } };
    moved.set(node, next);
    placed.push({ node: next, height });
  }
  return nodes.map((n) => moved.get(n) ?? n);
}

export class GraphBuilder {
  nodes: WorkflowNode[] = [];
  edges: WorkflowEdge[] = [];

  node(
    id: string,
    type: string,
    x: number,
    y: number,
    params: Record<string, unknown> = {},
    label?: string,
  ): string {
    this.nodes.push({
      id,
      type,
      position: { x: x * COLUMN_WIDTH, y: y * ROW_HEIGHT },
      params: { ...defaultParams(type), ...params },
      ...(label ? { label } : {}),
    });
    return id;
  }

  link(source: string, target: string, sourceHandle = CLIP_PORT, targetHandle = CLIP_PORT): this {
    const ends = { source, sourceHandle, target, targetHandle };
    this.edges.push({ id: edgeId(ends), ...ends });
    return this;
  }

  clipsInto(target: string, sources: string[]): this {
    sources.forEach((source, i) => this.link(source, target, CLIP_PORT, `in${i + 1}`));
    return this;
  }

  chain(ids: string[]): this {
    for (let i = 1; i < ids.length; i++) this.link(ids[i - 1], ids[i], SESSION_PORT, SESSION_PORT);
    return this;
  }

  recordTake(steps: TakeStep[], { camera = {}, row = 0, prefix = "" }: TakeOptions = {}): string {
    const ids = steps.map((step, x) =>
      this.node(`${prefix}${step.id}`, step.type, x, row, step.params, step.label),
    );
    const rec = this.node(`${prefix}record`, "capture.record", steps.length, row);
    this.chain([...ids, rec]);
    const cam = this.node(`${prefix}camera`, "camera.autoZoom", steps.length, row + 1, camera);
    this.link(rec, cam, "recording", "recording");
    return cam;
  }

  build(meta: {
    id?: string;
    name: string;
    settings?: Partial<WorkflowSettings>;
    variables?: Record<string, string>;
  }): Workflow {
    return {
      version: 1,
      id: meta.id ?? randomId("wf"),
      name: meta.name,
      settings: { ...DEFAULT_WORKFLOW_SETTINGS, ...meta.settings },
      variables: meta.variables ?? {},
      nodes: unstack(this.nodes),
      edges: [...this.edges],
      markers: [],
    };
  }
}
