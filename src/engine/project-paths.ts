import path from "node:path";
import type { LastRun } from "../shared/schema/values";
import { readJson } from "./json";

export const projectPaths = (projectDir: string) => ({
  meta: path.join(projectDir, ".demoflow"),
  cache: path.join(projectDir, ".demoflow", "cache"),
  out: path.join(projectDir, ".demoflow", "out"),
  lastRun: path.join(projectDir, ".demoflow", "last-run.json"),
});

export const toPosixRel = (root: string, file: string): string =>
  path.relative(root, file).split(path.sep).join("/");

export const toProjectRef = (root: string, file: string): string =>
  `project:${toPosixRel(root, file)}`;

export const readLastRun = (projectDir: string): Promise<LastRun | null> =>
  readJson<LastRun>(projectPaths(projectDir).lastRun);
