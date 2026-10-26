import type { JsonSchema } from "@/core/schema";
import { parseNumberText } from "@/core/text-input";

export type LengthLimits = { min: number; max: number };

export const lengthLimits = (schema: JsonSchema): LengthLimits => ({
  min: schema.minItems ?? 0,
  max: schema.maxItems ?? Infinity,
});

const THOUSANDS = /^\D*\d{1,3}(,\d{3})+(\.\d+)?\D*$/;

const withoutThousands = (part: string) =>
  THOUSANDS.test(part.trim()) ? part.replaceAll(",", "") : part;

export function parseListNumbers(text: string): number[] {
  const cells = text.split(/[\r\n\t]+/).filter((part) => part.trim());
  const parts =
    cells.length > 1
      ? cells.map(withoutThousands)
      : text.split(/[;,\s]+/).filter((part) => part.trim());
  return parts.flatMap((part) => parseNumberText(part, { lenient: true }) ?? []);
}
