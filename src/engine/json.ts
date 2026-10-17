import { readFile, rename, writeFile } from "node:fs/promises";
import type { z } from "zod";
import { tryParseJson } from "../shared/guards";

export async function readJson<T>(file: string): Promise<T | null> {
  try {
    return JSON.parse(await readFile(file, "utf8")) as T;
  } catch {
    return null;
  }
}

export const isMissingFile = (error: unknown): boolean =>
  error instanceof Error && "code" in error && error.code === "ENOENT";

export async function readJsonKeepingBroken<T>(
  file: string,
  schema: z.ZodType<T>,
): Promise<T | null> {
  let text: string;
  try {
    text = await readFile(file, "utf8");
  } catch (error) {
    if (isMissingFile(error)) return null;
    throw error;
  }
  const json = tryParseJson(text);
  const parsed = schema.safeParse(json.ok ? json.value : undefined);
  if (parsed.success) return parsed.data;
  const broken = `${file}.broken`;
  await rename(file, broken).catch(() => undefined);
  process.stderr.write(`[json] ${file} couldn't be read and was moved to ${broken}\n`);
  return null;
}

export async function atomicWrite(file: string, data: string | Uint8Array): Promise<void> {
  const tmp = `${file}.${process.pid}.${Math.random().toString(36).slice(2)}.tmp`;
  await writeFile(tmp, data);
  await rename(tmp, file);
}

export const writeJson = (file: string, value: unknown): Promise<void> =>
  atomicWrite(file, `${JSON.stringify(value, null, 2)}\n`);
