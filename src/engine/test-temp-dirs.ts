import { mkdtemp, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach } from "vitest";

export function tempDirs(prefix: string): () => Promise<string> {
  const made: string[] = [];
  afterEach(async () => {
    await Promise.all(made.splice(0).map((d) => rm(d, { recursive: true, force: true })));
  });
  return async () => {
    const dir = await mkdtemp(path.join(os.tmpdir(), prefix));
    made.push(dir);
    return dir;
  };
}
