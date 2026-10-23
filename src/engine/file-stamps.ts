import { statSync } from "node:fs";
import path from "node:path";
import { projectRelPath } from "../shared/asset-ref";
import { namedFiles } from "../shared/named-files";
import { isInside } from "./project-paths";
import { secretFile } from "./secret-paths";

export function fileStamp(file: string): string | null {
  const stat = statSync(file, { throwIfNoEntry: false });
  return stat?.isFile() ? `${stat.size}:${stat.mtimeMs}` : null;
}

export const fileStamps = (projectDir: string, params: unknown): Record<string, string> =>
  refStamps(projectDir, namedFiles(params));

export function refStamps(projectDir: string, refs: readonly string[]): Record<string, string> {
  const stamps: Record<string, string> = {};
  for (const value of refs) {
    const rel = projectRelPath(value.trim());
    const file = rel === null ? null : path.join(projectDir, rel);
    const stamp = file && isInside(projectDir, file) ? fileStamp(file) : null;
    if (stamp) stamps[value] = stamp;
  }
  return stamps;
}

export function signInStamp(projectDir: string, name: string): string | null {
  const stat = statSync(secretFile(projectDir, name), { throwIfNoEntry: false });
  return stat ? `${stat.size}:${stat.mtimeMs}` : null;
}
