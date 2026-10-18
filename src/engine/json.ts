import { readFile, rename, writeFile } from "node:fs/promises";
import type { z } from "zod";
import { errorCode } from "../shared/errors";
import { parseJsonWith } from "../shared/guards";
import { writeViaTemp } from "./atomic";
import { logWarning } from "./log";

const MISSING_CODES = new Set<string | undefined>(["ENOENT", "ENOTDIR"]);

const isMissingFile = (error: unknown): boolean => MISSING_CODES.has(errorCode(error));

export async function nullIfMissing<T>(work: Promise<T>): Promise<T | null> {
  try {
    return await work;
  } catch (error) {
    if (isMissingFile(error)) return null;
    throw error;
  }
}

export const readTextIfPresent = (file: string): Promise<string | null> =>
  nullIfMissing(readFile(file, "utf8"));

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
    logWarning("json", `${file} couldn't be read and was moved to ${broken}`);
  } catch {
    logWarning("json", `${file} couldn't be read`);
  }
  return null;
}

export const atomicWrite = (file: string, data: string | Uint8Array): Promise<void> =>
  writeViaTemp(file, (temp) => writeFile(temp, data));

export const writeJson = (file: string, value: unknown): Promise<void> =>
  atomicWrite(file, `${JSON.stringify(value, null, 2)}\n`);
