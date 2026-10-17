import { existsSync } from "node:fs";
import { mkdtemp, readdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { strToU8, zipSync } from "fflate";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { applyRetention, ingestTrace, listTraces, readTrace, traceRoot } from "./store";
import { FRAME, LATER, ndjson } from "./test-fixtures";

const dirs: string[] = [];
const START = Date.UTC(2026, 0, 1);

beforeEach(() => {
  vi.useFakeTimers({ toFake: ["Date"] });
  vi.setSystemTime(START);
});

afterEach(async () => {
  vi.useRealTimers();
  for (const d of dirs.splice(0)) await rm(d, { recursive: true, force: true });
});

async function project() {
  const dir = await mkdtemp(path.join(tmpdir(), "demoflow-trace-"));
  dirs.push(dir);
  return dir;
}

const EVENTS = [
  { type: "before", callId: "c@1", startTime: 100, method: "tracingGroup", title: "Open site" },
  { type: "after", callId: "c@1", endTime: 410 },
  { type: "screencast-frame", file: FRAME, width: 1280, height: 720, timestamp: 300 },
  { type: "screencast-frame", file: LATER, width: 1280, height: 720, timestamp: 800 },
];

async function traceZip(dir: string, name: string, trace = ndjson(EVENTS)) {
  const file = path.join(dir, name);
  await writeFile(
    file,
    zipSync({
      "trace.trace": strToU8(trace),
      "trace.network": strToU8(""),
      "resources/page.html": strToU8("<html>shared</html>".repeat(200)),
      [FRAME]: new Uint8Array([255, 216, 255, 1]),
      [LATER]: new Uint8Array([255, 216, 255, 2]),
    }),
  );
  return file;
}

const meta = (ok = true) => ({ nodeId: "rec", label: "Record", ok, groups: ["nav"] });

async function blobs(dir: string) {
  const root = path.join(traceRoot(dir), "blobs");
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
    expect(names).toHaveLength(6);
    expect(names.filter((n) => n.endsWith(".jpeg"))).toHaveLength(2);
    expect(names.filter((n) => n.endsWith(".zst"))).toHaveLength(4);
  });

  it("keeps only failures when asked and collects orphaned blobs", async () => {
    const dir = await project();
    await ingestTrace(dir, await traceZip(dir, "a.zip", "ok"), meta(true));
    vi.setSystemTime(START + 1_000);
    await ingestTrace(dir, await traceZip(dir, "b.zip", "bad"), meta(false));
    await applyRetention(dir, { retention: "failures", keepLast: 10 });
    expect((await listTraces(dir)).map((m) => m.ok)).toEqual([false]);
    expect(await blobs(dir)).toHaveLength(5);
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
