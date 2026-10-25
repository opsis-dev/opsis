import { getNodeDef } from "@opsis/core/nodes/registry";
import { CARD_WIDTH, estimateCardHeight } from "@opsis/core/nodes/size";
import type { WorkflowNode } from "@opsis/core/schema/workflow";

function measured(id: string): { width: number; height: number } | null {
  const el = document.querySelector<HTMLElement>(`.react-flow__node[data-id="${CSS.escape(id)}"]`);
  return el && el.offsetWidth && el.offsetHeight
    ? { width: el.offsetWidth, height: el.offsetHeight }
    : null;
}

export const cardSize = (node: Pick<WorkflowNode, "id" | "type">) =>
  measured(node.id) ?? { width: CARD_WIDTH, height: estimateCardHeight(getNodeDef(node.type)) };
