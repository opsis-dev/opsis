import type { z } from "zod";

type FieldPreset = { label: string; values: Record<string, unknown> };

export const FIELD_WIDGETS = [
  "action",
  "asset",
  "backdrop",
  "boxes",
  "code",
  "color",
  "datetime",
  "envelope",
  "font",
  "keys",
  "loudness",
  "multiselect",
  "offsets",
  "quality",
  "secret",
  "sound",
  "spot",
  "target",
  "textarea",
  "timezone",
  "transition",
  "variable",
] as const;

export type FieldWidget = (typeof FIELD_WIDGETS)[number];

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
  presets?: FieldPreset[];
  presetsLabel?: string;
  pauses?: boolean;
  showIf?: Record<string, string[]>;
};

const WRAPPERS = [
  "default",
  "optional",
  "nullable",
  "readonly",
  "catch",
  "prefault",
  "nonoptional",
];

const defOf = (schema: z.ZodType) => schema.def as { type: string; innerType?: z.ZodType };

export function fieldMeta(schema: z.ZodType): FieldMeta {
  const chain: z.ZodType[] = [];
  for (let cur: z.ZodType | undefined = schema; cur; cur = defOf(cur).innerType) chain.push(cur);
  const out: FieldMeta = {};
  for (const s of chain.toReversed()) Object.assign(out, s.meta() as FieldMeta | undefined);
  return out;
}

export function unwrap(schema: z.ZodType): z.ZodType {
  let cur = schema;
  for (let def = defOf(cur); def.innerType && WRAPPERS.includes(def.type); def = defOf(cur))
    cur = def.innerType;
  return cur;
}

export const targetRoleKey = (targetKey: string): string => `${targetKey}Role`;

export const spotYKey = (spotXKey: string): string =>
  spotXKey === "x" ? "y" : spotXKey.replace(/X$/, "Y");

export const widgetKeys = (schema: z.ZodObject, widget: FieldWidget): string[] =>
  Object.entries(schema.shape)
    .filter(([, s]) => fieldMeta(s).widget === widget)
    .map(([k]) => k);

export const assetKeys = (schema: z.ZodObject): string[] => widgetKeys(schema, "asset");

export const hasWidget = (schema: z.ZodObject, widget: FieldWidget): boolean =>
  widgetKeys(schema, widget).length > 0;

export function schemaDefaults(schema: z.ZodObject): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const [key, field] of Object.entries(schema.shape)) {
    const parsed = field.safeParse(undefined);
    if (parsed.success && parsed.data !== undefined) out[key] = parsed.data;
  }
  return out;
}
