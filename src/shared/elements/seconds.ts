import { isNumber, isRecord, isString } from "../guards";
import { roundTo } from "../math";
import { type FieldMeta, mapLeaves } from "./introspect";
import type { ElementMeta } from "./types";

const toFrames = (seconds: number, rate: number): number => roundTo(seconds * rate, 2);

export function framesFromSeconds(meta: FieldMeta, value: unknown): unknown {
  const rate = meta.framesAt;
  if (!rate) return value;
  if (isNumber(value)) return toFrames(value, rate);
  if (!isString(value)) return value;
  return value
    .split(",")
    .map((part) => {
      const seconds = Number(part.trim());
      return part.trim() && Number.isFinite(seconds)
        ? String(toFrames(seconds, rate))
        : part.trim();
    })
    .join(", ");
}

export function settingsInFrames(
  meta: ElementMeta,
  settings: Record<string, unknown>,
): Record<string, unknown> {
  const mapped = mapLeaves(meta.schema, settings, framesFromSeconds);
  return isRecord(mapped) ? mapped : settings;
}
