import { isString } from "@opsis/video/common/guards";
import type { WorkflowNode } from "@/generated/bindings";
import { getNodeDef } from "@/store/catalog";

export const CARD_WIDTH = 256;
export const COLUMN_WIDTH = 300;
export const ROW_HEIGHT = 170;

export const GROUP_TYPE = "core.group";
export const NOTE_TYPE = "core.note";

export const CLIP_PORT = "clip";

export const isGroupNode = (node: Pick<WorkflowNode, "type">): boolean => node.type === GROUP_TYPE;

export const isNoteNode = (node: Pick<WorkflowNode, "type">): boolean => node.type === NOTE_TYPE;

const textParam = (node: WorkflowNode, key: string): string => {
  const own = node.params[key];
  const fallback = getNodeDef(node.type)?.defaults[key];
  return isString(own) ? own : isString(fallback) ? fallback : "";
};

export const groupLook = (node: WorkflowNode) => ({
  title: textParam(node, "title"),
  color: textParam(node, "color"),
});

export const noteText = (node: WorkflowNode): string => textParam(node, "text");

const TRANSITION_WARN_RATIO = 0.4;

export const coversTooMuch = (ratio: number): boolean => ratio > TRANSITION_WARN_RATIO;
