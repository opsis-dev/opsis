import { audioNodes } from "./audio";
import { browserNodes } from "./browser";
import { cameraNodes } from "./camera";
import { captureNodes } from "./capture";
import { cursorNodes } from "./cursor";
import type { NodeDef } from "./define";
import { schemaDefaults } from "../elements/introspect";
import { elementNodes } from "./elements";
import { mediaNodes } from "./media";
import { outputNodes } from "./output";
import { textNodes } from "./text";
import { timelineNodes } from "./timeline";
import { transitionNodes } from "./transitions";
import { utilityNodes } from "./utility";
import { findById } from "../lists";

export const NODE_DEFS: NodeDef[] = [
  ...browserNodes,
  ...captureNodes,
  ...cameraNodes,
  ...cursorNodes,
  ...textNodes,
  ...elementNodes,
  ...transitionNodes,
  ...audioNodes,
  ...timelineNodes,
  ...mediaNodes,
  ...outputNodes,
  ...utilityNodes,
];

export const VISIBLE_NODE_DEFS = NODE_DEFS.filter((d) => !d.hidden);

const byType = new Map<string, NodeDef>();
for (const def of NODE_DEFS) {
  if (byType.has(def.type)) throw new Error(`Duplicate node type: ${def.type}`);
  byType.set(def.type, def);
}

export const getNodeDef = (type: string): NodeDef | undefined => byType.get(type);

export const customLabel = (node: { label?: string }): string | undefined =>
  node.label?.trim() || undefined;

export const nodeName = (
  node: { label?: string; type: string },
  fallback = getNodeDef(node.type)?.title || node.type,
): string => customLabel(node) ?? fallback;

export function nodeNameById(
  wf: { nodes: ReadonlyArray<{ id: string; label?: string; type: string }> },
  id: string,
  fallback: string,
): string {
  const node = findById(wf.nodes, id);
  return node ? nodeName(node) : fallback;
}

export function defaultParams(type: string): Record<string, unknown> {
  const def = getNodeDef(type);
  return def ? schemaDefaults(def.params) : {};
}
