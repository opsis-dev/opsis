import type { z } from "zod";
import { fieldMeta, unwrap } from "@/shared/elements/introspect";
import { humanize } from "@/shared/format";

const REMOTION_COLOR = "__remotion-color";

export const isColor = (schema: z.ZodType): boolean =>
  unwrap(schema).description === REMOTION_COLOR || fieldMeta(schema).widget === "color";

export function labelFor(key: string, schema: z.ZodType): string {
  const d = fieldMeta(schema).description;
  return d && d !== REMOTION_COLOR ? d : humanize(key);
}
