import { existsSync } from "node:fs";
import { mkdir } from "node:fs/promises";
import path from "node:path";
import { nodeResultSchema, type LastRun, type NodeResult } from "../../shared/schema/values";
import { versionedHash } from "../../engine/hash";
import { readJson, writeJson } from "../../engine/json";
import { projectPaths } from "../../engine/project-paths";
import { byCodeUnit } from "../../shared/lists";

export const ENGINE_VERSION = "demoflow-engine/1";

export function nodeHash(type: string, params: unknown, upstream: Array<[string, string]>): string {
  const ports = [...upstream].sort(([a], [b]) => byCodeUnit(a, b));
  return versionedHash(ENGINE_VERSION, { type, params, ports });
}

export async function readCached(
  cacheDir: string,
  hash: string,
  projectDir: string,
): Promise<NodeResult | null> {
  const result = await readJson(path.join(cacheDir, hash, "result.json"), nodeResultSchema);
  if (!result) return null;
  const missing = result.artifacts.some((a) => !existsSync(path.join(projectDir, a.path)));
  return missing ? null : result;
}

export async function writeCached(cacheDir: string, result: NodeResult): Promise<void> {
  const dir = path.join(cacheDir, result.hash);
  await mkdir(dir, { recursive: true });
  await writeJson(path.join(dir, "result.json"), result);
}

export async function writeLastRun(projectDir: string, lastRun: LastRun): Promise<void> {
  await mkdir(projectPaths(projectDir).meta, { recursive: true });
  await writeJson(projectPaths(projectDir).lastRun, lastRun);
}
