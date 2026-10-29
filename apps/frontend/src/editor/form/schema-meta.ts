import { humanize } from "@opsis/video/common/format";
import { fieldMeta, type JsonSchema } from "@/editor/form/schema";

export function labelFor(key: string, schema: JsonSchema): string {
  return fieldMeta(schema).description || humanize(key);
}
