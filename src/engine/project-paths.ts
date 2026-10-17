import path from "node:path";
import type { LastRun } from "../shared/schema/values";
import { readJson } from "./json";
import { PROJECT_DIRS } from "../shared/project-dirs";

export const projectPaths = (projectDir: string) => ({
  meta: path.join(projectDir, PROJECT_DIRS.meta),
  cache: path.join(projectDir, PROJECT_DIRS.cache),
  stills: path.join(projectDir, PROJECT_DIRS.stills),
  traces: path.join(projectDir, PROJECT_DIRS.traces),
  out: path.join(projectDir, PROJECT_DIRS.out),
  lastRun: path.join(projectDir, PROJECT_DIRS.lastRun),
});

export function isInside(root: string, file: string): boolean {
  const rel = path.relative(root, file);
  return rel !== ".." && !rel.startsWith(`..${path.sep}`) && !path.isAbsolute(rel);
}

export const toPosixRel = (root: string, file: string): string =>
  path.relative(root, file).split(path.sep).join("/");

export const toProjectRef = (root: string, file: string): string =>
  `project:${toPosixRel(root, file)}`;

export const readLastRun = (projectDir: string): Promise<LastRun | null> =>
  readJson<LastRun>(projectPaths(projectDir).lastRun);
