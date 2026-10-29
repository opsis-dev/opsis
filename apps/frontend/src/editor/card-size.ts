import type { WorkflowNode } from "@/generated/bindings";
import { getNodeDef } from "@/store/catalog";
import { CARD_WIDTH } from "@/workflow/nodes";

const UNKNOWN_HEIGHT = 120;

export type MeasuredPort = { handle: string; side: "in" | "out"; y: number };

const cardElement = (id: string) =>
  document.querySelector<HTMLElement>(`.react-flow__node[data-id="${CSS.escape(id)}"]`);

function measured(id: string): { width: number; height: number } | null {
  const el = cardElement(id);
  return el && el.offsetWidth && el.offsetHeight
    ? { width: el.offsetWidth, height: el.offsetHeight }
    : null;
}

export const cardSize = (node: Pick<WorkflowNode, "id" | "type">) =>
  measured(node.id) ?? {
    width: CARD_WIDTH,
    height: getNodeDef(node.type)?.cardHeight ?? UNKNOWN_HEIGHT,
  };

export function measuredPorts(id: string): MeasuredPort[] | null {
  const el = cardElement(id);
  if (!el?.offsetWidth) return null;
  const card = el.getBoundingClientRect();
  const zoom = card.width / el.offsetWidth;
  if (!zoom) return null;
  return Array.from(el.querySelectorAll<HTMLElement>(".react-flow__handle[data-handleid]"), (h) => {
    const box = h.getBoundingClientRect();
    return {
      handle: h.dataset.handleid ?? "",
      side: h.classList.contains("target") ? "in" : "out",
      y: (box.top + box.height / 2 - card.top) / zoom,
    };
  });
}
