import { z } from "zod";
import { fieldMeta, type FieldMeta } from "../elements/introspect";
import { humanize, signedText } from "../format";
import { isKeyOf } from "../guards";
import { MAX_FRAMES, MAX_SPEED, MAX_VOLUME } from "../limits";
import type { MinMax } from "../math";
import { audioEnvelopeSchema, visualEnvelopeSchema } from "../schema/timeline";
import { PAUSE_MARK, tokenize } from "../vars";

export const more = <T extends z.ZodType>(schema: T, extra: FieldMeta): T =>
  schema.meta({ ...fieldMeta(schema), ...extra });

const target = (description = "Element", help = "Pick it on the page, or type its visible text.") =>
  z.string().min(1, "Pick an element").meta({ description, help, widget: "target" });

export const optionalTarget = (description: string, help?: string) =>
  z
    .string()
    .default("")
    .meta({ description, widget: "target", ...(help ? { help } : {}) });

export const pickedRole = () =>
  z.string().optional().meta({ description: "Role of the picked element", hidden: true });

type NumberOptions = FieldMeta & MinMax;

export const num = (def: number, description: string, { min, max, ...meta }: NumberOptions) =>
  z
    .number()
    .min(min)
    .max(max)
    .default(def)
    .meta({ description, ...meta });

export const int = (def: number, description: string, { min, max, ...meta }: NumberOptions) =>
  z
    .number()
    .int()
    .min(min)
    .max(max)
    .default(def)
    .meta({ description, ...meta });

export const ms = (def: number, description: string, max = 600_000) =>
  int(def, description, { min: 0, max, step: 50, unit: "ms" });

export const frames = (def: number, description: string) =>
  int(def, description, { min: 1, max: MAX_FRAMES, step: 1, unit: "frames" });

export const fraction = (def: number, description: string) =>
  num(def, description, { min: 0, max: 1, step: 0.01 });

export const rate = (
  def: number,
  description: string,
  { min, max = MAX_SPEED, step = 0.05 }: { min: number; max?: number; step?: number },
) => num(def, description, { min, max, step, unit: "×" });

export const volume = (def: number) => num(def, "Volume", { min: 0, max: MAX_VOLUME, step: 0.05 });

export const zoomFactor = (def: number, description = "Zoom", step = 0.05) =>
  num(def, description, { min: 1.1, max: 2.5, step, unit: "×" });

export const text = (def: string, description: string, widget?: "textarea" | "code") =>
  z
    .string()
    .default(def)
    .meta({ description, ...(widget ? { widget } : {}) });

export const requiredText = (description: string, missing: string, def = "") =>
  z
    .string()
    .refine((t) => t.replaceAll(PAUSE_MARK, "").trim() !== "", missing)
    .default(def)
    .meta({ description });

export const asset = (description: string, accept = "*/*") =>
  z.string().default("").meta({ description, widget: "asset", accept });

export const bool = (def: boolean, description: string) =>
  z.boolean().default(def).meta({ description });

export function keysOf<K extends string>(map: Record<K, unknown>): [K, ...K[]] {
  const [first, ...rest] = Object.keys(map).filter((key) => isKeyOf(map, key));
  if (first === undefined) throw new Error("keysOf needs at least one key.");
  return [first, ...rest];
}

export const choice = <const T extends readonly [string, ...string[]]>(
  values: T,
  def: T[number],
  description: string,
  labels?: Partial<Record<T[number], string>>,
) =>
  z
    .enum(values)
    .default(def)
    .meta({ description, ...(labels ? { labels } : {}) });

type ChoiceEntry = string | { label: string };

const labelOf = (entry: ChoiceEntry): string => (typeof entry === "string" ? entry : entry.label);

function labelsOf<K extends string>(table: Record<K, ChoiceEntry>): Partial<Record<K, string>> {
  const labels: Partial<Record<K, string>> = {};
  for (const key of keysOf(table)) labels[key] = labelOf(table[key]);
  return labels;
}

export const choiceOf = <K extends string>(
  table: Record<K, ChoiceEntry>,
  def: NoInfer<K>,
  description: string,
) => choice(keysOf(table), def, description, labelsOf(table));

export const FORMAT_NAMES = {
  h264: "MP4",
  h265: "MP4 (HEVC)",
  vp8: "WebM (VP8)",
  vp9: "WebM",
  prores: "ProRes",
  gif: "GIF",
};

export const QUALITY_PRESETS = { Low: 28, Medium: 23, High: 18, Max: 12 } as const;

export function qualityName(crf: number): string {
  const [[firstName, firstCrf], ...rest] = Object.entries(QUALITY_PRESETS);
  let bestName = firstName;
  let bestCrf: number = firstCrf;
  for (const [name, presetCrf] of rest) {
    const distance = Math.abs(presetCrf - crf);
    const bestDistance = Math.abs(bestCrf - crf);
    const finer = presetCrf < bestCrf;
    if (distance < bestDistance || (distance === bestDistance && finer)) {
      bestName = name;
      bestCrf = presetCrf;
    }
  }
  return bestName;
}

