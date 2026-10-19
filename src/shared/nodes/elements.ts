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
import {
  assetKeys,
  fieldMeta,
  isColorSchema,
  mapLeaves,
  shapeEntries,
  unwrap,
  widgetKeys,
  type FieldMeta,
} from "../elements/introspect";
import { isThemeValue, themeColors } from "../elements/theme";
import type { ElementCategory, ElementMeta } from "../elements/types";
import { fontProp } from "../fonts";
import { plural } from "../format";
import { parseGradient } from "../gradient";
import { isNumber, isRecord, isString, nonBlankString } from "../guards";
import { MAX_FRAMES } from "../limits";
import type { AudioItem, EditHandles, VisualEnvelope } from "../schema/timeline";
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
  type CardLine,
  type CompileInputs,
  type NodeCategory,
  type NodeDef,
} from "./define";
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

const titleKeys = (schema: ElementMeta["schema"]): string[] =>
  TEXT_KEYS.filter((k) => k in schema.shape);

function paletteRow(colors: unknown): CardRow | null {
  if (!Array.isArray(colors) || colors.length === 0 || !colors.every(isString)) return null;
  return row("palette", plural(colors.length, "colour"), {
    title: colors.join(", "),
    swatches: colors,
  });
}

function gradientRow(value: unknown): CardRow | null {
  if (!isString(value)) return null;
  const gradient = parseGradient(value);
  if (!gradient) return null;
  const stops = gradient.stops.map((s) => s.color);
  return row("blend", gradient.type === "linear" ? "Straight gradient" : "Round gradient", {
    title: value,
    swatches: stops,
  });
}

function themeRow(value: unknown): CardRow | null {
  if (!isThemeValue(value)) return null;
  const changed = Object.values(themeColors(value));
  const mode = value.mode === "light" ? "Light theme" : "Dark theme";
  return row(
    value.mode === "light" ? "sun" : "moon",
    changed.length ? `${mode} · ${plural(changed.length, "colour")} changed` : mode,
    changed.length ? { swatches: changed } : {},
  );
}

function lookRow(meta: ElementMeta, params: ElementParams): CardRow | null {
  for (const [key, field] of shapeEntries(meta.schema)) {
    const inner = unwrap(field);
    const { widget, showIf } = fieldMeta(field);
    const shown = Object.entries(showIf ?? {}).every(([k, allowed]) =>
      allowed.includes(String(params[k] ?? "")),
    );
    if (!shown) continue;
    const found =
      inner instanceof z.ZodArray &&
      inner.element instanceof z.ZodType &&
      isColorSchema(inner.element)
        ? paletteRow(params[key])
        : widget === "gradient"
          ? gradientRow(params[key])
          : widget === "theme"
            ? themeRow(params[key])
            : null;
    if (found) return found;
  }
  return null;
}

function elementCard(
  meta: ElementMeta,
  titles: string[],
  params: ElementParams,
  fps: number,
): CardRow[] {
  const rows: CardRow[] = [];
  const title = titles.map((k) => nonBlankString(params[k])).find(Boolean);
  if (title) rows.push(row("type", quote(title)));
  const file = assetKeys(meta.schema)
    .map((k) => nonBlankString(params[k]))
    .find(Boolean);
  if (file) rows.push(fileRow("file", file));
  const look = lookRow(meta, params);
  if (look) rows.push(look);
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

function settingValue(meta: FieldMeta, value: unknown): unknown {
  if (meta.zeroLabel && value === "") return undefined;
  if (meta.widget === "asset" && isString(value)) return projectRef(value);
  return value;
}

function elementProps(meta: ElementMeta, params: Record<string, unknown>): Record<string, unknown> {
  const mapped = mapLeaves(meta.schema, params, settingValue);
  const props: Record<string, unknown> = {
    ...meta.fixedProps,
    ...(isRecord(mapped) ? mapped : {}),
  };
  for (const key of widgetKeys(meta.schema, "font")) {
    const stack = fontProp(props[key]);
    if (stack) props[key] = stack;
    else delete props[key];
  }
  return props;
}

export const soundVolumeKey = (soundKey: string): string => `${soundKey}Volume`;

function elementSounds(
  meta: ElementMeta,
  nodeId: string,
  props: Record<string, unknown>,
  frames: number,
): AudioItem[] {
  return widgetKeys(meta.schema, "sound").flatMap((key): AudioItem[] => {
    const src = props[key];
    const volume = props[soundVolumeKey(key)];
    delete props[key];
    delete props[soundVolumeKey(key)];
    if (!isString(src) || !src) return [];
    return [
      {
        id: `${nodeId}:${key}`,
        src: projectRef(src),
        from: 0,
        durationInFrames: frames,
        volume: isNumber(volume) ? volume : 1,
        nodeId,
        channel: nodeId,
        role: "music",
        handles: { volume: { nodeId, param: soundVolumeKey(key), unit: "number" } },
      },
    ];
  });
}

function compileElement(
  meta: ElementMeta,
  nodeId: string,
  { clipDuration, envelope, ...params }: ElementParams,
  input: ClipValue | undefined,
): ClipValue {
  const props = elementProps(meta, params);
  const handles: EditHandles = { trimEnd: { nodeId, param: "clipDuration", unit: "frames" } };
  if (meta.kind !== "wrapper" || !input) {
    const frames = clipDuration || meta.defaultDurationInFrames;
    const audio = elementSounds(meta, nodeId, props, frames);
    return itemClip(nodeId, meta.key, props, frames, { handles, envelope, audio });
  }
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
    category: NODE_CATEGORY_FOR[meta.category],
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
}: CardContext<SequencerParams>): CardLine[] {
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
    category: "Timeline",
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
