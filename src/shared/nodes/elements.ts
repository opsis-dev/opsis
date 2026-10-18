import { z } from "zod";
import { projectRef } from "../asset-ref";
import {
  EMPTY_CLIP_FRAMES,
  emptyClip,
  itemClip,
  sceneSequence,
  withHandles,
  wrapClip,
} from "../compile/clip";
import { ELEMENTS, elementNodeType } from "../elements";
import { CAPTIONS_ELEMENT } from "../elements/core";
import { assetKeys, widgetKeys } from "../elements/introspect";
import type { ElementCategory, ElementMeta } from "../elements/types";
import { fontStack } from "../fonts";
import { plural } from "../format";
import { MAX_FRAMES } from "../limits";
import type { EditHandles, VisualEnvelope } from "../schema/timeline";
import type { ClipValue } from "../schema/values";
import {
  connectedPorts,
  fileRow,
  frameSecs,
  quote,
  row,
  warnIf,
  type CardContext,
  type CardRow,
} from "./card";
import {
  clipInput,
  defineNode,
  fromElement,
  port,
  type CompileInputs,
  type NodeCategory,
  type NodeDef,
} from "./define";
import type { NodeIconName } from "./icon-name";
import { choice, int, more, text, visualEnvelope } from "./params";
import { clipPorts, connectedClips, IN_PORTS, sceneLabel } from "./ports";

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

const stringParam = (params: Record<string, unknown>, key: string): string | undefined => {
  const value = params[key];
  return typeof value === "string" ? value : undefined;
};

const titleKeys = (schema: ElementMeta["schema"]): string[] =>
  TEXT_KEYS.filter((k) => k in schema.shape);

function elementCard(
  meta: ElementMeta,
  titles: string[],
  params: ElementParams,
  fps: number,
): CardRow[] {
  const rows: CardRow[] = [];
  const title = titles.map((k) => stringParam(params, k)?.trim()).find(Boolean);
  if (title) rows.push(row("type", quote(title)));
  for (const asset of assetKeys(meta.schema)) {
    const file = stringParam(params, asset);
    if (file) rows.push(fileRow("file", file));
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

const NODE_KIND_FOR: Record<ElementCategory, { category: NodeCategory; icon: NodeIconName }> = {
  core: { category: "Elements", icon: "box" },
  text: { category: "Text & overlays", icon: "type" },
  overlay: { category: "Text & overlays", icon: "sticker" },
  background: { category: "Backgrounds", icon: "wallpaper" },
  effect: { category: "Effects", icon: "sparkles" },
  ui: { category: "Elements", icon: "app-window" },
  data: { category: "Elements", icon: "chart-column" },
  scene: { category: "Elements", icon: "clapperboard" },
  device: { category: "Elements", icon: "smartphone" },
  transition: { category: "Transitions", icon: "blend" },
};

function elementProps(meta: ElementMeta, params: Record<string, unknown>): Record<string, unknown> {
  const props: Record<string, unknown> = { ...meta.fixedProps, ...params };
  for (const key of widgetKeys(meta.schema, "font")) {
    const name = stringParam(props, key)?.trim();
    if (name) props[key] = fontStack(name);
    else delete props[key];
  }
  for (const key of assetKeys(meta.schema)) {
    const path = stringParam(props, key);
    if (path !== undefined) props[key] = projectRef(path);
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
  const titles = titleKeys(meta.schema);
  return defineNode({
    ...fromElement(meta),
    ...NODE_KIND_FOR[meta.category],
    type: elementNodeType(meta.key),
    description: meta.description,
    card: ({ params, settings }) => elementCard(meta, titles, params, settings.fps),
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

const SEQUENCER_KIND = { category: "Timeline", icon: "clapperboard" } satisfies {
  category: NodeCategory;
  icon: NodeIconName;
};

const MAX_SCENE_GAP_FRAMES = 120;
const DEFAULT_SCENE_BACKGROUND = "#0b0b10";

const sceneBackgroundSchema = z.object({
  scene: choice(
    IN_PORTS,
    "in1",
    "Scene",
    Object.fromEntries(IN_PORTS.map((handle) => [handle, sceneLabel(handle)])),
  ),
  color: more(text(DEFAULT_SCENE_BACKGROUND, "Colour"), { widget: "color" }),
});

type SceneBackground = z.infer<typeof sceneBackgroundSchema>;

const sequencerFields = {
  gapFrames: int(1, "Pause between scenes", {
    min: 0,
    max: MAX_SCENE_GAP_FRAMES,
    step: 1,
    unit: "frames",
    help: "How long the stage stays empty between one scene leaving and the next landing.",
  }),
  backgrounds: z.array(sceneBackgroundSchema).default([]).meta({
    description: "Scene backgrounds",
    help: "A colour behind a scene. It fades into the next scene's colour.",
    advanced: true,
  }),
  envelope: visualEnvelope(),
};

type SequencerParams = Record<string, unknown> & {
  gapFrames: number;
  backgrounds: SceneBackground[];
  envelope: VisualEnvelope;
};

function sequencerCard({
  params,
  connected,
  inputItems,
  clipFrames,
  settings,
}: CardContext<SequencerParams>): Array<CardRow | false | 0 | undefined> {
  const ports = connectedPorts(IN_PORTS, connected);
  const items = ports.map((handle) => inputItems?.[handle] ?? 1);
  return [
    row("clapperboard", plural(ports.length, "scene"), warnIf(!ports.length)),
    items.some((n) => n > 1) && row("layers", `Items per scene: ${items.join(" · ")}`),
    params.gapFrames > 0 &&
      row("timer", `${frameSecs(params.gapFrames, settings.fps)} between scenes`),
    clipFrames && row("clock", frameSecs(clipFrames, settings.fps)),
  ];
}

function compileSequencer(
  meta: ElementMeta,
  nodeId: string,
  { gapFrames, backgrounds, envelope, ...params }: SequencerParams,
  inputs: CompileInputs,
): ClipValue {
  const scenes = connectedClips(inputs);
  if (!scenes.length) return emptyClip(EMPTY_CLIP_FRAMES);
  const colorOf = (handle: string) => backgrounds.findLast((b) => b.scene === handle)?.color;
  const clip = sceneSequence(nodeId, scenes, {
    gap: gapFrames,
    sequencer: {
      component: meta.key,
      props: { ...elementProps(meta, params), gapFrames },
      backgrounds: scenes.map(({ port: handle }) => colorOf(handle) ?? null),
    },
  });
  return withHandles(clip, {}, envelope);
}

function sequencerNode(meta: ElementMeta): NodeDef {
  return defineNode({
    ...fromElement(meta),
    ...SEQUENCER_KIND,
    type: elementNodeType(meta.key),
    description: meta.description,
    card: sequencerCard,
    inputs: clipPorts(sceneLabel),
    outputs: { clip: port("clip", "Clip") },
    params: z.object({ ...meta.schema.shape, ...sequencerFields }),
    compile: ({ nodeId, params, inputs }) => ({
      clip: compileSequencer(meta, nodeId, params, inputs),
    }),
  });
}

export const elementNodes: NodeDef[] = ELEMENTS.filter(
  (e) => e.kind !== "presentation" && e.key !== CAPTIONS_ELEMENT,
).map((meta) => (meta.kind === "sequencer" ? sequencerNode(meta) : elementNode(meta)));
