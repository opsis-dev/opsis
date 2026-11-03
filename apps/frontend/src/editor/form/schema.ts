import { isRecord, isString } from "@opsis/video/common/guards";
import type { FieldWidget, ThemeMode } from "@opsis/video/generated/spec";

type FieldPreset = { label: string; values: Record<string, unknown> };

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
  showIfTake?: Array<"browser" | "terminal">;
  themes?: Partial<Record<ThemeMode, Record<string, string>>>;
};

export type JsonSchema = {
  type?: string | string[];
  enum?: unknown[];
  properties?: Record<string, JsonSchema>;
  items?: JsonSchema;
  minimum?: number;
  maximum?: number;
  minItems?: number;
  maxItems?: number;
  default?: unknown;
  description?: string;
  [keyword: `x-${string}`]: unknown;
};

export const ANY_VALUE = "*";

const META_KEYWORDS: Record<string, keyof FieldMeta> = {
  "x-help": "help",
  "x-widget": "widget",
  "x-step": "step",
  "x-unit": "unit",
  "x-accept": "accept",
  "x-advanced": "advanced",
  "x-hidden": "hidden",
  "x-labels": "labels",
  "x-zero-label": "zeroLabel",
  "x-unconnected-zero-label": "unconnectedZeroLabel",
  "x-presets": "presets",
  "x-presets-label": "presetsLabel",
  "x-pauses": "pauses",
  "x-time-key": "timeKey",
  "x-length-key": "lengthKey",
  "x-show-if": "showIf",
  "x-show-if-file": "showIfFile",
  "x-show-if-take": "showIfTake",
  "x-themes": "themes",
};

export function asSchema(value: unknown): JsonSchema {
  return isRecord(value) ? value : {};
}

export function fieldMeta(schema: JsonSchema): FieldMeta {
  const out: Record<string, unknown> = {};
  if (isString(schema.description)) out.description = schema.description;
  for (const [keyword, key] of Object.entries(META_KEYWORDS))
    if (schema[keyword as `x-${string}`] !== undefined) out[key] = schema[keyword as `x-${string}`];
  return out as FieldMeta;
}

export function targetRoleKey(targetKey: string): string {
  return `${targetKey}Role`;
}

export function spotYKey(spotXKey: string): string {
  return spotXKey === "x" ? "y" : spotXKey.replace(/X$/, "Y");
}

export function shapeEntries(schema: JsonSchema): Array<[string, JsonSchema]> {
  return Object.entries(schema.properties ?? {});
}

export function schemaDefaults(schema: JsonSchema): Record<string, unknown> {
  return isRecord(schema.default) ? structuredClone(schema.default) : {};
}

export function withDefaults(
  schema: JsonSchema,
  values: Record<string, unknown>,
): Record<string, unknown> {
  const out = schemaDefaults(schema);
  for (const [key, value] of Object.entries(values)) if (value !== undefined) out[key] = value;
  return out;
}

function typesOf(schema: JsonSchema): string[] {
  return Array.isArray(schema.type) ? schema.type : schema.type ? [schema.type] : [];
}

export function takesNull(schema: JsonSchema): boolean {
  return typesOf(schema).includes("null");
}

export function baseType(schema: JsonSchema): string | undefined {
  return typesOf(schema).find((t) => t !== "null");
}

const COLOUR_RULE = "colour";

export function isColorSchema(schema: JsonSchema): boolean {
  const check = schema["x-check"];
  return schema["x-widget"] === "color" || (isRecord(check) && check.rule === COLOUR_RULE);
}
