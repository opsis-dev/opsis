import { audioNodes } from "./audio";
import { browserNodes } from "./browser";
import { cameraNodes } from "./camera";
import { captureNodes } from "./capture";
import { cursorNodes } from "./cursor";
import type { Workflow } from "../schema/workflow";
import { mixSchema } from "../mix/types";
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

const byType = new Map(NODE_DEFS.map((d) => [d.type, d]));

if (byType.size !== NODE_DEFS.length) {
  const seen = new Set<string>();
  const dupes = NODE_DEFS.filter((d) => (seen.has(d.type) ? true : (seen.add(d.type), false))).map(
    (d) => d.type,
  );
  throw new Error(`Duplicate node types: ${dupes.join(", ")}`);
}

export const getNodeDef = (type: string): NodeDef | undefined => byType.get(type);

export function defaultParams(type: string): Record<string, unknown> {
  const def = getNodeDef(type);
  if (!def) return {};
  const out: Record<string, unknown> = {};
  for (const [key, field] of Object.entries(def.params.shape)) {
    const parsed = (
      field as { safeParse: (v: unknown) => { success: boolean; data?: unknown } }
    ).safeParse(undefined);
    if (parsed.success && parsed.data !== undefined) out[key] = parsed.data;
  }
  return out;
}

export function normalizeWorkflow(wf: Workflow): Workflow {
  if (!wf.mix) return wf;
  const ids = new Set(wf.nodes.map((n) => n.id));
  const mix = mixSchema.parse(wf.mix);
  return {
    ...wf,
    mix: {
      ...mix,
      channels: Object.fromEntries(Object.entries(mix.channels).filter(([id]) => ids.has(id))),
    },
  };
}
