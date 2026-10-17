import type { z } from "zod";

export const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

export type ParsedJson = { ok: true; value: unknown } | { ok: false };

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
export const isNumber = (v: unknown): v is number => typeof v === "number";
export const isBoolean = (v: unknown): v is boolean => typeof v === "boolean";
export const isStringOrNumber = (v: unknown): v is string | number => isString(v) || isNumber(v);
