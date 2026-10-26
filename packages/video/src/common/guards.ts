import type { z } from "zod";

export const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const hasStringType = (value: unknown): value is Record<string, unknown> & { type: string } =>
  isRecord(value) && typeof value.type === "string";

export const isKeyOf = <K extends string>(
  table: Readonly<Record<K, unknown>>,
  key: string,
): key is K => Object.hasOwn(table, key);

export const hasTypeIn =
  <T extends { type: string }>(types: Record<T["type"], true>) =>
  (value: unknown): value is T =>
    hasStringType(value) && isKeyOf(types, value.type);

type ParsedJson = { ok: true; value: unknown } | { ok: false };

export function tryParseJson(text: string): ParsedJson {
  try {
    return { ok: true, value: JSON.parse(text) };
  } catch {
    return { ok: false };
  }
}

export function parseJsonWith<T>(
  schema: z.ZodType<T>,
  text: string,
): z.ZodSafeParseResult<T> | null {
  const json = tryParseJson(text);
  return json.ok ? schema.safeParse(json.value) : null;
}

export const isString = (v: unknown): v is string => typeof v === "string";

export const nonBlankString = (v: unknown): string | undefined =>
  isString(v) && v.trim() ? v.trim() : undefined;
export const isNumber = (v: unknown): v is number => typeof v === "number";
export const isBoolean = (v: unknown): v is boolean => typeof v === "boolean";
export const isStringOrNumber = (v: unknown): v is string | number => isString(v) || isNumber(v);

export const isOneOf = <T extends string>(values: readonly T[], value: unknown): value is T =>
  values.some((v) => v === value);
