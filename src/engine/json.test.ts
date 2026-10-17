import { existsSync } from "node:fs";
import { mkdtemp, readdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
import { z } from "zod";
import { nullIfMissing, readJson, readJsonKeepingBroken, writeJson } from "./json";

const made: string[] = [];
const schema = z.object({ n: z.number() });

async function tempFile(): Promise<string> {
  const dir = await mkdtemp(path.join(os.tmpdir(), "demoflow-json-"));
  made.push(dir);
  return path.join(dir, "data.json");
}

afterEach(async () => {
  vi.restoreAllMocks();
  await Promise.all(made.splice(0).map((d) => rm(d, { recursive: true, force: true })));
});

describe("reading JSON files", () => {
  it("returns what the schema accepts and nothing for a missing or wrong file", async () => {
    const file = await tempFile();
    expect(await readJson(file, schema)).toBeNull();
    await writeJson(file, { n: 1 });
    expect(await readJson(file, schema)).toEqual({ n: 1 });
    await writeFile(file, "{ broken");
    expect(await readJson(file, schema)).toBeNull();
    await writeJson(file, { n: "one" });
    expect(await readJson(file, schema)).toBeNull();
  });

  it("moves a file it can't read aside when asked to keep it", async () => {
    vi.spyOn(process.stderr, "write").mockReturnValue(true);
    const file = await tempFile();
    await writeFile(file, "{ broken");
    expect(await readJsonKeepingBroken(file, schema)).toBeNull();
    expect(existsSync(file)).toBe(false);
    expect(existsSync(`${file}.broken`)).toBe(true);
  });
});

describe("nullIfMissing", () => {
  it("turns a missing file or folder into null and passes other results and errors through", async () => {
    const file = await tempFile();
    expect(await nullIfMissing(readdir(file))).toBeNull();
    expect(await nullIfMissing(readdir(path.dirname(file)))).toEqual([]);
    await writeFile(file, "x");
    expect(await nullIfMissing(readdir(path.join(file, "inside")))).toBeNull();
    await expect(nullIfMissing(Promise.reject(new Error("denied")))).rejects.toThrow("denied");
  });
});
