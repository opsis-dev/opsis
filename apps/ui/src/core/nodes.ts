import { signedText } from "@opsis/video/common/format";
import { isString } from "@opsis/video/common/guards";
import type { ReframeMode, WorkflowNode, WorkflowSettings } from "@/generated/bindings";
import { getNodeDef } from "@/store/catalog";

export const CARD_WIDTH = 256;
export const CARD_GAP = 32;
export const COLUMN_WIDTH = 300;
export const ROW_HEIGHT = 170;

export const GROUP_TYPE = "core.group";
export const NOTE_TYPE = "core.note";

export const CLIP_PORT = "clip";
export const SESSION_PORT = "session";

export const IN_PORTS = ["in1", "in2", "in3", "in4", "in5", "in6"] as const;

export type InPort = (typeof IN_PORTS)[number];

export const inPortIndex = (handle: string): number => IN_PORTS.findIndex((p) => p === handle);

export function nextInPort(handle: string): InPort | undefined {
  const index = inPortIndex(handle);
  return index === -1 ? undefined : IN_PORTS[index + 1];
}

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

export const VARIANTS = ["16:9", "1:1", "4:5", "9:16"] as const;
export type Variant = (typeof VARIANTS)[number];

const VARIANT_RATIOS: Record<Variant, [width: number, height: number]> = {
  "16:9": [16, 9],
  "1:1": [1, 1],
  "4:5": [4, 5],
  "9:16": [9, 16],
};

const evenPx = (value: number): number => Math.round(value / 2) * 2;

function sizeFor(variant: Variant, edge: number, anchor: "long" | "short") {
  const [w, h] = VARIANT_RATIOS[variant];
  const landscape = w >= h;
  const edgeIsWidth = anchor === "long" ? landscape : !landscape;
  return edgeIsWidth
    ? { width: evenPx(edge), height: evenPx((edge * h) / w) }
    : { width: evenPx((edge * w) / h), height: evenPx(edge) };
}

export const variantSize = (
  variant: Variant,
  settings: Pick<WorkflowSettings, "width" | "height">,
) => sizeFor(variant, Math.max(settings.width, settings.height), "long");

export const variantCanvas = (variant: Variant, short: number) => sizeFor(variant, short, "short");

const REFRAMES: ReframeMode[] = ["follow", "center", "fit"];

export function exportReframe(nodes: WorkflowNode[]): ReframeMode {
  const render = nodes.find((n) => n.type === "output.render" && !n.disabled);
  const chosen = render?.params.reframe ?? getNodeDef("output.render")?.defaults.reframe;
  return REFRAMES.find((r) => r === chosen) ?? "follow";
}

export const QUALITY_PRESETS = { Low: 28, Medium: 23, High: 18, Max: 12 } as const;

export const LOUDNESS_PRESETS = { "Online video": -14, Podcast: -16, Broadcast: -23 } as const;

export const loudnessName = (lufs: number): string =>
  Object.entries(LOUDNESS_PRESETS).find(([, v]) => v === lufs)?.[0] ?? `${signedText(lufs)} LUFS`;
