import type { z } from "zod";

type ParsedJson = { ok: true; value: unknown } | { ok: false };

function tryParseJson(text: string): ParsedJson {
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

export function isNumber(v: unknown): v is number {
  return typeof v === "number";
}

export function isBoolean(v: unknown): v is boolean {
  return typeof v === "boolean";
}
