import { existsSync } from "node:fs";
import { utimes, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { fileIdentity, pruneDir } from "./dir";
import { tempDirs } from "./test-temp-dirs";

const tempDir = tempDirs("opsis-dir-");

async function dirWith(ages: Record<string, number>): Promise<string> {
  const dir = await tempDir();
  const now = Date.now() / 1000;
  for (const [name, ageSeconds] of Object.entries(ages)) {
    const file = path.join(dir, name);
    await writeFile(file, name);
    await utimes(file, now - ageSeconds, now - ageSeconds);
  }
  return dir;
}

const left = (dir: string, names: string[]) =>
  names.filter((name) => existsSync(path.join(dir, name)));

describe("pruneDir", () => {
  it("keeps the newest files", async () => {
    const dir = await dirWith({ a: 30, b: 20, c: 10 });
    await pruneDir(dir, { newest: 2 });
    expect(left(dir, ["a", "b", "c"])).toEqual(["b", "c"]);
  });

  it("keeps what the caller names and anything used recently", async () => {
    const dir = await dirWith({ kept: 900, old: 900, fresh: 1 });
    await pruneDir(dir, { keep: (name) => name === "kept", unusedForMs: 600_000 });
    expect(left(dir, ["kept", "old", "fresh"])).toEqual(["kept", "fresh"]);
  });

  it("does nothing for a missing folder", async () => {
    await expect(pruneDir(path.join(os.tmpdir(), "opsis-dir-none"), {})).resolves.toBeUndefined();
  });
});

describe("fileIdentity", () => {
  it("changes when the file changes and fails for a missing file", async () => {
    const dir = await dirWith({ a: 10 });
    const file = path.join(dir, "a");
    const before = await fileIdentity(file);
    await writeFile(file, "longer text");
    expect(await fileIdentity(file)).not.toBe(before);
    await expect(fileIdentity(path.join(dir, "b"))).rejects.toThrow(/ENOENT/);
  });
});
