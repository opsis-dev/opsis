import { statSync } from "node:fs";
import path from "node:path";
import { isInside } from "../../engine/project-paths";
import { secretFile } from "../../engine/secret-paths";
import { projectRelPath } from "../../shared/asset-ref";
import { stringsIn } from "../../shared/vars";

const FILE_NAME = /\.[a-z0-9]{2,5}$/i;

export function fileStamps(projectDir: string, params: unknown): Record<string, string> {
  const stamps: Record<string, string> = {};
  for (const value of stringsIn(params, (text) => FILE_NAME.test(text.trim()))) {
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
