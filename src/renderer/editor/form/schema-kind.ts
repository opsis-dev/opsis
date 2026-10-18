import { z } from "zod";
import { fieldMeta, unwrap } from "@/shared/elements/introspect";
import { isBoolean, isNumber, isRecord, isString, isStringOrNumber } from "@/shared/guards";
import { isBoundsList, type Bounds } from "@/shared/schema/actions";
import { isEnvelope, type Envelope } from "./envelope-field";
import { isColor } from "./schema-meta";
import type { ParamValues } from "./types";

type SchemaKind =
  | { kind: "boolean" }
  | { kind: "number"; min?: number; max?: number }
  | { kind: "string" }
  | { kind: "enum"; options: string[] }
  | { kind: "literals"; options: Array<string | number> }
  | { kind: "enumList"; options: string[] }
  | { kind: "objectList"; item: z.ZodObject }
  | { kind: "other" };

export type TypedField =
  | { kind: "color"; value: string }
  | { kind: "boxes"; value: Bounds[] }
  | { kind: "envelope"; value: Envelope }
  | { kind: "boolean"; value: boolean }
  | { kind: "number"; value: number; min?: number; max?: number }
  | { kind: "string"; value: string }
  | { kind: "enum"; value: string; options: string[] }
  | { kind: "literals"; value: string | number; options: Array<string | number> }
  | { kind: "enumList"; value: string[]; options: string[] }
  | { kind: "objectList"; value: ParamValues[]; item: z.ZodObject };

const enumOptions = (schema: z.ZodEnum): string[] => schema.options.map(String);

const isStringList = (v: unknown): v is string[] => Array.isArray(v) && v.every(isString);
const isRecordList = (v: unknown): v is ParamValues[] => Array.isArray(v) && v.every(isRecord);

function arrayKind(element: z.core.$ZodType): SchemaKind {
  if (!(element instanceof z.ZodType)) return { kind: "other" };
  const inner = unwrap(element);
  if (inner instanceof z.ZodEnum) return { kind: "enumList", options: enumOptions(inner) };
  if (inner instanceof z.ZodObject) return { kind: "objectList", item: inner };
  return { kind: "other" };
}

function unionKind(options: readonly z.core.$ZodType[]): SchemaKind {
  const values = options
    .flatMap((o) => (o instanceof z.ZodLiteral ? [...o.values] : []))
    .filter(isStringOrNumber);
  return values.length ? { kind: "literals", options: values } : { kind: "other" };
}

function schemaKind(schema: z.ZodType): SchemaKind {
  const inner = unwrap(schema);
  if (inner.def.type === "string") return { kind: "string" };
  if (inner instanceof z.ZodBoolean) return { kind: "boolean" };
  if (inner instanceof z.ZodNumber)
    return { kind: "number", min: inner.minValue ?? undefined, max: inner.maxValue ?? undefined };
  if (inner instanceof z.ZodEnum) return { kind: "enum", options: enumOptions(inner) };
  if (inner instanceof z.ZodUnion) return unionKind(inner.options);
  if (inner instanceof z.ZodArray) return arrayKind(inner.element);
  return { kind: "other" };
}

const typed = <K extends { kind: string }, V>(
  kind: K,
  value: unknown,
  is: (v: unknown) => v is V,
): (K & { value: V }) | null => (is(value) ? { ...kind, value } : null);

export function typedField(schema: z.ZodType, value: unknown): TypedField | null {
  const { widget } = fieldMeta(schema);
  if (isColor(schema)) return typed({ kind: "color" }, value ?? "", isString);
  if (widget === "boxes") return typed({ kind: "boxes" }, value ?? [], isBoundsList);
  if (widget === "envelope") return typed({ kind: "envelope" }, value ?? {}, isEnvelope);
  const kind = schemaKind(schema);
  switch (kind.kind) {
    case "boolean":
      return typed(kind, value ?? false, isBoolean);
    case "number":
      return typed(kind, value ?? 0, isNumber);
    case "string":
      return typed(kind, value ?? "", isString);
    case "enum":
      return typed(kind, value ?? kind.options[0] ?? "", isString);
    case "literals":
      return typed(kind, value ?? "", isStringOrNumber);
    case "enumList":
      return typed(kind, value ?? [], isStringList);
    case "objectList":
      return typed(kind, value ?? [], isRecordList);
    case "other":
      return null;
  }
}
