import { z } from "zod";
import { mapItems } from "../compile/clip";
import { styleStep, type StyleSteps } from "../compile/style";
import { formatDuration, percentText, plural } from "../format";
import type { BlurArea, PictureLook } from "../schema/recording";
import type { ClipValue } from "../schema/values";
import { missing, row, type CardRow, type InputTake } from "./card";
import { defineNode, modifierBase, RECORDING_PORT, recordingModifier, withClip } from "./define";
import { more, num } from "./params";

const MAX_BLUR_OF_WIDTH = 0.04;
const MAX_SOFTNESS_OF_WIDTH = 0.02;
const MAX_TAKE_MS = 3_600_000;

const areaSchema = z.object({
  x: z.number().min(0).max(1),
  y: z.number().min(0).max(1),
  width: z.number().min(0).max(1),
  height: z.number().min(0).max(1),
});

const optionalTime = (description: string, zeroLabel: string) =>
  z.number().int().min(0).max(MAX_TAKE_MS).optional().meta({ description, unit: "ms", zeroLabel });

function timesText(fromMs: number | undefined, untilMs: number | undefined): string | null {
  if (fromMs === undefined && untilMs === undefined) return null;
  if (untilMs === undefined) return `From ${formatDuration(fromMs ?? 0)}`;
  if (fromMs === undefined) return `Until ${formatDuration(untilMs)}`;
  return `${formatDuration(fromMs)} → ${formatDuration(untilMs)}`;
}

function afterTakeRow(take: InputTake | undefined, fromMs: number | undefined): CardRow | null {
  if (!take || fromMs === undefined || fromMs < take.durationMs) return null;
  return row("triangle-alert", `After the take ends at ${formatDuration(take.durationMs)}`, {
    tone: "warn",
  });
}

export const blurAreasNode = defineNode({
  ...recordingModifier("Effects"),
  type: "effect.blurAreas",
  title: "Blur areas",
  description:
    "Blurs parts of the browser window to hide private details. The blur stays on the page as the camera moves.",
  card: ({ params: p, inputTakes }) => [
    p.areas.length
      ? row("eye-off", `${plural(p.areas.length, "area")} · ${percentText(p.strength)} strength`)
      : missing("eye-off", "Draw an area to blur"),
    timesText(p.fromMs, p.untilMs) && row("clock", timesText(p.fromMs, p.untilMs) ?? ""),
    afterTakeRow(inputTakes?.[RECORDING_PORT], p.fromMs),
  ],
  params: z
    .object({
      areas: z
        .array(areaSchema)
        .max(8)
        .default([{ x: 0.62, y: 0.06, width: 0.3, height: 0.08 }])
        .meta({
          description: "Areas to blur",
          widget: "boxes",
          timeKey: "frameMs",
          help: "Drag on the picture to add an area. Move the slider to draw on a later moment of the take. The blur stays on that part of the window while the camera zooms and pans.",
        }),
      frameMs: more(num(0, "Frame to draw on", { min: 0, max: MAX_TAKE_MS }), {
        unit: "ms",
        hidden: true,
      }),
      strength: num(0.6, "Strength", { min: 0.05, max: 1, step: 0.05 }),
      softness: num(0.3, "Soft edge", { min: 0, max: 1, step: 0.05 }),
      roundness: num(0.3, "Rounded corners", { min: 0, max: 1, step: 0.05 }),
      fromMs: optionalTime("From", "Start of the take"),
      untilMs: optionalTime("Until", "End of the take"),
    })
    .refine((p) => p.fromMs === undefined || p.untilMs === undefined || p.untilMs > p.fromMs, {
      message: "Until must come after From",
      path: ["untilMs"],
    }),
});

export const blurAreasOf = (p: z.infer<typeof blurAreasNode.params>): BlurArea[] =>
  p.areas.map((area) => ({
    ...area,
    strength: p.strength * MAX_BLUR_OF_WIDTH,
    softness: p.softness * MAX_SOFTNESS_OF_WIDTH,
    roundness: p.roundness,
    ...(p.fromMs === undefined ? {} : { fromMs: p.fromMs }),
    ...(p.untilMs === undefined ? {} : { untilMs: p.untilMs }),
  }));

