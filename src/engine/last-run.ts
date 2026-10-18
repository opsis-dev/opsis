import { mkdir } from "node:fs/promises";
import { lastRunSchema, type LastRun } from "../shared/schema/values";
import { readJson, writeJson } from "./json";
import { projectPaths } from "./project-paths";

export const readLastRun = (projectDir: string): Promise<LastRun | null> =>
  readJson(projectPaths(projectDir).lastRun, lastRunSchema);

export async function writeLastRun(projectDir: string, lastRun: LastRun): Promise<void> {
  await mkdir(projectPaths(projectDir).meta, { recursive: true });
  await writeJson(projectPaths(projectDir).lastRun, lastRun);
}
