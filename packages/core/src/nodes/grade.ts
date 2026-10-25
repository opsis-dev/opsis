import { z } from "zod";
import { baseName } from "../asset-ref";
import { rgbOf } from "../color";
import { percentText, plural } from "../format";
import { clamp, round3 } from "../math";
import type { StyleEntry } from "../schema/values";
import { fileRow, missing, row, truncate, type CardRow } from "./card";
import { defineNode, gradeModifier, type NodeDef } from "./define";
import { LOOK_PRESETS } from "./effects";
import type { NodeIconName } from "./icon-name";
import { asset, bool, choiceOf, colorOr, colorSetting, fraction, int, more, num } from "./params";

type Change = [label: string, active: boolean];

const GRADE_STEPS = new Map<
  string,
  { title: string; isNeutral: (params: Record<string, unknown>) => boolean }
>();

const activeChanges = (changes: readonly Change[]): string[] =>
  changes.filter(([, active]) => active).map(([label]) => label);

const changesRow = (icon: NodeIconName, changes: readonly Change[]): CardRow => {
  const names = activeChanges(changes);
  return names.length
    ? row(icon, truncate(`${plural(names.length, "change")} · ${names.join(", ")}`))
    : row(icon, "No change", { tone: "muted" });
};

function gradeStep<S extends z.ZodObject>(
  def: Omit<NodeDef<S>, "category" | "phase" | "inputs" | "outputs" | "compile">,
  isNeutral: (params: z.infer<S>) => boolean,
) {
  const node = defineNode<S>({ ...gradeModifier("Grade"), ...def });
  GRADE_STEPS.set(node.type, {
    title: node.title,
    isNeutral: (params) => isNeutral(node.params.parse(params)),
  });
  return node;
}

export const gradeStepTitle = (type: string): string => GRADE_STEPS.get(type)?.title ?? type;

export const isNeutralEntry = (entry: Pick<StyleEntry, "type" | "params">): boolean =>
  GRADE_STEPS.get(entry.type)?.isNeutral(entry.params) ?? true;

const needsMotion = (entry: Pick<StyleEntry, "type" | "params">): boolean => {
  if (entry.type === lensNode.type) return lensParams.parse(entry.params).trails > 0;
  if (entry.type === textureNode.type) return textureParams.parse(entry.params).grain > 0;
  return false;
};

export const gradeNeedsVideo = (entries: readonly Pick<StyleEntry, "type" | "params">[]): boolean =>
  entries.some(needsMotion);

export const gradeChangesNothing = (entries: readonly Pick<StyleEntry, "type" | "params">[]) =>
  entries.every(isNeutralEntry);

const colourPresets = LOOK_PRESETS.map(({ label, values }) => ({
  label,
  values: {
    brightness: values.brightness,
    contrast: values.contrast,
    saturation: values.saturation,
    warmth: values.warmth,
  },
}));

export const TONE_CURVES = {
  none: "None",
  vintage: "Vintage",
  cross_process: "Cross-process",
  lighter: "Lighter",
  darker: "Darker",
  medium_contrast: "More contrast",
  strong_contrast: "Strong contrast",
  negative: "Negative",
};

export function toneOffsets(color: string, amount: number): [number, number, number] | null {
  const rgb = rgbOf(color);
  if (!rgb || amount <= 0) return null;
  const mean = (rgb.r + rgb.g + rgb.b) / 3;
  const offset = (channel: number) => round3(clamp((channel - mean) * amount, -1, 1));
  const offsets: [number, number, number] = [offset(rgb.r), offset(rgb.g), offset(rgb.b)];
  return offsets.some((o) => o !== 0) ? offsets : null;
}

const toneAmount = (color: string, amount: number): boolean => toneOffsets(color, amount) !== null;

