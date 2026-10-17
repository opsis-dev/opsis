import { defaultParams, getNodeDef } from "../nodes/registry";
import { CARD_WIDTH, estimateCardHeight } from "../nodes/size";
import {
  DEFAULT_WORKFLOW_SETTINGS,
  type Workflow,
  type WorkflowEdge,
  type WorkflowNode,
  type WorkflowSettings,
} from "../schema/workflow";

const GAP = 32;
const COLUMN_WIDTH = 300;
const ROW_HEIGHT = 170;
const CLIP_PORT = "clip";
const SESSION_PORT = "session";

type Placed = { node: WorkflowNode; height: number };

const overlaps = (p: Placed, x: number, y: number, height: number) =>
  Math.abs(p.node.position.x - x) < CARD_WIDTH + GAP &&
  y < p.node.position.y + p.height + GAP &&
  y + height + GAP > p.node.position.y;

function nextFreeY(placed: Placed[], x: number, y: number, height: number): number {
  const below = placed.find((p) => overlaps(p, x, y, height));
  return below ? nextFreeY(placed, x, below.node.position.y + below.height + GAP, height) : y;
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
    this.edges.push({
      id: `${source}.${sourceHandle}->${target}.${targetHandle}`,
      source,
      sourceHandle,
      target,
      targetHandle,
    });
    return this;
  }

  private unstack(): void {
    const placed: Placed[] = [];
    const order = [...this.nodes].sort(
      (a, b) => a.position.y - b.position.y || a.position.x - b.position.x,
    );
    for (const node of order) {
      const height = estimateCardHeight(getNodeDef(node.type));
      const y = nextFreeY(placed, node.position.x, node.position.y, height);
      node.position = { ...node.position, y };
      placed.push({ node, height });
    }
  }

  chain(ids: string[]): this {
    for (let i = 1; i < ids.length; i++) this.link(ids[i - 1], ids[i], SESSION_PORT, SESSION_PORT);
    return this;
  }

  build(meta: {
    id: string;
    name: string;
    settings?: Partial<WorkflowSettings>;
    variables?: Record<string, string>;
  }): Workflow {
    this.unstack();
    return {
      version: 1,
      id: meta.id,
      name: meta.name,
      settings: { ...DEFAULT_WORKFLOW_SETTINGS, ...meta.settings },
      variables: meta.variables ?? {},
      nodes: this.nodes,
      edges: this.edges,
      markers: [],
    };
  }
}
