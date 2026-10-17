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
