import { existsSync } from "node:fs";
import { mkdir, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
import { z } from "zod";
import { atomicWrite, nullIfMissing, readJson, readJsonKeepingBroken, writeJson } from "./json";
import { tempDirs } from "./test-temp-dirs";

const schema = z.object({ n: z.number() });
const tempDir = tempDirs("demoflow-json-");

const tempFile = async (): Promise<string> => path.join(await tempDir(), "data.json");

afterEach(() => {
  vi.restoreAllMocks();
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

describe("atomicWrite", () => {
  it("leaves no temporary file behind when the write fails", async () => {
    const file = await tempFile();
    await mkdir(path.join(file, "inside"), { recursive: true });
    await expect(atomicWrite(file, "x")).rejects.toHaveProperty("code");
    expect(await readdir(path.dirname(file))).toEqual([path.basename(file)]);
  });
});
