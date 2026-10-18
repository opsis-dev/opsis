import { readFileSync } from "node:fs";
import type { z } from "zod";

export const readJsonWith = <T>(schema: z.ZodType<T>, file: string): T =>
  schema.parse(JSON.parse(readFileSync(file, "utf8")));
