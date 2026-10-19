import { mkdir } from "node:fs/promises";
import { lastRunSchema, type LastRun } from "../shared/schema/values";
import { readJson, writeJson } from "./json";
import { projectPaths } from "./project-paths";

export const readLastRun = (projectDir: string): Promise<LastRun | null> =>
  readJson(projectPaths(projectDir).lastRun, lastRunSchema);

export async function writeLastRun(projectDir: string, lastRun: LastRun): Promise<void> {
  const paths = projectPaths(projectDir);
  await mkdir(paths.meta, { recursive: true });
  await writeJson(paths.lastRun, lastRun);
}
