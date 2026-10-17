import path from "node:path";
import type { LastRun } from "../shared/schema/values";
import { readJson } from "./json";

export const projectPaths = (projectDir: string) => ({
  meta: path.join(projectDir, ".demoflow"),
  cache: path.join(projectDir, ".demoflow", "cache"),
  out: path.join(projectDir, ".demoflow", "out"),
  lastRun: path.join(projectDir, ".demoflow", "last-run.json"),
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
