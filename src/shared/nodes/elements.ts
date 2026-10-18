import { z } from "zod";
import { projectRef } from "../asset-ref";
import { itemClip, withHandles, wrapClip } from "../compile/clip";
import { ELEMENTS, elementNodeType } from "../elements";
import { CAPTIONS_ELEMENT } from "../elements/core";
import { assetKeys, widgetKeys } from "../elements/introspect";
import type { ElementCategory, ElementMeta } from "../elements/types";
import { fontStack } from "../fonts";
import type { EditHandles, VisualEnvelope } from "../schema/timeline";
import type { ClipValue } from "../schema/values";
import { fileRow, frameSecs, quote, row, sentence, type CardRow } from "./card";
import { clipInput, defineNode, port, type NodeCategory, type NodeDef } from "./define";
import type { NodeIconName } from "./icon-name";
import { int, MAX_FRAMES, visualEnvelope } from "./params";

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

type ElementTiming = { clipDuration: number; envelope: VisualEnvelope };
type ElementParams = Record<string, unknown> & ElementTiming;

const SHORT_ELEMENT_RATIO = 0.6;

const textParam = (params: Record<string, unknown>, key: string): string | undefined => {
  const value = params[key];
  return typeof value === "string" && value.trim() ? value : undefined;
};

function elementCard(meta: ElementMeta, params: ElementParams, fps: number): CardRow[] {
  const rows: CardRow[] = [];
  const title = TEXT_KEYS.filter((k) => k in meta.schema.shape)
    .map((k) => textParam(params, k))
    .find((v) => v !== undefined);
  if (title) rows.push(row("type", quote(title)));
  for (const asset of assetKeys(meta.schema)) {
    const v = params[asset];
    if (typeof v === "string" && v) rows.push(fileRow("file", v));
  }
  rows.push(durationRow(meta, params.clipDuration, fps));
  return rows;
}

function durationRow(meta: ElementMeta, duration: number, fps: number): CardRow {
  const usual = meta.defaultDurationInFrames;
  if (meta.kind === "wrapper")
    return duration
      ? row("clock", frameSecs(duration, fps))
      : row("clock", "Matches the clip", { tone: "muted" });
  if (duration > 0 && duration < SHORT_ELEMENT_RATIO * usual)
    return row("clock", `${frameSecs(duration, fps)} · usually ${frameSecs(usual, fps)}`, {
      tone: "warn",
      title: "Shorter than this element usually runs, so its animation may be cut off.",
    });
  return row("clock", frameSecs(duration || usual, fps));
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

const ICON_FOR: Record<ElementCategory, NodeIconName> = {
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

function compileElement(
  meta: ElementMeta,
  nodeId: string,
  { clipDuration, envelope, ...params }: ElementParams,
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
      clipDuration: int(wrapper ? 0 : meta.defaultDurationInFrames, "Length", {
        min: wrapper ? 0 : 1,
        max: MAX_FRAMES,
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
