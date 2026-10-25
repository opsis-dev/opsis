import { schemaDefaults } from "#core/elements/introspect";
import { nonBlankString } from "#core/guards";
import { findById } from "#core/lists";
import { audioNodes } from "#core/nodes/audio";
import { browserNodes } from "#core/nodes/browser";
import { cameraNodes } from "#core/nodes/camera";
import { captureNodes } from "#core/nodes/capture";
import { cursorNodes } from "#core/nodes/cursor";
import type { NodeDef } from "#core/nodes/define";
import { effectNodes } from "#core/nodes/effects";
import { elementNodes } from "#core/nodes/elements";
import { gradeNodes } from "#core/nodes/grade";
import { mediaNodes } from "#core/nodes/media";
import { outputNodes } from "#core/nodes/output";
import { textNodes } from "#core/nodes/text";
import { timelineNodes } from "#core/nodes/timeline";
import { transitionNodes } from "#core/nodes/transitions";
import { utilityNodes } from "#core/nodes/utility";

export const NODE_DEFS: NodeDef[] = [
  ...browserNodes,
  ...captureNodes,
  ...cameraNodes,
  ...cursorNodes,
  ...textNodes,
  ...effectNodes,
  ...gradeNodes,
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
  nonBlankString(node.label);

export const nodeName = (node: { label?: string; type: string }, fallback?: string): string =>
  customLabel(node) ?? fallback ?? (getNodeDef(node.type)?.title || node.type);

export function nodeNameById(
  nodes: ReadonlyArray<{ id: string; label?: string; type: string }>,
  id: string,
  whenMissing = id,
): string {
  const node = findById(nodes, id);
  return node ? nodeName(node) : whenMissing;
}

export const isOutputType = (type: string): boolean => getNodeDef(type)?.category === "Output";

export function defaultParams(type: string): Record<string, unknown> {
  const def = getNodeDef(type);
  return def ? schemaDefaults(def.params) : {};
}
