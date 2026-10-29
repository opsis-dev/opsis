import type { WorkflowNode } from "@/generated/bindings";
import { getNodeDef } from "@/store/catalog";
import { CARD_WIDTH } from "@/workflow/nodes";

const UNKNOWN_HEIGHT = 120;

function measured(id: string): { width: number; height: number } | null {
  const el = document.querySelector<HTMLElement>(`.react-flow__node[data-id="${CSS.escape(id)}"]`);
  return el && el.offsetWidth && el.offsetHeight
    ? { width: el.offsetWidth, height: el.offsetHeight }
    : null;
}

export const cardSize = (node: Pick<WorkflowNode, "id" | "type">) =>
  measured(node.id) ?? {
    width: CARD_WIDTH,
    height: getNodeDef(node.type)?.cardHeight ?? UNKNOWN_HEIGHT,
  };
