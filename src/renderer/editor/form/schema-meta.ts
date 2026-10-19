import type { z } from "zod";
import { fieldMeta, isColorSchema, REMOTION_COLOR } from "@/shared/elements/introspect";
import { humanize } from "@/shared/format";

export const isColor = isColorSchema;

export function labelFor(key: string, schema: z.ZodType): string {
  const d = fieldMeta(schema).description;
  return d && d !== REMOTION_COLOR ? d : humanize(key);
}