const NEUTRAL_LOOK = {
  brightness: 0,
  contrast: 1,
  saturation: 1,
  warmth: 0,
  vignette: 0,
  grain: 0,
} satisfies Required<PictureLook>;

export const LOOK_PRESETS: Array<{ label: string; values: Required<PictureLook> }> = [
  { label: "Natural", values: NEUTRAL_LOOK },
  { label: "Warm", values: { ...NEUTRAL_LOOK, warmth: 0.35, saturation: 1.1, vignette: 0.2 } },
  { label: "Cool", values: { ...NEUTRAL_LOOK, warmth: -0.3, contrast: 1.05 } },
  { label: "Punchy", values: { ...NEUTRAL_LOOK, contrast: 1.2, saturation: 1.25 } },
  { label: "Soft", values: { ...NEUTRAL_LOOK, contrast: 0.88, brightness: 0.1, saturation: 0.9 } },
  { label: "Black and white", values: { ...NEUTRAL_LOOK, saturation: 0, contrast: 1.15 } },
  {
    label: "Old film",
    values: { ...NEUTRAL_LOOK, warmth: 0.25, saturation: 0.75, vignette: 0.45, grain: 0.35 },
  },
];

const LOOK_COMPONENTS = new Set(["core/video", "core/image"]);

const changedLook = (look: Required<PictureLook>): PictureLook =>
  Object.fromEntries(
    Object.entries(look).filter(([key, value]) => NEUTRAL_LOOK[key as keyof PictureLook] !== value),
  );

function withLook(clip: ClipValue, look: PictureLook): ClipValue {
  return {
    ...clip,
    tracks: mapItems(clip.tracks, (item) => {
      if (item.kind === "recording") return { ...item, props: { ...item.props, look } };
      return item.kind === "element" && LOOK_COMPONENTS.has(item.component)
        ? { ...item, props: { ...item.props, look } }
        : item;
    }),
  };
}

export const pictureLookNode = defineNode({
  ...modifierBase("Effects"),
  type: "effect.look",
  title: "Picture look",
  description:
    "Gives recordings, videos and images a look: colour, brightness, a dark vignette and film grain. To grade a recording's whole picture, window and backdrop included, use Apply grade on an exported file.",
  card: ({ params: p }) => {
    const preset = LOOK_PRESETS.find((l) =>
      Object.entries(l.values).every(([k, v]) => p[k as keyof PictureLook] === v),
    );
    const changed = Object.keys(changedLook(p)).length;
    return [
      row(
        "sparkles",
        preset ? preset.label : plural(changed, "change"),
        preset?.label === "Natural" ? { tone: "muted" as const } : {},
      ),
    ];
  },
  params: z.object({
    brightness: more(num(0, "Brightness", { min: -1, max: 1, step: 0.05 }), {
      presets: LOOK_PRESETS,
      presetsLabel: "Look",
    }),
    contrast: num(1, "Contrast", { min: 0.5, max: 1.5, step: 0.05, unit: "×" }),
    saturation: num(1, "Colour", { min: 0, max: 2, step: 0.05, unit: "×" }),
    warmth: num(0, "Warmth", { min: -1, max: 1, step: 0.05 }),
    vignette: num(0, "Dark edges", { min: 0, max: 1, step: 0.05 }),
    grain: num(0, "Film grain", { min: 0, max: 1, step: 0.05 }),
  }),
  compile: ({ inputs, params }) => withClip(inputs, (clip) => withLook(clip, changedLook(params))),
});

export const effectNodes = [blurAreasNode, pictureLookNode];

export const effectStyles: StyleSteps = [
  styleStep(blurAreasNode, {
    restyle: (props, p) => ({
      ...props,
      blurAreas: [...(props.blurAreas ?? []), ...blurAreasOf(p)],
    }),
  }),
];
