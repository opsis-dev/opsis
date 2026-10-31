import { createHash } from "node:crypto";
import { createReadStream, existsSync, mkdirSync, statSync, writeFileSync } from "node:fs";
import { rm } from "node:fs/promises";
import path from "node:path";
import { pipeline } from "node:stream/promises";
import { z } from "zod";
import { downloadTo } from "./download";

export const pinnedSchema = z.object({
  url: z.string(),
  sha256: z.string().regex(/^[0-9a-f]{64}$/),
  size: z.number().int().positive(),
});

export type Pinned = z.infer<typeof pinnedSchema>;

export function cachedName(url: string): string {
  const key = createHash("sha256").update(url).digest("hex").slice(0, 16);
  const base = url.split("?")[0]?.split("/").at(-1) || "download";
  return `${key}-${base}`;
}

async function fileSha256(file: string): Promise<string> {
  const hash = createHash("sha256");
  await pipeline(createReadStream(file), hash);
  return hash.digest("hex");
}

export async function pinned(url: string, cache: string): Promise<Pinned> {
  const file = path.join(cache, cachedName(url));
  if (!existsSync(file)) {
    mkdirSync(cache, { recursive: true });
    console.error(`downloading ${url}`);
    await downloadTo(url, file);
  }
  return { url, sha256: await fileSha256(file), size: statSync(file).size };
}

export async function verifiedDownload(expected: Pinned, cache: string): Promise<string> {
  const got = await pinned(expected.url, cache);
  const file = path.join(cache, cachedName(expected.url));
  if (got.sha256 !== expected.sha256 || got.size !== expected.size) {
    await rm(file, { force: true });
    throw new Error(`The download from ${expected.url} isn't the pinned file.`);
  }
  return file;
}

export function generatedJson(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}

export function writeGeneratedJson(file: string, value: unknown): void {
  writeFileSync(file, generatedJson(value));
  console.log(`wrote ${file}`);
}
