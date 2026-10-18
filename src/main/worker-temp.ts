import { mkdirSync, readdirSync, rmSync } from "node:fs";
import path from "node:path";
import { errorCode } from "../shared/errors";

export type WorkerKind = "run" | "stills" | "analysis" | "setup";

const OWNER = /^(\d+)-/;

export function workerTempName(kind: WorkerKind, id: string, pid = process.pid): string {
  return `${pid}-${kind}-${id.replace(/[^\w-]/g, "").slice(0, 8)}`;
}

export function makeWorkerTemp(root: string, name: string): string {
  const dir = path.join(root, name);
  mkdirSync(dir, { recursive: true });
  return dir;
}

export function removeWorkerTemp(dir: string): void {
  rmSync(dir, { recursive: true, force: true, maxRetries: 3 });
}

const alive = (pid: number): boolean => {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return errorCode(error) === "EPERM";
  }
};

export function clearStaleWorkerTemps(
  root: string,
  isAlive: (pid: number) => boolean = alive,
): string[] {
  let names: string[];
  try {
    names = readdirSync(root);
  } catch {
    return [];
  }
  const stale = names.filter((name) => {
    const owner = Number(OWNER.exec(name)?.[1]);
    return !owner || (owner !== process.pid && !isAlive(owner));
  });
  for (const name of stale) removeWorkerTemp(path.join(root, name));
  return stale;
}

export const tempEnv = (dir: string): Record<string, string> => ({
  TMPDIR: dir,
  TEMP: dir,
  TMP: dir,
});
