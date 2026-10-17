import { describe, expect, it } from "vitest";
import { frameAt } from "../shared/trace";
import { parseTrace } from "./model";

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

const parse = () =>
  parseTrace({
    trace: ndjson(EVENTS),
    network: ndjson(NETWORK),
    groups: ["nav", "login"],
    frameSrc: (file) => `src:${file}`,
  });

describe("trace parsing", () => {
  it("reads actions with their groups, targets, inputs and errors", () => {
    const trace = parse();
    expect(trace.viewport).toEqual({ width: 1280, height: 720 });
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
    expect([trace.start, trace.end]).toEqual([100, 910]);
  });

  it("reads console messages, page errors and network requests", () => {
    const trace = parse();
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
  });

  it("finds the frame showing a moment", () => {
    const { frames } = parse();
    expect(frames.map((f) => f.src)).toEqual([`src:${FRAME}`, `src:${LATER}`]);
    expect(frameAt(frames, 450)?.time).toBe(300);
    expect(frameAt(frames, 900)?.time).toBe(800);
    expect(frameAt(frames, 10)?.time).toBe(300);
  });
});
