import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import { mkdir, readFile, rename, writeFile } from "node:fs/promises";
import path from "node:path";
import type { LastRun, NodeResult } from "../../shared/schema/values";
import { stableStringify } from "../../shared/vars";

export const ENGINE_VERSION = "demoflow-engine/1";

export const projectPaths = (projectDir: string) => ({
  meta: path.join(projectDir, ".demoflow"),
  cache: path.join(projectDir, ".demoflow", "cache"),
  out: path.join(projectDir, ".demoflow", "out"),
  lastRun: path.join(projectDir, ".demoflow", "last-run.json"),
});

export function nodeHash(type: string, params: unknown, upstream: Array<[string, string]>): string {
  const h = createHash("sha256");
  h.update(ENGINE_VERSION);
  h.update("\0");
  h.update(type);
  h.update("\0");
  h.update(stableStringify(params));
  h.update("\0");
  for (const [portId, hash] of [...upstream].sort(([a], [b]) => (a < b ? -1 : 1)))
    h.update(`${portId}=${hash};`);
  return h.digest("hex").slice(0, 24);
}

export async function readCached(
  cacheDir: string,
  hash: string,
  projectDir: string,
): Promise<NodeResult | null> {
  const file = path.join(cacheDir, hash, "result.json");
  if (!existsSync(file)) return null;
  try {
    const result = JSON.parse(await readFile(file, "utf8")) as NodeResult;
    const missing = result.artifacts.some((a) => !existsSync(path.join(projectDir, a.path)));
    return missing ? null : result;
  } catch {
    return null;
  }
}

export async function writeCached(cacheDir: string, result: NodeResult): Promise<void> {
  const dir = path.join(cacheDir, result.hash);
  await mkdir(dir, { recursive: true });
  await atomicWrite(path.join(dir, "result.json"), JSON.stringify(result, null, 2));
}

export async function readLastRun(projectDir: string): Promise<LastRun | null> {
  try {
    return JSON.parse(await readFile(projectPaths(projectDir).lastRun, "utf8")) as LastRun;
  } catch {
    return null;
  }
}

export async function writeLastRun(projectDir: string, lastRun: LastRun): Promise<void> {
  await mkdir(projectPaths(projectDir).meta, { recursive: true });
  await atomicWrite(projectPaths(projectDir).lastRun, JSON.stringify(lastRun, null, 2));
}

async function atomicWrite(file: string, text: string): Promise<void> {
  const tmp = `${file}.${process.pid}.tmp`;
  await writeFile(tmp, text);
  await rename(tmp, file);
}
