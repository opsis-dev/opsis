import { z } from "zod";
import { fieldMeta, unwrap } from "@/shared/elements/introspect";
import { isBoolean, isNumber, isRecord, isString, isStringOrNumber } from "@/shared/guards";
import type { MinMax } from "@/shared/math";
import { isBoundsList, type Bounds } from "@/shared/schema/actions";
import { isEnvelopeKey, type Envelope, type EnvelopeKey } from "@/shared/schema/timeline";
import { isEnvelope } from "./envelope-field";
import { isColor } from "./schema-meta";
import type { ParamValues } from "./types";

type SchemaKind =
  | { kind: "boolean" }
  | ({ kind: "number" } & Partial<MinMax>)
  | { kind: "string" }
  | { kind: "enum"; options: string[] }
  | { kind: "literals"; options: Array<string | number> }
  | { kind: "enumList"; options: string[] }
  | { kind: "objectList"; item: z.ZodObject };

type FieldKind =
  | SchemaKind
  | { kind: "color" }
  | { kind: "boxes" }
  | { kind: "envelope"; tracks: EnvelopeKey[] };

type FieldValue = {
  color: string;
  boxes: Bounds[];
  envelope: Envelope;
  boolean: boolean;
  number: number;
  string: string;
  enum: string;
  literals: string | number;
  enumList: string[];
  objectList: ParamValues[];
};

export type TypedField = {
  [K in FieldKind["kind"]]: Extract<FieldKind, { kind: K }> & { value: FieldValue[K] };
}[FieldKind["kind"]];

const enumOptions = (schema: z.ZodEnum): string[] => schema.options.map(String);

const isStringList = (v: unknown): v is string[] => Array.isArray(v) && v.every(isString);
const isRecordList = (v: unknown): v is ParamValues[] => Array.isArray(v) && v.every(isRecord);

function arrayKind(element: z.core.$ZodType): SchemaKind | null {
  if (!(element instanceof z.ZodType)) return null;
  const inner = unwrap(element);
  if (inner instanceof z.ZodEnum) return { kind: "enumList", options: enumOptions(inner) };
  if (inner instanceof z.ZodObject) return { kind: "objectList", item: inner };
  return null;
}

function unionKind(options: readonly z.core.$ZodType[]): SchemaKind | null {
  const values = options
    .flatMap((o) => (o instanceof z.ZodLiteral ? [...o.values] : []))
    .filter(isStringOrNumber);
  return values.length ? { kind: "literals", options: values } : null;
}

function schemaKind(schema: z.ZodType): SchemaKind | null {
  const inner = unwrap(schema);
  if (inner.def.type === "string") return { kind: "string" };
  if (inner instanceof z.ZodBoolean) return { kind: "boolean" };
  if (inner instanceof z.ZodNumber)
    return { kind: "number", min: inner.minValue ?? undefined, max: inner.maxValue ?? undefined };
  if (inner instanceof z.ZodEnum) return { kind: "enum", options: enumOptions(inner) };
  if (inner instanceof z.ZodUnion) return unionKind(inner.options);
  if (inner instanceof z.ZodArray) return arrayKind(inner.element);
  return null;
}

function envelopeTracks(schema: z.ZodType): EnvelopeKey[] {
  const inner = unwrap(schema);
  return inner instanceof z.ZodObject ? Object.keys(inner.shape).filter(isEnvelopeKey) : [];
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
  if (widget === "envelope")
    return typed({ kind: "envelope", tracks: envelopeTracks(schema) }, value ?? {}, isEnvelope);
  const kind = schemaKind(schema);
  switch (kind?.kind) {
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
    case undefined:
      return null;
  }
}
