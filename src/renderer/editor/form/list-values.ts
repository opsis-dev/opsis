import { z } from "zod";
import { parseNumberText } from "@/shared/text-input";

export type LengthLimits = { min: number; max: number };

export function lengthLimits(schema: z.ZodType): LengthLimits {
  const limits: LengthLimits = { min: 0, max: Infinity };
  if (!(schema instanceof z.ZodArray)) return limits;
  for (const check of schema.def.checks ?? []) {
    const def = check._zod.def;
    if (def.check === "min_length" && "minimum" in def && typeof def.minimum === "number")
      limits.min = def.minimum;
    if (def.check === "max_length" && "maximum" in def && typeof def.maximum === "number")
      limits.max = def.maximum;
  }
  return limits;
}

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
