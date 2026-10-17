import { readFile, rename, writeFile } from "node:fs/promises";
import type { z } from "zod";

export async function readJson<T>(file: string): Promise<T | null> {
  try {
    return JSON.parse(await readFile(file, "utf8")) as T;
  } catch {
    return null;
  }
}

const isMissing = (error: unknown) =>
  error instanceof Error && "code" in error && error.code === "ENOENT";

function parseOrUndefined(text: string): unknown {
  try {
    return JSON.parse(text);
  } catch {
    return undefined;
  }
}

export async function readJsonKeepingBroken<T>(
  file: string,
  schema: z.ZodType<T>,
): Promise<T | null> {
  let text: string;
  try {
    text = await readFile(file, "utf8");
  } catch (error) {
    if (isMissing(error)) return null;
    throw error;
  }
  const parsed = schema.safeParse(parseOrUndefined(text));
  if (parsed.success) return parsed.data;
  const broken = `${file}.broken`;
  await rename(file, broken).catch(() => undefined);
  process.stderr.write(`[json] ${file} couldn't be read and was moved to ${broken}\n`);
  return null;
}

export async function atomicWrite(file: string, text: string): Promise<void> {
  const tmp = `${file}.${process.pid}.${Math.random().toString(36).slice(2)}.tmp`;
  await writeFile(tmp, text);
  await rename(tmp, file);
}
