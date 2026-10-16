import { z } from "zod";
import { fieldMeta, unwrap } from "@/shared/elements/introspect";
import { humanize } from "@/shared/format";

export type SchemaKind =
  | { kind: "boolean" }
  | { kind: "number"; min?: number; max?: number }
  | { kind: "string" }
  | { kind: "enum"; options: string[] }
  | { kind: "literals"; options: Array<string | number> }
  | { kind: "enumList"; options: string[] }
  | { kind: "objectList"; item: z.ZodObject }
  | { kind: "other" };

const REMOTION_COLOR = "__remotion-color";

const isLiteralValue = (v: unknown): v is string | number =>
  typeof v === "string" || typeof v === "number";

function arrayKind(element: z.core.$ZodType): SchemaKind {
  if (!(element instanceof z.ZodType)) return { kind: "other" };
  const inner = unwrap(element);
  if (inner instanceof z.ZodEnum) return { kind: "enumList", options: inner.options.map(String) };
  if (inner instanceof z.ZodObject) return { kind: "objectList", item: inner };
  return { kind: "other" };
}

function unionKind(options: readonly z.core.$ZodType[]): SchemaKind {
  const values = options
    .flatMap((o) => (o instanceof z.ZodLiteral ? [...o.values] : []))
    .filter(isLiteralValue);
  return values.length ? { kind: "literals", options: values } : { kind: "other" };
}

export function schemaKind(schema: z.ZodType): SchemaKind {
  const inner = unwrap(schema);
  if (inner.def.type === "string") return { kind: "string" };
  if (inner instanceof z.ZodBoolean) return { kind: "boolean" };
  if (inner instanceof z.ZodNumber)
    return { kind: "number", min: inner.minValue ?? undefined, max: inner.maxValue ?? undefined };
  if (inner instanceof z.ZodEnum) return { kind: "enum", options: inner.options.map(String) };
  if (inner instanceof z.ZodUnion) return unionKind(inner.options);
  if (inner instanceof z.ZodArray) return arrayKind(inner.element);
  return { kind: "other" };
}

export const isColor = (schema: z.ZodType): boolean =>
  unwrap(schema).description === REMOTION_COLOR || fieldMeta(schema).widget === "color";

export function labelFor(key: string, schema: z.ZodType): string {
  const d = fieldMeta(schema).description;
  return d && d !== REMOTION_COLOR ? d : humanize(key);
}
