import path from "node:path";
import { lastRunSchema, type LastRun } from "../shared/schema/values";
import { readJson } from "./json";
import { PROJECT_DIRS } from "../shared/project-dirs";
import { projectRef } from "../shared/asset-ref";
import { mapValues } from "../shared/pick";

export const projectPaths = (projectDir: string) =>
  mapValues(PROJECT_DIRS, (rel) => path.join(projectDir, rel));

export function isInside(root: string, file: string): boolean {
  const rel = path.relative(root, file);
  return rel !== ".." && !rel.startsWith(`..${path.sep}`) && !path.isAbsolute(rel);
}

export const toPosixRel = (root: string, file: string): string =>
  path.relative(root, file).split(path.sep).join("/");

export const toProjectRef = (root: string, file: string): string =>
  projectRef(toPosixRel(root, file));

export const readLastRun = (projectDir: string): Promise<LastRun | null> =>
  readJson(projectPaths(projectDir).lastRun, lastRunSchema);
