import { isString } from "@opsis/video/common/guards";
import type { WorkflowNode } from "@/generated/bindings";
import { getNodeDef } from "@/store/catalog";

export const CARD_WIDTH = 256;
export const COLUMN_WIDTH = 300;
export const ROW_HEIGHT = 170;

export const GROUP_TYPE = "core.group";
export const NOTE_TYPE = "core.note";

export const CLIP_PORT = "clip";

export function isGroupNode(node: Pick<WorkflowNode, "type">): boolean {
  return node.type === GROUP_TYPE;
}

export function isNoteNode(node: Pick<WorkflowNode, "type">): boolean {
  return node.type === NOTE_TYPE;
}

function textParam(node: WorkflowNode, key: string): string {
  const own = node.params[key];
  const fallback = getNodeDef(node.type)?.defaults[key];
  return isString(own) ? own : isString(fallback) ? fallback : "";
}

export function groupLook(node: WorkflowNode) {
  return {
    title: textParam(node, "title"),
    color: textParam(node, "color"),
  };
}

export function noteText(node: WorkflowNode): string {
  return textParam(node, "text");
}

export function noteColor(node: WorkflowNode): string {
  return textParam(node, "color");
}

const TRANSITION_WARN_RATIO = 0.4;

export function coversTooMuch(ratio: number): boolean {
  return ratio > TRANSITION_WARN_RATIO;
}
