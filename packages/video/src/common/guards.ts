import type { z } from "zod";

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function hasStringType(value: unknown): value is Record<string, unknown> & { type: string } {
  return isRecord(value) && typeof value.type === "string";
}

export function isKeyOf<K extends string>(
  table: Readonly<Record<K, unknown>>,
  key: string,
): key is K {
  return Object.hasOwn(table, key);
}

export function hasTypeIn<T extends { type: string }>(types: Record<T["type"], true>) {
  return (value: unknown): value is T => hasStringType(value) && isKeyOf(types, value.type);
}

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

export function isString(v: unknown): v is string {
  return typeof v === "string";
}

export function nonBlankString(v: unknown): string | undefined {
  return isString(v) && v.trim() ? v.trim() : undefined;
}
export function isNumber(v: unknown): v is number {
  return typeof v === "number";
}
export function isBoolean(v: unknown): v is boolean {
  return typeof v === "boolean";
}
export function isStringOrNumber(v: unknown): v is string | number {
  return isString(v) || isNumber(v);
}

export function isOneOf<T extends string>(values: readonly T[], value: unknown): value is T {
  return values.some((v) => v === value);
}
