import { humanize } from "@opsis/video/common/format";
import { fieldMeta, type JsonSchema } from "@/core/schema";

export function labelFor(key: string, schema: JsonSchema): string {
  return fieldMeta(schema).description || humanize(key);
}
