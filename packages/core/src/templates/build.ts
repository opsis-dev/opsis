import { edgeId, randomId } from "#core/ids";
import { navigateNode, startRecordingNode, waitNode } from "#core/nodes/browser";
import { CLIP_PORT, RECORDING_PORT, STYLE_PORT } from "#core/nodes/define";
import { SESSION_PORT } from "#core/nodes/ports";
import { defaultParams, getNodeDef } from "#core/nodes/registry";
import {
  CARD_GAP,
  CARD_WIDTH,
  COLUMN_WIDTH,
  estimateCardHeight,
  ROW_HEIGHT,
} from "#core/nodes/size";
import {
  DEFAULT_WORKFLOW_SETTINGS,
  type Workflow,
  type WorkflowEdge,
  type WorkflowNode,
  type WorkflowSettings,
} from "#core/schema/workflow";

type Placed = { node: WorkflowNode; height: number };

export type TakeStep = {
  id: string;
  type: string;
  params?: Record<string, unknown>;
  label?: string;
};

type TakeOptions = {
  camera?: Record<string, unknown>;
  row?: number;
  prefix?: string;
  style?: TakeStep[];
};

function withVideoStart(steps: TakeStep[]): TakeStep[] {
  const page = steps.findIndex((s) => s.type === navigateNode.type);
  if (page < 0) return steps;
  let at = page + 1;
  while (steps[at]?.type === waitNode.type) at++;
  const start = { id: "start", type: startRecordingNode.type };
  return [...steps.slice(0, at), start, ...steps.slice(at)];
}

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

  recordTake(
    steps: TakeStep[],
    { camera = {}, row = 0, prefix = "", style = [] }: TakeOptions = {},
  ): string {
    const ids = withVideoStart(steps).map((step, x) =>
      this.node(`${prefix}${step.id}`, step.type, x, row, step.params, step.label),
    );
    const rec = this.node(`${prefix}record`, "capture.record", ids.length, row);
    this.chain([...ids, rec]);
    const cam = this.node(`${prefix}camera`, "camera.autoZoom", ids.length, row + 1, camera);
    this.link(rec, cam, RECORDING_PORT, RECORDING_PORT);
    const styled = style.map((step, i) =>
      this.node(
        `${prefix}${step.id}`,
        step.type,
        ids.length - style.length + i,
        row + 1,
        step.params,
        step.label,
      ),
    );
    for (const [i, id] of styled.entries())
      this.link(id, styled[i + 1] ?? cam, STYLE_PORT, STYLE_PORT);
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
