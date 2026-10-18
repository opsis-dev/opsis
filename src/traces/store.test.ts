import { existsSync } from "node:fs";
import type * as FsPromises from "node:fs/promises";
import { readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { strToU8, zipSync } from "fflate";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { sha256Hex } from "../engine/hash";
import { projectPaths } from "../engine/project-paths";
import { tempDirs } from "../engine/test-temp-dirs";
import { applyRetention, ingestTrace, readTrace } from "./store";
import { FRAME, LATER, ndjson } from "./test-fixtures";

const unreadable = vi.hoisted(() => new Set<string>());
vi.mock("node:fs/promises", async (importOriginal) => {
  const fs = await importOriginal<typeof FsPromises>();
  return {
    ...fs,
    readFile: (...args: Parameters<typeof fs.readFile>) =>
      unreadable.has(String(args[0]))
        ? Promise.reject(Object.assign(new Error("EACCES: permission denied"), { code: "EACCES" }))
        : fs.readFile(...args),
  };
});

const project = tempDirs("demoflow-trace-");
const START = Date.UTC(2026, 0, 1);

beforeEach(() => {
  vi.useFakeTimers({ toFake: ["Date"] });
  vi.setSystemTime(START);
});

afterEach(() => {
  vi.useRealTimers();
});

const EVENTS = [
  { type: "before", callId: "c@1", startTime: 100, method: "tracingGroup", title: "Open site" },
  { type: "after", callId: "c@1", endTime: 410 },
  { type: "screencast-frame", file: FRAME, width: 1280, height: 720, timestamp: 300 },
  { type: "screencast-frame", file: LATER, width: 1280, height: 720, timestamp: 800 },
];

const FRAMES = [FRAME, LATER];

const traceFiles = (trace: string): Record<string, Uint8Array> => ({
  "trace.trace": strToU8(trace),
  "trace.network": strToU8(""),
  "resources/page.html": strToU8("<html>shared</html>".repeat(200)),
  [FRAME]: new Uint8Array([255, 216, 255, 1]),
  [LATER]: new Uint8Array([255, 216, 255, 2]),
});

const distinctFiles = (...traces: string[]) =>
  new Set(traces.flatMap((t) => Object.values(traceFiles(t)).map((data) => sha256Hex(data)))).size;

async function traceZip(dir: string, name: string, trace = ndjson(EVENTS)) {
  const file = path.join(dir, name);
  await writeFile(file, zipSync(traceFiles(trace)));
  return file;
}

const meta = (ok = true) => ({ nodeId: "rec", label: "Record", ok, groups: ["nav"] });

async function blobs(dir: string) {
  const root = path.join(projectPaths(dir).traces, "blobs");
  const shards = await readdir(root);
  return (await Promise.all(shards.map((s) => readdir(path.join(root, s))))).flat();
}

describe("trace store", () => {
  it("reads a stored trace with its groups and its frames from the blob store", async () => {
    const dir = await project();
    const m = await ingestTrace(dir, await traceZip(dir, "a.zip"), meta(false));
    const trace = await readTrace(dir, m.id);

    expect(trace).toMatchObject({ id: m.id, nodeId: "rec", ok: false, start: 100, end: 800 });
    expect(trace.actions.map((a) => [a.title, a.nodeId])).toEqual([["Open site", "nav"]]);
    expect(trace.frames).toHaveLength(2);
    for (const f of trace.frames) {
      expect(f.src).toMatch(/^project:\.demoflow\/traces\/blobs\/[0-9a-f]{2}\/[0-9a-f]{64}\.jpeg$/);
      expect(existsSync(path.join(dir, f.src.slice("project:".length)))).toBe(true);
    }
  });

  it("stores identical resources once across runs, compressed text and raw images", async () => {
    const dir = await project();
    await ingestTrace(dir, await traceZip(dir, "a.zip", "one"), meta());
    vi.setSystemTime(START + 1_000);
    await ingestTrace(dir, await traceZip(dir, "b.zip", "two"), meta());
    const names = await blobs(dir);
    expect(names).toHaveLength(distinctFiles("one", "two"));
    expect(names.filter((n) => n.endsWith(".jpeg"))).toHaveLength(FRAMES.length);
    expect(names.filter((n) => n.endsWith(".zst"))).toHaveLength(
      distinctFiles("one", "two") - FRAMES.length,
    );
  });

  it("keeps only failures when asked and collects orphaned blobs", async () => {
    const dir = await project();
    const ok = await ingestTrace(dir, await traceZip(dir, "a.zip", "ok"), meta(true));
    vi.setSystemTime(START + 1_000);
    const bad = await ingestTrace(dir, await traceZip(dir, "b.zip", "bad"), meta(false));
    await applyRetention(dir, { retention: "failures", keepLast: 10 });
    await expect(readTrace(dir, ok.id)).rejects.toThrow("This trace can't be read.");
    expect((await readTrace(dir, bad.id)).ok).toBe(false);
    expect(await blobs(dir)).toHaveLength(distinctFiles("bad"));
  });

  it("keeps every blob when a manifest can't be read for a moment", async () => {
    const dir = await project();
    const failed = await ingestTrace(dir, await traceZip(dir, "a.zip", "kept"), meta(false));
    vi.setSystemTime(START + 1_000);
    await ingestTrace(dir, await traceZip(dir, "b.zip", "ok"), meta(true));
    const before = await blobs(dir);
    const manifest = path.join(projectPaths(dir).traces, "runs", `${failed.id}.json.zst`);
    unreadable.add(manifest);
    try {
      await expect(applyRetention(dir, { retention: "failures", keepLast: 10 })).rejects.toThrow(
        "EACCES",
      );
    } finally {
      unreadable.delete(manifest);
    }
    expect((await blobs(dir)).toSorted()).toEqual(before.toSorted());
  });

  it("removes broken manifests and still applies retention", async () => {
    const dir = await project();
    const ok = await ingestTrace(dir, await traceZip(dir, "a.zip", "ok"), meta(true));
    const broken = path.join(projectPaths(dir).traces, "runs", "broken-1.json.zst");
    await writeFile(broken, "not zstd");
    expect(await applyRetention(dir, { retention: "failures", keepLast: 10 })).toEqual(new Set());
    expect(existsSync(broken)).toBe(false);
    await expect(readTrace(dir, ok.id)).rejects.toThrow("This trace can't be read.");
    expect(await blobs(dir)).toEqual([]);
  });

  it("says a missing trace can't be read", async () => {
    const dir = await project();
    await expect(readTrace(dir, "gone-1")).rejects.toThrow("This trace can't be read.");
  });

  it("reads traces of nodes whose ids aren't plain names", async () => {
    const dir = await project();
    const m = await ingestTrace(dir, await traceZip(dir, "a.zip"), {
      ...meta(),
      nodeId: "my node/1.x",
    });
    const trace = await readTrace(dir, m.id);
    expect(trace).toMatchObject({ id: m.id, nodeId: "my node/1.x" });
  });

  it("rejects trace ids that are not plain names", async () => {
    const dir = await project();
    await expect(readTrace(dir, "../../etc/passwd")).rejects.toThrow("Invalid trace id");
  });
});
