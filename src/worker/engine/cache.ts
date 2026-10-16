import { existsSync } from "node:fs";
import { mkdir } from "node:fs/promises";
import path from "node:path";
import type { LastRun, NodeResult } from "../../shared/schema/values";
import { stableJson } from "../../shared/hash";
import { sha256Hex } from "../../engine/hash";
import { atomicWrite, readJson } from "../../engine/json";
import { projectPaths } from "../../engine/project-paths";

export const ENGINE_VERSION = "demoflow-engine/1";

export function nodeHash(type: string, params: unknown, upstream: Array<[string, string]>): string {
  const ports = [...upstream]
    .sort(([a], [b]) => (a < b ? -1 : 1))
    .map(([portId, hash]) => `${portId}=${hash};`)
    .join("");
  return sha256Hex(`${ENGINE_VERSION}\0${type}\0${stableJson(params)}\0${ports}`, 24);
}

export async function readCached(
  cacheDir: string,
  hash: string,
  projectDir: string,
): Promise<NodeResult | null> {
  const file = path.join(cacheDir, hash, "result.json");
  if (!existsSync(file)) return null;
  const result = await readJson<NodeResult>(file);
  if (!result) return null;
  const missing = result.artifacts.some((a) => !existsSync(path.join(projectDir, a.path)));
  return missing ? null : result;
}

export async function writeCached(cacheDir: string, result: NodeResult): Promise<void> {
  const dir = path.join(cacheDir, result.hash);
  await mkdir(dir, { recursive: true });
  await atomicWrite(path.join(dir, "result.json"), JSON.stringify(result, null, 2));
}

export async function writeLastRun(projectDir: string, lastRun: LastRun): Promise<void> {
  await mkdir(projectPaths(projectDir).meta, { recursive: true });
  await atomicWrite(projectPaths(projectDir).lastRun, JSON.stringify(lastRun, null, 2));
}
