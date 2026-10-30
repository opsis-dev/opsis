import { readFileSync } from "node:fs";
import type { z } from "zod";

export function readJsonWith<T>(schema: z.ZodType<T>, file: string): T {
  return schema.parse(JSON.parse(readFileSync(file, "utf8")));
}
