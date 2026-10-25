import { fieldMeta, unwrap } from "@opsis/core/elements/introspect";
import { isThemeValue, type ThemeValue } from "@opsis/core/elements/theme";
import { isBoolean, isNumber, isRecord, isString, isStringOrNumber } from "@opsis/core/guards";
import type { MinMax } from "@opsis/core/math";
import { isBoundsList, type Bounds } from "@opsis/core/schema/actions";
import {
  isCameraPath,
  isEnvelope,
  isEnvelopeKey,
  type CameraPath,
  type Envelope,
  type EnvelopeKey,
} from "@opsis/core/schema/timeline";
import { z } from "zod";
import { lengthLimits, type LengthLimits } from "./list-values";
import { isColor } from "./schema-meta";
import type { ParamValues } from "./types";

type SchemaKind =
  | { kind: "boolean" }
  | ({ kind: "number" } & Partial<MinMax>)
  | ({ kind: "optionalNumber" } & Partial<MinMax>)
  | { kind: "string" }
  | { kind: "enum"; options: string[] }
  | { kind: "literals"; options: Array<string | number> }
  | { kind: "enumList"; options: string[] }
  | ({ kind: "stringList" } & LengthLimits)
  | ({ kind: "numberList" } & LengthLimits)
  | ({ kind: "colorList" } & LengthLimits)
  | ({ kind: "objectList"; item: z.ZodObject } & LengthLimits);

type FieldKind =
  | SchemaKind
  | { kind: "color" }
  | { kind: "gradient" }
  | { kind: "theme" }
  | { kind: "boxes" }
  | { kind: "camera" }
  | { kind: "envelope"; tracks: EnvelopeKey[] };

type FieldValue = {
  color: string;
  gradient: string;
  theme: ThemeValue;
  stringList: string[];
  numberList: number[];
  colorList: string[];
  boxes: Bounds[];
  camera: CameraPath;
  envelope: Envelope;
  boolean: boolean;
  number: number;
  optionalNumber: number | undefined;
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
const isNumberList = (v: unknown): v is number[] => Array.isArray(v) && v.every(isNumber);
const isOptionalNumber = (v: unknown): v is number | undefined => v === undefined || isNumber(v);
const isRecordList = (v: unknown): v is ParamValues[] => Array.isArray(v) && v.every(isRecord);

function arrayKind(list: z.ZodArray): SchemaKind | null {
  const { element } = list;
  if (!(element instanceof z.ZodType)) return null;
  const inner = unwrap(element);
  const limits = lengthLimits(list);
  if (isColor(element)) return { kind: "colorList", ...limits };
  if (inner instanceof z.ZodEnum) return { kind: "enumList", options: enumOptions(inner) };
  if (inner instanceof z.ZodObject) return { kind: "objectList", item: inner, ...limits };
  if (inner.def.type === "string") return { kind: "stringList", ...limits };
  if (inner instanceof z.ZodNumber) return { kind: "numberList", ...limits };
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
    return {
      kind: unsetAllowed(schema) ? "optionalNumber" : "number",
      min: inner.minValue ?? undefined,
      max: inner.maxValue ?? undefined,
    };
  if (inner instanceof z.ZodEnum) return { kind: "enum", options: enumOptions(inner) };
  if (inner instanceof z.ZodUnion) return unionKind(inner.options);
  if (inner instanceof z.ZodArray) return arrayKind(inner);
  return null;
}

function unsetAllowed(schema: z.ZodType): boolean {
  const parsed = schema.safeParse(undefined);
  return parsed.success && parsed.data === undefined;
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
  if (widget === "gradient") return typed({ kind: "gradient" }, value ?? "", isString);
  if (widget === "theme") return typed({ kind: "theme" }, value ?? undefined, isThemeValue);
  if (widget === "boxes") return typed({ kind: "boxes" }, value ?? [], isBoundsList);
  if (widget === "camera") return typed({ kind: "camera" }, value ?? { keys: [] }, isCameraPath);
  if (widget === "envelope")
    return typed({ kind: "envelope", tracks: envelopeTracks(schema) }, value ?? {}, isEnvelope);
  const kind = schemaKind(schema);
  switch (kind?.kind) {
    case "boolean":
      return typed(kind, value ?? false, isBoolean);
    case "number":
      return typed(kind, value ?? 0, isNumber);
    case "optionalNumber":
      return typed(kind, value, isOptionalNumber);
    case "string":
      return typed(kind, value ?? "", isString);
    case "enum":
      return typed(kind, value ?? kind.options[0] ?? "", isString);
    case "literals":
      return typed(kind, value ?? "", isStringOrNumber);
    case "enumList":
    case "stringList":
    case "colorList":
      return typed(kind, value ?? [], isStringList);
    case "numberList":
      return typed(kind, value ?? [], isNumberList);
    case "objectList":
      return typed(kind, value ?? [], isRecordList);
    case undefined:
      return null;
  }
}
