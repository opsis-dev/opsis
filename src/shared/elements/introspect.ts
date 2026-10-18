import { z } from "zod";

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

declare module "zod" {
  interface GlobalMeta extends FieldMeta {}
}

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
  timeKey?: string;
  showIf?: Record<string, string[]>;
};

export const ANY_VALUE = "*";

const WRAPPERS: ReadonlySet<z.core.$ZodTypeDef["type"]> = new Set([
  "default",
  "optional",
  "nullable",
  "readonly",
  "catch",
  "prefault",
  "nonoptional",
]);

function innerOf(schema: z.ZodType): z.ZodType | undefined {
  const { def } = schema;
  if (!WRAPPERS.has(def.type) || !("innerType" in def)) return undefined;
  return def.innerType instanceof z.ZodType ? def.innerType : undefined;
}

function chainOf(schema: z.ZodType): [z.ZodType, ...z.ZodType[]] {
  const chain: [z.ZodType, ...z.ZodType[]] = [schema];
  for (let inner = innerOf(schema); inner; inner = innerOf(inner)) chain.push(inner);
  return chain;
}

export function fieldMeta(schema: z.ZodType): FieldMeta {
  const out: FieldMeta = {};
  for (const s of chainOf(schema).toReversed()) Object.assign(out, s.meta());
  return out;
}

export function unwrap(schema: z.ZodType): z.ZodType {
  const chain = chainOf(schema);
  return chain[chain.length - 1];
}

export const targetRoleKey = (targetKey: string): string => `${targetKey}Role`;

export const spotYKey = (spotXKey: string): string =>
  spotXKey === "x" ? "y" : spotXKey.replace(/X$/, "Y");

export const shapeEntries = (schema: z.ZodObject): Array<[string, z.ZodType]> =>
  Object.entries(schema.shape);

export const widgetKeys = (schema: z.ZodObject, widget: FieldWidget): string[] =>
  shapeEntries(schema)
    .filter(([, s]) => fieldMeta(s).widget === widget)
    .map(([k]) => k);

export const assetKeys = (schema: z.ZodObject): string[] => widgetKeys(schema, "asset");

export const hasWidget = (schema: z.ZodObject, widget: FieldWidget): boolean =>
  widgetKeys(schema, widget).length > 0;

export function schemaDefaults(schema: z.ZodObject): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const [key, field] of shapeEntries(schema)) {
    const parsed = field.safeParse(undefined);
    if (parsed.success && parsed.data !== undefined) out[key] = parsed.data;
  }
  return out;
}

export function withDefaults(
  schema: z.ZodObject,
  values: Record<string, unknown>,
): Record<string, unknown> {
  const out = schemaDefaults(schema);
  for (const [key, value] of Object.entries(values)) if (value !== undefined) out[key] = value;
  return out;
}
