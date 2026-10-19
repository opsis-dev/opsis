import path from "node:path";
import { projectRef } from "../shared/asset-ref";
import { mapValues } from "../shared/objects";
import { PROJECT_DIRS } from "../shared/project-dirs";

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