const colourChanges = (p: z.infer<typeof colourParams>): Change[] => [
  ["Brightness", p.brightness !== 0],
  ["Contrast", p.contrast !== 1],
  ["Colour", p.saturation !== 1],
  ["Gamma", p.gamma !== 1],
  ["Exposure", p.exposure !== 0],
  ["Warmth", p.warmth !== 0],
  ["Vibrance", p.vibrance !== 0],
  ["Hue", p.hue !== 0],
  ["Shadows", toneAmount(p.shadows, p.shadowsAmount)],
  ["Midtones", toneAmount(p.midtones, p.midtonesAmount)],
  ["Highlights", toneAmount(p.highlights, p.highlightsAmount)],
  ["Tone curve", p.curve !== "none"],
];

const colourParams = z.object({
  brightness: more(num(0, "Brightness", { min: -1, max: 1, step: 0.05 }), {
    presets: colourPresets,
    presetsLabel: "Look",
  }),
  contrast: num(1, "Contrast", { min: 0.5, max: 1.5, step: 0.05, unit: "×" }),
  saturation: num(1, "Colour", { min: 0, max: 2, step: 0.05, unit: "×" }),
  gamma: num(1, "Gamma", { min: 0.5, max: 2, step: 0.05 }),
  exposure: num(0, "Exposure", { min: -2, max: 2, step: 0.05, unit: "stops" }),
  warmth: num(0, "Warmth", { min: -1, max: 1, step: 0.05 }),
  vibrance: num(0, "Vibrance", { min: -1, max: 1, step: 0.05 }),
  hue: num(0, "Hue shift", { min: -180, max: 180, step: 1, unit: "°" }),
  shadows: colorSetting("#3b6ea5", "Colour for shadows"),
  shadowsAmount: num(0, "Shadows amount", { min: 0, max: 1, step: 0.05 }),
  midtones: colorSetting("#c9a27a", "Colour for midtones"),
  midtonesAmount: num(0, "Midtones amount", { min: 0, max: 1, step: 0.05 }),
  highlights: colorSetting("#ffd9a0", "Colour for highlights"),
  highlightsAmount: num(0, "Highlights amount", { min: 0, max: 1, step: 0.05 }),
  curve: choiceOf(TONE_CURVES, "none", "Tone curve"),
});

export const colourNode = gradeStep(
  {
    type: "grade.colour",
    title: "Colour and light",
    description:
      "Changes brightness, contrast, colour, warmth and the colour of shadows and highlights. Used by Apply grade.",
    card: ({ params: p }) => [
      changesRow("sun", colourChanges(p)),
      p.curve !== "none" && row("spline", TONE_CURVES[p.curve]),
    ],
    params: colourParams,
  },
  (p) => !activeChanges(colourChanges(p)).length,
);

const lookFileParams = z.object({
  file: more(asset("Look file", ".cube,.3dl,.png"), {
    help: "A colour look saved from another video editor, or downloaded. Files ending in .cube or .3dl, or a look image.",
  }),
  strength: num(1, "Strength", { min: 0, max: 1, step: 0.05 }),
});

function lookFileRow(file: string, missingFiles: readonly string[] = []): CardRow {
  if (!file) return missing("file", "Choose a look file");
  if (missingFiles.includes(file)) return missing("file", `${baseName(file)} isn't in the project`);
  return fileRow("file", file);
}

export const lookFileNode = gradeStep(
  {
    type: "grade.lookFile",
    title: "Look file",
    description: "Applies a colour look saved from another video editor. Used by Apply grade.",
    card: ({ params: p, missingFiles }) => [
      lookFileRow(p.file, missingFiles),
      p.strength < 1 && row("sparkles", `${percentText(p.strength)} strength`),
    ],
    params: lookFileParams,
  },
  (p) => p.strength === 0,
);

export const MONO_KINDS = { bw: "Black and white", sepia: "Sepia", tint: "Tint" };

export const DEFAULT_TINT = "#d8a15b";

const monoParams = z.object({
  kind: choiceOf(MONO_KINDS, "bw", "Kind"),
  tint: more(colorSetting(DEFAULT_TINT, "Tint colour"), { showIf: { kind: ["tint"] } }),
  strength: num(1, "Strength", { min: 0, max: 1, step: 0.05 }),
});

