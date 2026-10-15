import { existsSync } from "node:fs";
import { mkdtemp, readdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { strToU8, zipSync } from "fflate";
import { afterEach, describe, expect, it } from "vitest";
import { frameAt } from "../shared/trace";
import { applyRetention, ingestTrace, listTraces, readTrace, traceRoot } from "./store";

const dirs: string[] = [];
afterEach(async () => {
  for (const d of dirs.splice(0)) await rm(d, { recursive: true, force: true });
});

async function project() {
  const dir = await mkdtemp(path.join(tmpdir(), "demoflow-trace-"));
  dirs.push(dir);
  return dir;
}

const ndjson = (events: object[]) => events.map((e) => JSON.stringify(e)).join("\n");

const FRAME = "screencast/page@1-1000.jpeg";
const LATER = "screencast/page@1-2000.jpeg";

const EVENTS = [
  {
    type: "context-options",
    options: { viewport: { width: 1280, height: 720 } },
  },
  {
    type: "before",
    callId: "c@1",
    startTime: 100,
    class: "Tracing",
    method: "tracingGroup",
    title: "Open site",
    params: {},
  },
  {
    type: "before",
    callId: "c@2",
    parentId: "c@1",
    startTime: 110,
    class: "Frame",
    method: "goto",
    params: { url: "https://example.test/login" },
  },
  { type: "after", callId: "c@2", endTime: 400 },
  { type: "after", callId: "c@1", endTime: 410 },
  {
    type: "before",
    callId: "c@3",
    startTime: 420,
    class: "Tracing",
    method: "tracingGroup",
    title: "Sign in",
    params: {},
  },
  {
    type: "before",
    callId: "c@4",
    parentId: "c@3",
    startTime: 430,
    class: "Frame",
    method: "fill",
    params: { selector: 'internal:label="Username"i', value: "tomsmith" },
  },
  { type: "log", callId: "c@4", time: 431, message: "waiting for getByLabel('Username')" },
  { type: "log", callId: "c@4", time: 432, message: "  locator resolved to <input/>" },
  { type: "input", callId: "c@4", box: { x: 10, y: 20, width: 100, height: 30 } },
  { type: "after", callId: "c@4", endTime: 450 },
  {
    type: "before",
    callId: "c@5",
    parentId: "c@3",
    startTime: 460,
    class: "Page",
    method: "mouseMove",
    params: { x: 1, y: 1 },
  },
  {
    type: "before",
    callId: "c@5w",
    parentId: "c@3",
    startTime: 461,
    class: "Frame",
    method: "waitForTimeout",
    params: { waitTimeout: 8 },
  },
  { type: "after", callId: "c@5w", endTime: 461.5 },
  { type: "after", callId: "c@5", endTime: 461 },
  {
    type: "before",
    callId: "c@6",
    parentId: "c@3",
    startTime: 462,
    class: "Page",
    method: "mouseMove",
    params: { x: 2, y: 2 },
  },
  { type: "after", callId: "c@6", endTime: 463 },
  {
    type: "before",
    callId: "c@7",
    parentId: "c@3",
    startTime: 470,
    class: "Frame",
    method: "click",
    params: { selector: "#missing" },
  },
  {
    type: "after",
    callId: "c@7",
    endTime: 900,
    error: { message: "Timeout 400ms exceeded.", name: "TimeoutError" },
  },
  { type: "after", callId: "c@3", endTime: 910 },
  {
    type: "console",
    messageType: "warning",
    text: "careful",
    time: 440,
    location: { url: "https://example.test/app.js" },
  },
  {
    type: "event",
    time: 500,
    class: "BrowserContext",
    method: "pageError",
    params: { error: { error: { message: "boom", stack: "Error: boom\n    at x" } } },
  },
  {
    type: "screencast-frame",
    pageId: "page@1",
    file: FRAME,
    width: 1280,
    height: 720,
    timestamp: 300,
  },
  {
    type: "screencast-frame",
    pageId: "page@1",
    file: LATER,
    width: 1280,
    height: 720,
    timestamp: 800,
  },
];

const NETWORK = [
  {
    type: "resource-snapshot",
    snapshot: {
      request: { method: "GET", url: "https://example.test/login" },
      response: { status: 200, content: { size: 2443, mimeType: "text/html" } },
      _monotonicTime: 120,
      time: 250,
      _resourceType: "document",
    },
  },
  {
    type: "resource-snapshot",
    snapshot: {
      request: { method: "GET", url: "https://tracker.test/event" },
      response: {
        status: -1,
        content: { size: -1, mimeType: "x-unknown" },
        _failureText: "net::ERR_NAME_NOT_RESOLVED",
      },
      _monotonicTime: 130,
      time: 20,
    },
  },
];

async function traceZip(dir: string, name: string, trace = ndjson(EVENTS)) {
  const file = path.join(dir, name);
  await writeFile(
    file,
    zipSync({
      "trace.trace": strToU8(trace),
      "trace.network": strToU8(ndjson(NETWORK)),
      "resources/page.html": strToU8("<html>shared</html>".repeat(200)),
      [FRAME]: new Uint8Array([255, 216, 255, 1]),
      [LATER]: new Uint8Array([255, 216, 255, 2]),
    }),
  );
  return file;
}

const meta = (ok = true) => ({ nodeId: "rec", label: "Record", ok, groups: ["nav", "login"] });

async function blobs(dir: string) {
  const root = path.join(traceRoot(dir), "blobs");
  const shards = await readdir(root);
  return (await Promise.all(shards.map((s) => readdir(path.join(root, s))))).flat();
}

describe("trace store", () => {
  it("reads a stored trace into actions, frames, console, errors and network", async () => {
    const dir = await project();
    const m = await ingestTrace(dir, await traceZip(dir, "a.zip"), meta(false));
    const trace = await readTrace(dir, m.id);

    expect(trace).toMatchObject({
      id: m.id,
      nodeId: "rec",
      ok: false,
      viewport: { width: 1280, height: 720 },
    });
    expect(trace.actions.map((a) => [a.title, a.parentId ?? null, a.nodeId ?? null])).toEqual([
      ["Open site", null, "nav"],
      ["Navigate", "c@1", null],
      ["Sign in", null, "login"],
      ['Fill "tomsmith"', "c@3", null],
      ["Move mouse ×2", "c@3", null],
      ["Click", "c@3", null],
    ]);
    const fill = trace.actions[3];
    expect(fill.target).toBe("getByLabel('Username')");
    expect(fill.box).toEqual({ x: 10, y: 20, width: 100, height: 30 });
    expect(trace.actions[4].point).toEqual({ x: 2, y: 2 });
    expect(trace.actions[1].target).toBe("https://example.test/login");
    expect(trace.actions[5]).toMatchObject({
      target: "#missing",
      error: "Timeout 400ms exceeded.",
    });

    expect(trace.console).toEqual([
      { level: "warning", text: "careful", time: 440, url: "https://example.test/app.js" },
    ]);
    expect(trace.errors.map((e) => [e.message, e.actionId ?? null])).toEqual([
      ["boom", null],
      ["Timeout 400ms exceeded.", "c@7"],
    ]);
    expect(trace.network.map((r) => [r.url, r.status, r.size, r.failure ?? null])).toEqual([
      ["https://example.test/login", 200, 2443, null],
      ["https://tracker.test/event", -1, -1, "net::ERR_NAME_NOT_RESOLVED"],
    ]);

    expect(trace.frames).toHaveLength(2);
    for (const f of trace.frames) {
      expect(f.src).toMatch(/^project:\.demoflow\/traces\/blobs\/[0-9a-f]{2}\/[0-9a-f]{64}\.jpeg$/);
      expect(existsSync(path.join(dir, f.src.slice("project:".length)))).toBe(true);
    }
    expect(frameAt(trace.frames, 450)?.time).toBe(300);
    expect(frameAt(trace.frames, 900)?.time).toBe(800);
    expect(frameAt(trace.frames, 10)?.time).toBe(300);
    expect([trace.start, trace.end]).toEqual([100, 910]);
  });

  it("stores identical resources once across runs, compressed text and raw images", async () => {
    const dir = await project();
    await ingestTrace(dir, await traceZip(dir, "a.zip", "one"), meta());
    await ingestTrace(dir, await traceZip(dir, "b.zip", "two"), meta());
    const names = await blobs(dir);
    expect(names).toHaveLength(6);
    expect(names.filter((n) => n.endsWith(".jpeg"))).toHaveLength(2);
    expect(names.filter((n) => n.endsWith(".zst"))).toHaveLength(4);
  });

  it("keeps only failures when asked and collects orphaned blobs", async () => {
    const dir = await project();
    await ingestTrace(dir, await traceZip(dir, "a.zip", "ok"), meta(true));
    await new Promise((r) => setTimeout(r, 5));
    await ingestTrace(dir, await traceZip(dir, "b.zip", "bad"), meta(false));
    await applyRetention(dir, { retention: "failures", keepLast: 10 });
    expect((await listTraces(dir)).map((m) => m.ok)).toEqual([false]);
    expect(await blobs(dir)).toHaveLength(5);
  });

  it("rejects trace ids that are not plain names", async () => {
    const dir = await project();
    await expect(readTrace(dir, "../../etc/passwd")).rejects.toThrow("Invalid trace id");
  });
});
