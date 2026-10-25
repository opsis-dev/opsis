import { fieldMeta, isColorSchema, REMOTION_COLOR } from "@opsis/core/elements/introspect";
import { humanize } from "@opsis/core/format";
import type { z } from "zod";

export const isColor = isColorSchema;

export function labelFor(key: string, schema: z.ZodType): string {
  const d = fieldMeta(schema).description;
  return d && d !== REMOTION_COLOR ? d : humanize(key);
}
