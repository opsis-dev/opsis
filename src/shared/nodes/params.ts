import { z } from "zod";
import { fieldMeta, type FieldMeta } from "../elements/introspect";
import { audioEnvelopeSchema, visualEnvelopeSchema } from "../schema/timeline";

// oxlint-disable-next-line typescript/no-explicit-any
export const more = <T extends z.ZodType<any>>(schema: T, extra: FieldMeta): T =>
  schema.meta({ ...fieldMeta(schema), ...extra }) as T;

export const target = (
  description = "Element",
  help = "Pick it on the page, or type its visible text.",
) => z.string().min(1, "Pick an element").meta({ description, help, widget: "target" });

export const optionalTarget = (description: string, help?: string) =>
  z
    .string()
    .default("")
    .meta({ description, widget: "target", ...(help ? { help } : {}) });

export const ms = (def: number, description: string, max = 600_000) =>
  z.number().int().min(0).max(max).default(def).meta({ description, step: 50, unit: "ms" });

export const frames = (def: number, description: string) =>
  z.number().int().min(1).max(108_000).default(def).meta({ description, step: 1, unit: "frames" });

export const fraction = (def: number, description: string) =>
  z.number().min(0).max(1).default(def).meta({ description, step: 0.01 });

export const text = (def: string, description: string, widget?: "textarea" | "code") =>
  z
    .string()
    .default(def)
    .meta({ description, ...(widget ? { widget } : {}) });

export const asset = (description: string, accept = "*/*") =>
  z.string().default("").meta({ description, widget: "asset", accept });

export const bool = (def: boolean, description: string) =>
  z.boolean().default(def).meta({ description });

export const choice = <const T extends readonly [string, ...string[]]>(
  values: T,
  def: T[number],
  description: string,
  labels?: Partial<Record<T[number], string>>,
) =>
  z
    .enum(values)
    .default(def)
    .meta({ description, ...(labels ? { labels: labels as Record<string, string> } : {}) });

export const QUALITY_PRESETS = { Low: 28, Medium: 23, High: 18, Max: 12 } as const;

export const quality = (def: number, description = "Quality") =>
  z.number().int().min(0).max(51).default(def).meta({ description, widget: "quality" });

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
  frame: optionalTarget(
    "Zoom to fit",
    "Zoom to a bigger area, like the whole card, instead of the element.",
  ).meta({
    description: "Zoom to fit",
    widget: "target",
    help: "Zoom to a bigger area, like the whole card, instead of the element.",
    advanced: true,
  }),
  zoomScale: z.number().min(0).max(3).default(0).meta({
    description: "Zoom amount",
    step: 0.05,
    unit: "×",
    zeroLabel: "Auto",
    advanced: true,
  }),
  travelMs: more(ms(0, "Cursor travel time", 10_000), { zeroLabel: "Auto", advanced: true }),
  hoverMs: ms(220, "Hover before clicking", 10_000),
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
