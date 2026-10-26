import { z } from "zod";

type FieldPreset = { label: string; values: Record<string, unknown> };

export const FIELD_WIDGETS = [
  "action",
  "asset",
  "backdrop",
  "boxes",
  "camera",
  "code",
  "color",
  "datetime",
  "envelope",
  "font",
  "gradient",
  "keys",
  "loudness",
  "multiselect",
  "offsets",
  "quality",
  "sound",
  "spot",
  "target",
  "textarea",
  "theme",
  "timezone",
  "transition",
  "variable",
] as const;

export type FieldWidget = (typeof FIELD_WIDGETS)[number];

const THEME_MODES = ["light", "dark"] as const;

type ThemeMode = (typeof THEME_MODES)[number];

export type FieldMeta = {
  description?: string;
  help?: string;
  widget?: FieldWidget;
  step?: number;
  unit?: string;
  accept?: string;
  advanced?: boolean;
  hidden?: boolean;
  labels?: Partial<Record<string, string>>;
  zeroLabel?: string;
  unconnectedZeroLabel?: string;
  presets?: FieldPreset[];
  presetsLabel?: string;
  pauses?: boolean;
  timeKey?: string;
  lengthKey?: string;
  showIf?: Record<string, string[]>;
  showIfFile?: Record<string, Array<"image" | "video" | "audio">>;
  themes?: Partial<Record<ThemeMode, Record<string, string>>>;
  framesAt?: number;
};

declare module "zod" {
  interface GlobalMeta extends FieldMeta {}
}

export const themeValueSchema = z.object({
  mode: z.enum(THEME_MODES),
  colors: z.record(z.string(), z.string()),
});

export type ElementCategory =
  | "core"
  | "text"
  | "overlay"
  | "transition"
  | "background"
  | "effect"
  | "ui"
  | "data"
  | "scene"
  | "device";

export type ElementKind = "component" | "wrapper" | "presentation" | "sequencer";

export type ElementProps = Record<string, unknown>;

export interface ElementMeta {
  key: string;
  title: string;
  description?: string;
  category: ElementCategory;
  kind: ElementKind;
  source: { registry: string; url: string; license: string };
  schema: z.ZodObject;
  defaultProps: ElementProps;
  fixedProps: ElementProps;
  defaultDurationInFrames: number;
  design?: { width: number; height: number; exact?: boolean };
  optionalClip?: boolean;
  thumbnail?: string;
}

export type ElementMetaInput = Omit<ElementMeta, "defaultProps">;

export function defineElementMeta(def: ElementMetaInput): ElementMeta {
  return { ...def, defaultProps: def.schema.parse({}) };
}
