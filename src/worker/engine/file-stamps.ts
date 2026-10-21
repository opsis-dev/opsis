import { statSync } from "node:fs";
import path from "node:path";
import { isInside } from "../../engine/project-paths";
import { secretFile } from "../../engine/secret-paths";
import { projectRelPath } from "../../shared/asset-ref";
import { namedFiles } from "../../shared/named-files";

export function fileStamps(projectDir: string, params: unknown): Record<string, string> {
  const stamps: Record<string, string> = {};
  for (const value of namedFiles(params)) {
    const rel = projectRelPath(value.trim());
    const file = rel === null ? null : path.join(projectDir, rel);
    const stat =
      file && isInside(projectDir, file) ? statSync(file, { throwIfNoEntry: false }) : undefined;
    if (stat?.isFile()) stamps[value] = `${stat.size}:${stat.mtimeMs}`;
  }
  return stamps;
}

export function signInStamp(projectDir: string, name: string): string | null {
  const stat = statSync(secretFile(projectDir, name), { throwIfNoEntry: false });
  return stat ? `${stat.size}:${stat.mtimeMs}` : null;
}
