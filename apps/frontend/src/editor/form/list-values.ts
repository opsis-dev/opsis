import type { JsonSchema } from "@/editor/form/schema";
import { parseNumberText } from "@/lib/number-text";

export type LengthLimits = { min: number; max: number };

export function lengthLimits(schema: JsonSchema): LengthLimits {
  return {
    min: schema.minItems ?? 0,
    max: schema.maxItems ?? Infinity,
  };
}

const THOUSANDS = /^\D*\d{1,3}(,\d{3})+(\.\d+)?\D*$/;

function withoutThousands(part: string) {
  return THOUSANDS.test(part.trim()) ? part.replaceAll(",", "") : part;
}

export function parseListNumbers(text: string): number[] {
  const cells = text.split(/[\r\n\t]+/).filter((part) => part.trim());
  const parts =
    cells.length > 1
      ? cells.map(withoutThousands)
      : text.split(/[;,\s]+/).filter((part) => part.trim());
  return parts.flatMap((part) => parseNumberText(part, { lenient: true }) ?? []);
}
