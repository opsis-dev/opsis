import { audioNodes } from "./audio";
import { browserNodes } from "./browser";
import { cameraNodes } from "./camera";
import { captureNodes } from "./capture";
import { cursorNodes } from "./cursor";
import type { NodeDef } from "./define";
import { elementNodes } from "./elements";
import { mediaNodes } from "./media";
import { outputNodes } from "./output";
import { textNodes } from "./text";
import { timelineNodes } from "./timeline";
import { transitionNodes } from "./transitions";
import { utilityNodes } from "./utility";

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

const byType = new Map(NODE_DEFS.map((d) => [d.type, d]));

if (byType.size !== NODE_DEFS.length) {
  const seen = new Set<string>();
  const dupes: string[] = [];
  for (const { type } of NODE_DEFS) {
    if (seen.has(type)) dupes.push(type);
    seen.add(type);
  }
  throw new Error(`Duplicate node types: ${dupes.join(", ")}`);
}

export const getNodeDef = (type: string): NodeDef | undefined => byType.get(type);

export const nodeName = (node: { label?: string; type: string }): string =>
  node.label?.trim() || getNodeDef(node.type)?.title || node.type;

export function defaultParams(type: string): Record<string, unknown> {
  const def = getNodeDef(type);
  if (!def) return {};
  const out: Record<string, unknown> = {};
  for (const [key, field] of Object.entries(def.params.shape)) {
    const parsed = field.safeParse(undefined);
    if (parsed.success && parsed.data !== undefined) out[key] = parsed.data;
  }
  return out;
}
