import { randomUUID } from "node:crypto";
import { readFile, rename, writeFile } from "node:fs/promises";
import type { z } from "zod";
import { parseJsonWith } from "../shared/guards";

const MISSING_CODES = new Set(["ENOENT", "ENOTDIR"]);

export const isMissingFile = (error: unknown): boolean =>
  error instanceof Error && "code" in error && MISSING_CODES.has(String(error.code));

export async function readTextIfPresent(file: string): Promise<string | null> {
  try {
    return await readFile(file, "utf8");
  } catch (error) {
    if (isMissingFile(error)) return null;
    throw error;
  }
}

type Read<T> = { kind: "missing" } | { kind: "invalid" } | { kind: "valid"; value: T };

async function readValidJson<T>(file: string, schema: z.ZodType<T>): Promise<Read<T>> {
  const text = await readTextIfPresent(file);
  if (text === null) return { kind: "missing" };
  const parsed = parseJsonWith(schema, text);
  return parsed?.success ? { kind: "valid", value: parsed.data } : { kind: "invalid" };
}

export async function readJson<T>(file: string, schema: z.ZodType<T>): Promise<T | null> {
  const read = await readValidJson(file, schema);
  return read.kind === "valid" ? read.value : null;
}

export async function readJsonKeepingBroken<T>(
  file: string,
  schema: z.ZodType<T>,
): Promise<T | null> {
  const read = await readValidJson(file, schema);
  if (read.kind === "valid") return read.value;
  if (read.kind === "missing") return null;
  const broken = `${file}.broken`;
  try {
    await rename(file, broken);
    process.stderr.write(`[json] ${file} couldn't be read and was moved to ${broken}\n`);
  } catch {
    process.stderr.write(`[json] ${file} couldn't be read\n`);
  }
  return null;
}

export async function atomicWrite(file: string, data: string | Uint8Array): Promise<void> {
  const tmp = `${file}.${randomUUID()}.tmp`;
  await writeFile(tmp, data);
  await rename(tmp, file);
}

export const writeJson = (file: string, value: unknown): Promise<void> =>
  atomicWrite(file, `${JSON.stringify(value, null, 2)}\n`);
