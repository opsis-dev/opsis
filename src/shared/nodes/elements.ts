import { z } from "zod";
import { itemClip, withHandles, wrapClip } from "../compile/clip";
import type { EditHandles, VisualEnvelope } from "../schema/timeline";
import type { ClipValue } from "../schema/values";
import {
  CAPTIONS_ELEMENT,
  ELEMENTS,
  elementNodeType,
  type ElementCategory,
  type ElementMeta,
} from "../elements";
import { assetKeys, widgetKeys } from "../elements/introspect";
import { fontStack } from "../fonts";
import { fileRow, frameSecs, quote, row, sentence, type CardRow } from "./card";
import { clipInput, defineNode, port, type NodeCategory, type NodeDef } from "./define";
import { visualEnvelope } from "./params";
import { projectRef } from "../asset-ref";

const TEXT_KEYS = [
  "title",
  "text",
  "headline",
  "heading",
  "label",
  "name",
  "words",
  "quote",
  "message",
];

function elementCard(meta: ElementMeta, params: Record<string, unknown>, fps: number): CardRow[] {
  const rows: CardRow[] = [];
  const shape = meta.schema.shape as Record<string, unknown>;
  const key = TEXT_KEYS.find(
    (k) => k in shape && typeof params[k] === "string" && String(params[k]).trim(),
  );
  if (key) rows.push(row("type", quote(String(params[key]))));
  for (const asset of assetKeys(meta.schema)) {
    const v = params[asset];
    if (typeof v === "string" && v) rows.push(fileRow("file", v));
  }
  const duration = Number(params.clipDuration ?? 0);
  const short =
    meta.kind !== "wrapper" && duration > 0 && duration < 0.6 * meta.defaultDurationInFrames;
  rows.push(
    meta.kind === "wrapper" && !duration
      ? row("clock", "Matches the clip", { tone: "muted" })
      : short
        ? row(
            "clock",
            `${frameSecs(duration, fps)} · usually ${frameSecs(meta.defaultDurationInFrames, fps)}`,
            {
              tone: "warn",
              title: "Shorter than this element usually runs, so its animation may be cut off.",
            },
          )
        : row("clock", frameSecs(duration || meta.defaultDurationInFrames, fps)),
  );
  return rows;
}

const NODE_CATEGORY_FOR: Record<ElementCategory, NodeCategory> = {
  core: "Elements",
  text: "Text & overlays",
  overlay: "Text & overlays",
  background: "Backgrounds",
  effect: "Effects",
  ui: "Elements",
  data: "Elements",
  scene: "Elements",
  device: "Elements",
  transition: "Transitions",
};

const ICON_FOR: Record<ElementCategory, string> = {
  core: "box",
  text: "type",
  overlay: "sticker",
  background: "wallpaper",
  effect: "sparkles",
  ui: "app-window",
  data: "chart-column",
  scene: "clapperboard",
  device: "smartphone",
  transition: "blend",
};

function elementProps(meta: ElementMeta, params: Record<string, unknown>): Record<string, unknown> {
  const props: Record<string, unknown> = { ...meta.fixedProps, ...params };
  for (const key of widgetKeys(meta.schema, "font")) {
    const value = props[key];
    const name = typeof value === "string" ? value.trim() : "";
    if (name) props[key] = fontStack(name);
    else delete props[key];
  }
  for (const key of assetKeys(meta.schema)) {
    const value = props[key];
    if (typeof value === "string") props[key] = projectRef(value);
  }
  return props;
}

type ElementTiming = { clipDuration: number; envelope: VisualEnvelope };

function compileElement(
  meta: ElementMeta,
  nodeId: string,
  { clipDuration, envelope, ...params }: Record<string, unknown> & ElementTiming,
  input: ClipValue | undefined,
): ClipValue {
  const props = elementProps(meta, params);
  const handles: EditHandles = { trimEnd: { nodeId, param: "clipDuration", unit: "frames" } };
  if (meta.kind !== "wrapper" || !input)
    return itemClip(nodeId, meta.key, props, clipDuration || meta.defaultDurationInFrames, {
      handles,
      envelope,
    });
  const wrapped = wrapClip(
    input,
    nodeId,
    { wrapper: { component: meta.key, props } },
    clipDuration || input.durationInFrames,
  );
  return withHandles(wrapped, handles, envelope);
}

function elementNode(meta: ElementMeta): NodeDef {
  const wrapper = meta.kind === "wrapper";
  return defineNode({
    type: elementNodeType(meta.key),
    title: sentence(meta.title),
    category: NODE_CATEGORY_FOR[meta.category],
    description: meta.description,
    card: ({ params, settings }) => elementCard(meta, params, settings.fps),
    icon: ICON_FOR[meta.category],
    phase: "compile",
    element: meta.key,
    inputs: wrapper ? { clip: port("clip", "Clip") } : {},
    outputs: { clip: port("clip", "Clip") },
    params: z.object({
      ...meta.schema.shape,
      clipDuration: z
        .number()
        .int()
        .min(wrapper ? 0 : 1)
        .max(108_000)
        .default(wrapper ? 0 : meta.defaultDurationInFrames)
        .meta({
          description: "Length",
          unit: "frames",
          ...(wrapper ? { zeroLabel: "Match the clip" } : {}),
        }),
      envelope: visualEnvelope(),
    }),
    compile: ({ nodeId, params, inputs }) => ({
      clip: compileElement(meta, nodeId, params, wrapper ? clipInput(inputs) : undefined),
    }),
  });
}

export const elementNodes: NodeDef[] = ELEMENTS.filter(
  (e) => e.kind !== "presentation" && e.key !== CAPTIONS_ELEMENT,
).map(elementNode);
