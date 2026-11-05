import type { WorkflowNode } from "@/generated/bindings";
import { getNodeDef } from "@/store/catalog";
import { CARD_WIDTH } from "@/workflow/nodes";
import { cardShape } from "./card-shapes";

const UNKNOWN_HEIGHT = 120;

export type MeasuredPort = { handle: string; side: "in" | "out"; y: number };

export function cardSize(node: Pick<WorkflowNode, "id" | "type">) {
  const shape = cardShape(node.id);
  return shape
    ? { width: shape.width, height: shape.height }
    : { width: CARD_WIDTH, height: getNodeDef(node.type)?.cardHeight ?? UNKNOWN_HEIGHT };
}

export function measuredPorts(id: string): MeasuredPort[] | null {
  const shape = cardShape(id);
  if (!shape) return null;
  return shape.handles.flatMap((h) =>
    h.id
      ? [
          {
            handle: h.id,
            side: h.type === "target" ? ("in" as const) : ("out" as const),
            y: h.y + (h.height ?? 0) / 2,
          },
        ]
      : [],
  );
}
