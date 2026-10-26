import { isBoundsList, type Bounds } from "@opsis/video/common/geometry";
import {
  isBoolean,
  isNumber,
  isRecord,
  isString,
  isStringOrNumber,
} from "@opsis/video/common/guards";
import type { MinMax } from "@opsis/video/common/math";
import { isThemeValue, type ThemeValue } from "@opsis/video/common/theme";
import {
  baseType,
  fieldMeta,
  isColorSchema,
  shapeEntries,
  takesNull,
  type JsonSchema,
} from "@/editor/form/schema";
import type { CameraPath } from "@/generated/bindings";
import {
  isCameraPath,
  isEnvelope,
  isEnvelopeKey,
  type Envelope,
  type EnvelopeKey,
} from "@/timeline/envelopes";
import { lengthLimits, type LengthLimits } from "./list-values";
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
  | ({ kind: "objectList"; item: JsonSchema } & LengthLimits);

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

const isStringList = (v: unknown): v is string[] => Array.isArray(v) && v.every(isString);
const isNumberList = (v: unknown): v is number[] => Array.isArray(v) && v.every(isNumber);
const isOptionalNumber = (v: unknown): v is number | undefined => v === undefined || isNumber(v);
const isRecordList = (v: unknown): v is ParamValues[] => Array.isArray(v) && v.every(isRecord);

const stringOptions = (schema: JsonSchema): string[] => (schema.enum ?? []).filter(isString);

function arrayKind(list: JsonSchema): SchemaKind | null {
  const element = list.items;
  if (!element) return null;
  const limits = lengthLimits(list);
  const type = baseType(element);
  if (isColorSchema(element)) return { kind: "colorList", ...limits };
  if (type === "string" && element.enum)
    return { kind: "enumList", options: stringOptions(element) };
  if (type === "object") return { kind: "objectList", item: element, ...limits };
  if (type === "string") return { kind: "stringList", ...limits };
  if (type === "number" || type === "integer") return { kind: "numberList", ...limits };
  return null;
}

function schemaKind(schema: JsonSchema): SchemaKind | null {
  const type = baseType(schema);
  if (type === "string")
    return schema.enum ? { kind: "enum", options: stringOptions(schema) } : { kind: "string" };
  if (type === "boolean") return { kind: "boolean" };
  if ((type === "number" || type === "integer") && schema.enum)
    return { kind: "literals", options: schema.enum.filter(isStringOrNumber) };
  if (type === "number" || type === "integer")
    return {
      kind: takesNull(schema) ? "optionalNumber" : "number",
      min: schema.minimum,
      max: schema.maximum,
    };
  if (type === "array") return arrayKind(schema);
  return null;
}

const envelopeTracks = (schema: JsonSchema): EnvelopeKey[] =>
  shapeEntries(schema)
    .map(([key]) => key)
    .filter(isEnvelopeKey);

const typed = <K extends { kind: string }, V>(
  kind: K,
  value: unknown,
  is: (v: unknown) => v is V,
): (K & { value: V }) | null => (is(value) ? { ...kind, value } : null);

export function typedField(schema: JsonSchema, raw: unknown): TypedField | null {
  const value = raw === null ? undefined : raw;
  const { widget } = fieldMeta(schema);
  if (isColorSchema(schema)) return typed({ kind: "color" }, value ?? "", isString);
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
