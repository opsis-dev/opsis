import { defaultParams, getNodeDef, normalizeNode } from "../nodes/registry";
import { CARD_WIDTH, estimateCardHeight } from "../nodes/size";
import type { Workflow, WorkflowEdge, WorkflowNode, WorkflowSettings } from "../schema/workflow";

const GAP = 32;

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
    this.nodes.push(
      normalizeNode({
        id,
        type,
        position: { x: x * 300, y: y * 170 },
        params: { ...defaultParams(type), ...params },
        ...(label ? { label } : {}),
      }),
    );
    return id;
  }

  link(source: string, target: string, sourceHandle = "clip", targetHandle = "clip"): this {
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
    const placed: Array<{ node: WorkflowNode; height: number }> = [];
    const order = [...this.nodes].sort(
      (a, b) => a.position.y - b.position.y || a.position.x - b.position.x,
    );
    for (const node of order) {
      const height = estimateCardHeight(getNodeDef(node.type));
      let y = node.position.y;
      for (let moved = true; moved;) {
        moved = false;
        for (const p of placed) {
          const sideBySide = Math.abs(p.node.position.x - node.position.x) >= CARD_WIDTH + GAP;
          const clear =
            y >= p.node.position.y + p.height + GAP || y + height + GAP <= p.node.position.y;
          if (!sideBySide && !clear) {
            y = p.node.position.y + p.height + GAP;
            moved = true;
          }
        }
      }
      node.position = { ...node.position, y };
      placed.push({ node, height });
    }
  }

  chain(ids: string[]): this {
    for (let i = 1; i < ids.length; i++) this.link(ids[i - 1], ids[i], "session", "session");
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
      settings: { width: 1920, height: 1080, fps: 30, theme: "dark", ...meta.settings },
      variables: meta.variables ?? {},
      nodes: this.nodes,
      edges: this.edges,
    };
  }
}
