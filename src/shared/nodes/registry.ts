import { audioNodes } from "./audio";
import { browserNodes } from "./browser";
import { cameraNodes } from "./camera";
import { captureNodes } from "./capture";
import { cursorNodes } from "./cursor";
import type { Workflow, WorkflowNode } from "../schema/workflow";
import { defaultMix, mixSchema, type Mix } from "../mix/types";
import { renameVariable } from "../vars";
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

export function normalizeNode(node: WorkflowNode): WorkflowNode {
  if (node.type === "camera.autoZoom" && node.params && "chrome" in node.params) {
    const { chrome, ...rest } = node.params;
    node = {
      ...node,
      params: { ...rest, ...("frame" in rest ? {} : { frame: chrome ? "macos" : "none" }) },
    };
  }
  const def = getNodeDef(node.type);
  const label = node.params?.label;
  if (!def || typeof label !== "string" || "label" in def.params.shape) return node;
  const { label: _moved, ...params } = node.params;
  const name = node.label?.trim() || label.trim();
  return { ...node, params, ...(name ? { label: name } : {}) };
}

const LEGACY_VARIABLES: Record<string, string> = {
  BASE_URL: "Site address",
  PRODUCT_NAME: "Product name",
  VERSION: "Version",
  BEFORE_URL: "Before address",
  AFTER_URL: "After address",
};

export function normalizeWorkflow(wf: Workflow): Workflow {
  let nodes = wf.nodes.map(normalizeNode);
  const variables: Record<string, string> = {};
  for (const [name, value] of Object.entries(wf.variables)) {
    const next = LEGACY_VARIABLES[name];
    if (next && !(next in wf.variables)) {
      variables[next] = value;
      nodes = nodes.map((n) => ({ ...n, params: renameVariable(n.params, name, next) }));
    } else variables[name] = value;
  }
  nodes = nodes.map((n) => ({
    ...n,
    params: renameVariable(n.params, "PROJECT_NAME", "Project name"),
  }));
  const migrated = migrateMusicDucking(nodes, wf.mix);
  return {
    ...wf,
    variables,
    edges: migrateSoundPorts(wf.edges, nodes),
    nodes: migrated.nodes,
    ...(migrated.mix ? { mix: migrated.mix } : {}),
  };
}

const LEGACY_SOUND_PORTS = new Set(["a1", "a2", "a3"]);

export function migrateSoundPorts(
  edges: Workflow["edges"],
  nodes: WorkflowNode[],
): Workflow["edges"] {
  const mixers = new Set(nodes.filter((n) => n.type === "audio.mix").map((n) => n.id));
  return edges.map((e) =>
    mixers.has(e.target) && LEGACY_SOUND_PORTS.has(e.targetHandle)
      ? {
          ...e,
          targetHandle: "sounds",
          id: `${e.source}.${e.sourceHandle}->${e.target}.sounds`,
        }
      : e,
  );
}

const LEGACY_DUCK_PARAMS = ["duck", "duckLevel", "duckUnder", "duckFadeMs"];
const LEGACY_DUCK_KEYS: Record<string, string[]> = {
  all: ["effects", "clips"],
  effects: ["effects"],
  clips: ["clips"],
};

export function migrateMusicDucking(
  nodes: WorkflowNode[],
  current: Mix | undefined,
): { nodes: WorkflowNode[]; mix: Mix | undefined } {
  let mix = current ? mixSchema.parse(current) : undefined;
  if (mix) {
    const ids = new Set(nodes.map((n) => n.id));
    const channels = Object.fromEntries(Object.entries(mix.channels).filter(([id]) => ids.has(id)));
    mix = { ...mix, channels };
  }
  const legacy = nodes.filter(
    (n) => n.type === "audio.music" && LEGACY_DUCK_PARAMS.some((k) => k in n.params),
  );
  if (legacy.length === 0) return { nodes, mix };
  const source = legacy.find((n) => n.params.duck === true);
  if (source) {
    mix ??= defaultMix();
    const level = Number(source.params.duckLevel ?? 30);
    const amount = Math.min(
      24,
      Math.max(1, Math.round(-20 * Math.log10(Math.max(0.01, level / 100)))),
    );
    const fade = Number(source.params.duckFadeMs ?? 250);
    const keys = LEGACY_DUCK_KEYS[String(source.params.duckUnder ?? "all")] ?? LEGACY_DUCK_KEYS.all;
    mix = {
      ...mix,
      buses: mix.buses.map((b) =>
        b.id === "music" && !b.duck.on
          ? {
              ...b,
              duck: {
                on: true,
                keys,
                amount,
                attack: Math.max(5, fade),
                release: Math.max(20, fade),
                ignoreShorterMs: 0,
              },
            }
          : b,
      ),
    };
  }
  const strip = new Set(legacy.map((n) => n.id));
  return {
    nodes: nodes.map((n) => {
      if (!strip.has(n.id)) return n;
      const params = Object.fromEntries(
        Object.entries(n.params).filter(([k]) => !LEGACY_DUCK_PARAMS.includes(k)),
      );
      return { ...n, params };
    }),
    mix,
  };
}