export const monoNode = gradeStep(
  {
    type: "grade.mono",
    title: "Black and white and tint",
    description:
      "Turns the picture black and white, sepia or one tint of colour. Used by Apply grade.",
    card: ({ params: p }) => [
      row(
        "moon",
        MONO_KINDS[p.kind],
        p.kind === "tint" ? { swatches: [colorOr(p.tint, DEFAULT_TINT)] } : {},
      ),
      p.strength < 1 && row("sparkles", `${percentText(p.strength)} strength`),
    ],
    params: monoParams,
  },
  (p) => p.strength === 0,
);

const textureChanges = (p: z.infer<typeof textureParams>): Change[] => [
  ["Sharpen", p.sharpen > 0],
  ["Soften", p.soften > 0],
  ["Dark edges", p.vignette > 0],
  ["Film grain", p.grain > 0],
];

const textureParams = z.object({
  sharpen: num(0, "Sharpen", { min: 0, max: 1, step: 0.05 }),
  soften: num(0, "Soften", { min: 0, max: 1, step: 0.05 }),
  vignette: num(0, "Dark edges", { min: 0, max: 1, step: 0.05 }),
  vignetteX: more(fraction(0.5, "Dark edges centre"), {
    widget: "spot",
    help: "Click the frame to choose the middle of the light.",
  }),
  vignetteY: more(fraction(0.5, "Dark edges centre vertical"), { hidden: true }),
  grain: num(0, "Film grain", { min: 0, max: 1, step: 0.05 }),
});

export const textureNode = gradeStep(
  {
    type: "grade.texture",
    title: "Texture",
    description:
      "Sharpens or softens the picture and adds dark edges and film grain. Used by Apply grade.",
    card: ({ params: p }) => [changesRow("sparkles", textureChanges(p))],
    params: textureParams,
  },
  (p) => !activeChanges(textureChanges(p)).length,
);

const lensChanges = (p: z.infer<typeof lensParams>): Change[] => [
  ["Colour fringing", p.fringe > 0],
  ["Lens bulge", p.bulge !== 0],
  ["Motion trails", p.trails > 0],
];

const lensParams = z.object({
  fringe: num(0, "Colour fringing", { min: 0, max: 1, step: 0.05 }),
  bulge: num(0, "Lens bulge", { min: -1, max: 1, step: 0.05 }),
  trails: num(0, "Motion trails", { min: 0, max: 1, step: 0.05 }),
});

export const lensNode = gradeStep(
  {
    type: "grade.lens",
    title: "Lens",
    description:
      "Adds colour fringing, bends the picture like a lens and leaves trails behind motion. Used by Apply grade.",
    card: ({ params: p }) => [changesRow("scan", lensChanges(p))],
    params: lensParams,
  },
  (p) => !activeChanges(lensChanges(p)).length,
);

const autoFixChanges = (p: z.infer<typeof autoFixParams>): Change[] => [
  ["White balance", p.whiteBalance],
  ["Stretch the levels", p.levels && p.levelsStrength > 0],
  ["Remove banding", p.deband],
];

const autoFixParams = z.object({
  whiteBalance: bool(false, "White balance"),
  levels: bool(false, "Stretch the levels"),
  levelsStrength: more(num(0.5, "Levels strength", { min: 0, max: 1, step: 0.05 }), {
    showIf: { levels: ["true"] },
  }),
  levelsSmoothing: more(int(24, "Levels smoothing", { min: 0, max: 300, unit: "frames" }), {
    advanced: true,
    showIf: { levels: ["true"] },
  }),
  deband: bool(false, "Remove banding"),
});

export const autoFixNode = gradeStep(
  {
    type: "grade.autoFix",
    title: "Auto fix",
    description:
      "Fixes a colour cast, stretches dull levels and smooths banding in skies. Used by Apply grade.",
    card: ({ params: p }) => [changesRow("check", autoFixChanges(p))],
    params: autoFixParams,
  },
  (p) => !activeChanges(autoFixChanges(p)).length,
);

export const gradeNodes = [colourNode, lookFileNode, monoNode, textureNode, lensNode, autoFixNode];
