import { z } from "zod";
import { isClip, itemClip, wrapClip } from "../compile/clip";
import { ELEMENTS, elementNodeType, type ElementCategory, type ElementMeta } from "../elements";
import { assetKeys, projectRef, widgetKeys } from "../elements/introspect";
import { fontStack } from "../fonts";
import { frameSecs, quote, row, sentence, type CardRow } from "./card";
import { defineNode, port, type NodeCategory, type NodeDef } from "./define";

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
    if (typeof v === "string" && v) rows.push(row("file", v.split("/").pop() ?? v, { title: v }));
  }
  const duration = Number(params.clipDuration ?? 0);
  rows.push(
    meta.kind === "wrapper" && !duration
      ? row("clock", "Matches the clip", { tone: "muted" })
      : row("clock", frameSecs(duration || meta.defaultDurationInFrames, fps)),
  );
  return rows;
}

export const NODE_CATEGORY_FOR: Record<ElementCategory, NodeCategory> = {
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

const CUSTOM_NODE_ELEMENTS = new Set(["core/captions"]);

export function elementProps(
  meta: ElementMeta,
  params: Record<string, unknown>,
): Record<string, unknown> {
  const { clipDuration: _drop, ...rest } = params;
  const props: Record<string, unknown> = { ...meta.fixedProps, ...rest };
  for (const key of widgetKeys(meta.schema, "font")) {
    const name = typeof props[key] === "string" ? (props[key] as string).trim() : "";
    if (name) props[key] = fontStack(name, { families: [] });
    else delete props[key];
  }
  for (const key of assetKeys(meta.schema)) {
    if (typeof props[key] === "string") props[key] = projectRef(props[key] as string);
  }
  return props;
}

function elementNode(meta: ElementMeta): NodeDef {
  const wrapper = meta.kind === "wrapper";
  return defineNode({
    type: elementNodeType(meta.key),
    title: sentence(meta.title),
    category: NODE_CATEGORY_FOR[meta.category],
    description: meta.description,
    card: ({ params, settings }) =>
      elementCard(meta, params as Record<string, unknown>, settings.fps),
    icon: ICON_FOR[meta.category],
    phase: "compile",
    element: meta.key,
    inputs: wrapper ? { clip: port("clip", "Clip") } : {},
    outputs: { clip: port("clip", "Clip") },
    params: meta.schema.extend({
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
    }),
    compile: ({ nodeId, params, inputs }) => {
      const props = elementProps(meta, params as Record<string, unknown>);
      const duration = (params as { clipDuration: number }).clipDuration;
      if (!wrapper) return { clip: itemClip(nodeId, meta.key, props, duration) };
      const clip = isClip(inputs.clip) ? inputs.clip : undefined;
      if (!clip)
        return {
          clip: itemClip(nodeId, meta.key, props, duration || meta.defaultDurationInFrames),
        };
      return {
        clip: wrapClip(
          clip,
          nodeId,
          { wrapper: { component: meta.key, props } },
          duration || clip.durationInFrames,
        ),
      };
    },
  });
}

export const elementNodes: NodeDef[] = ELEMENTS.filter(
  (e) => e.kind !== "presentation" && !CUSTOM_NODE_ELEMENTS.has(e.key),
).map(elementNode);
