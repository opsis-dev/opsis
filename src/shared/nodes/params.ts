import { z } from "zod";
import { fieldMeta, type FieldMeta } from "../elements/introspect";
import { humanize } from "../format";
import { audioEnvelopeSchema, visualEnvelopeSchema } from "../schema/timeline";

export const more = <T extends z.ZodType>(schema: T, extra: FieldMeta): T =>
  schema.meta({ ...fieldMeta(schema), ...extra });

export const target = (
  description = "Element",
  help = "Pick it on the page, or type its visible text.",
) => z.string().min(1, "Pick an element").meta({ description, help, widget: "target" });

export const optionalTarget = (description: string, help?: string) =>
  z
    .string()
    .default("")
    .meta({ description, widget: "target", ...(help ? { help } : {}) });

export const pickedRole = () =>
  z.string().optional().meta({ description: "Role of the picked element", hidden: true });

export const MIN_SPEED = 0.1;
export const MAX_SPEED = 64;
export const MAX_VOLUME = 2;
export const MAX_FRAMES = 108_000;

type IntOptions = FieldMeta & { min: number; max: number };

export const int = (def: number, description: string, { min, max, ...meta }: IntOptions) =>
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
  z.number().min(0).max(1).default(def).meta({ description, step: 0.01 });

export const rate = (
  def: number,
  description: string,
  { min, max = MAX_SPEED, step = 0.05 }: { min: number; max?: number; step?: number },
) => z.number().min(min).max(max).default(def).meta({ description, step, unit: "×" });

export const volume = (def: number) =>
  z.number().min(0).max(MAX_VOLUME).default(def).meta({ description: "Volume", step: 0.05 });

export const zoomScale = (def: number, description = "Zoom", step = 0.05) =>
  z.number().min(1.1).max(2.5).default(def).meta({ description, step, unit: "×" });

export const text = (def: string, description: string, widget?: "textarea" | "code") =>
  z
    .string()
    .default(def)
    .meta({ description, ...(widget ? { widget } : {}) });

export const asset = (description: string, accept = "*/*") =>
  z.string().default("").meta({ description, widget: "asset", accept });

export const bool = (def: boolean, description: string) =>
  z.boolean().default(def).meta({ description });

export const keysOf = <K extends string>(map: Record<K, unknown>): [K, ...K[]] =>
  Object.keys(map) as [K, ...K[]];

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

export const LOUDNESS_PRESETS = { "Online video": -14, Podcast: -16, Broadcast: -23 } as const;

export const lufsText = (lufs: number): string => `${lufs < 0 ? "−" : ""}${Math.abs(lufs)}`;

export const loudnessName = (lufs: number): string =>
  Object.entries(LOUDNESS_PRESETS).find(([, v]) => v === lufs)?.[0] ?? `${lufsText(lufs)} LUFS`;

export const quality = (def: number, description = "Quality") =>
  int(def, description, { min: 0, max: 51, widget: "quality" });

export const beatFields = {
  after: more(ms(600, "Pause after"), {
    help: "How long the video lingers after this action.",
  }),
  cluster: more(text("", "Zoom group"), {
    help: "Actions in the same group share one zoom instead of zooming in and out each time.",
  }),
  zoom: bool(true, "Zoom in on this action"),
};

export const pointerFields = {
  frame: more(
    optionalTarget(
      "Zoom to fit",
      "Zoom to a bigger area, like the whole card, instead of the element.",
    ),
    { advanced: true },
  ),
  frameRole: pickedRole(),
  zoomScale: z.number().min(0).max(3).default(0).meta({
    description: "Zoom amount",
    step: 0.05,
    unit: "×",
    zeroLabel: "Auto",
    advanced: true,
  }),
  travelMs: more(ms(0, "Cursor travel time", 10_000), { zeroLabel: "Auto", advanced: true }),
};

export const visualEnvelope = () =>
  visualEnvelopeSchema.default({}).meta({
    description: "Keyframes",
    help: "Change opacity, position and size over time. Edit them here or on the timeline.",
    widget: "envelope",
    labels: { opacity: "Opacity", x: "Move across", y: "Move up and down", scale: "Size" },
  });

export const audioEnvelope = () =>
  audioEnvelopeSchema.default({}).meta({
    description: "Keyframes",
    help: "Change volume and left–right balance over time. Edit them here or on the timeline.",
    widget: "envelope",
    labels: { volume: "Volume", pan: "Balance" },
  });

type ParsedParams<T> = { ok: true; data: T } | { ok: false; problem: string };

function issueWording(issue: z.core.$ZodIssue): string | null {
  switch (issue.code) {
    case "too_small":
      return issue.origin === "string" && Number(issue.minimum) === 1 ? "is empty" : "is too small";
    case "too_big":
      return "is too large";
    case "invalid_type":
      return issue.input === undefined ? "is missing" : "isn't valid";
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
): ParsedParams<z.output<T>> {
  const parsed = schema.safeParse(values, { reportInput: true });
  if (parsed.success) return { ok: true, data: parsed.data };
  const issue = parsed.error.issues[0];
  const wording = issueWording(issue);
  if (wording === null) return { ok: false, problem: issue.message };
  const key = String(issue.path[0] ?? "");
  const field: z.ZodType | undefined = schema.shape[key];
  const label =
    (field ? fieldMeta(field).description : undefined) || (key ? humanize(key) : "A setting");
  return { ok: false, problem: `${label} ${wording}` };
}