export function choiceFrom<K extends string, const T extends readonly [K, ...K[]]>(
  labels: Record<K, string>,
  values: T,
  def: T[number],
  description: string,
) {
  const shown: Partial<Record<T[number], string>> = {};
  for (const value of values) shown[value] = labels[value];
  return choice(values, def, description, shown);
}

export const LOUDNESS_PRESETS = { "Online video": -14, Podcast: -16, Broadcast: -23 } as const;

export const loudnessName = (lufs: number): string =>
  Object.entries(LOUDNESS_PRESETS).find(([, v]) => v === lufs)?.[0] ?? `${signedText(lufs)} LUFS`;

export const quality = (def: number, description = "Quality") =>
  int(def, description, { min: 0, max: 51, widget: "quality" });

export const targetFields = (description?: string, help?: string) => ({
  target: target(description, help),
  targetRole: pickedRole(),
});

export const pauseAfter = {
  after: more(ms(600, "Pause after"), {
    help: "How long the video lingers after this action.",
  }),
};

const zoomFields = {
  cluster: more(text("", "Zoom group"), {
    help: "Actions in the same group share one zoom instead of zooming in and out each time.",
  }),
  zoom: bool(true, "Zoom in on this action"),
};

export const beatFields = { ...pauseAfter, ...zoomFields };

export const frameFields = {
  frame: more(
    optionalTarget(
      "Zoom to fit",
      "Zoom to a bigger area, like the whole card, instead of the element.",
    ),
    { advanced: true },
  ),
  frameRole: pickedRole(),
};

const pointerFields = {
  ...frameFields,
  zoomScale: num(0, "Zoom amount", {
    min: 0,
    max: 3,
    step: 0.05,
    unit: "×",
    zeroLabel: "Auto",
    advanced: true,
  }),
  travelMs: more(ms(0, "Cursor travel time", 10_000), { zeroLabel: "Auto", advanced: true }),
};

export const pointerAction = (hoverMs: number, hoverLabel: string) => ({
  ...beatFields,
  ...pointerFields,
  hoverMs: ms(hoverMs, hoverLabel, 10_000),
});

export const visualEnvelope = () =>
  visualEnvelopeSchema.default({}).meta({
    description: "Keyframes",
    help: "Change opacity, position and size over time. Edit them here or on the timeline.",
    widget: "envelope",
  });

export const audioEnvelope = () =>
  audioEnvelopeSchema.default({}).meta({
    description: "Keyframes",
    help: "Change volume and left–right balance over time. Edit them here or on the timeline.",
    widget: "envelope",
  });

type ParsedParams<T> = { ok: true; data: T } | { ok: false; problem: string };

const hasVariable = (input: unknown): boolean =>
  typeof input === "string" && tokenize(input).some((t) => t.kind === "var");

function allowedValues(issue: z.core.$ZodIssue): unknown[] | null {
  if (issue.code !== "invalid_union") return null;
  const nested = issue.errors.flat();
  if (!nested.length || !nested.every((i) => i.code === "invalid_value")) return null;
  return nested.flatMap((i) => (i.code === "invalid_value" ? i.values : []));
}

function choiceList(values: unknown[], unit: string | undefined): string {
  const words = values.map(String);
  const last = words.pop();
  const list = words.length ? `${words.join(", ")} or ${last}` : String(last);
  return unit ? `${list} ${unit}` : list;
}

function issueWording(
  issue: z.core.$ZodIssue,
  { unit, written }: { unit?: string; written?: unknown } = {},
): string | null {
  const allowed = allowedValues(issue);
  if (allowed) return `must be ${choiceList(allowed, unit)}`;
  switch (issue.code) {
    case "too_small":
      return issue.origin === "string" && Number(issue.minimum) === 1 ? "is empty" : "is too small";
    case "too_big":
      return "is too large";
    case "invalid_type":
      if (issue.input === undefined) return "is missing";
      return issue.expected === "number" && hasVariable(written ?? issue.input)
        ? "takes a number. Variables work only in text settings"
        : "isn't valid";
    case "invalid_value":
      return "has a value that isn't allowed";
    case "custom":
      return null;
    default:
      return "isn't valid";
  }
}

export function parseParams<T extends z.ZodObject>(
  schema: T,
  values: unknown,
  written?: Record<string, unknown>,
): ParsedParams<z.output<T>> {
  const parsed = schema.safeParse(values, { reportInput: true });
  if (parsed.success) return { ok: true, data: parsed.data };
  const issue = parsed.error.issues[0];
  const first = issue.path.at(0);
  const key = typeof first === "string" ? first : "";
  const field: z.ZodType | undefined = schema.shape[key];
  const meta = field ? fieldMeta(field) : undefined;
  const wording = issueWording(issue, { unit: meta?.unit, written: written?.[key] });
  if (wording === null) return { ok: false, problem: issue.message };
  const label = meta?.description || (key ? humanize(key) : "A setting");
  return { ok: false, problem: `${label} ${wording}` };
}
