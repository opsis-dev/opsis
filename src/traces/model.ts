import type {
  TraceAction,
  TraceConsole,
  TraceError,
  TraceFrame,
  TraceModel,
  TraceRequest,
} from "../shared/trace";
import { formatDuration, humanize } from "../shared/format";

type Json = Record<string, unknown>;

type ParsedTrace = Omit<TraceModel, "id" | "nodeId" | "label" | "ok" | "createdAt">;

type Call = { method: string; action: TraceAction };

const VERBS: Record<string, string> = {
  goto: "Navigate",
  dblclick: "Double click",
  setInputFiles: "Upload files",
  scrollIntoViewIfNeeded: "Scroll into view",
  waitForSelector: "Wait for",
  waitForTimeout: "Wait",
  evaluateExpression: "Evaluate",
  evaluateExpressionHandle: "Evaluate",
  setViewportSize: "Set viewport",
  mouseMove: "Move mouse",
  mouseWheel: "Scroll wheel",
  keyboardPress: "Press",
  keyboardType: "Type",
  keyboardDown: "Key down",
  keyboardUp: "Key up",
  keyboardInsertText: "Insert text",
  boundingBox: "Measure",
  textContent: "Read text",
  innerText: "Read text",
  getAttribute: "Read attribute",
  isVisible: "Check visible",
};

const COLLAPSIBLE = new Set(["mouseMove", "mouseWheel"]);
const PAUSE_MS = 100;

const LEVELS: Record<string, TraceConsole["level"]> = {
  log: "log",
  info: "info",
  warning: "warning",
  warn: "warning",
  error: "error",
  debug: "debug",
};

export function parseNdjson(text: string): Json[] {
  const out: Json[] = [];
  for (const line of text.split("\n")) {
    if (!line.trim()) continue;
    try {
      out.push(JSON.parse(line) as Json);
    } catch {
      continue;
    }
  }
  return out;
}

const str = (v: unknown) => (typeof v === "string" ? v : undefined);
const num = (v: unknown) => (typeof v === "number" && Number.isFinite(v) ? v : undefined);
const obj = (v: unknown) => (v && typeof v === "object" ? (v as Json) : {});

function readPoint(value: unknown, fallbackY?: number): TraceAction["point"] {
  const p = obj(value);
  const x = num(p.x);
  const y = num(p.y) ?? fallbackY;
  return x === undefined || y === undefined ? undefined : { x, y };
}

function readBox(value: unknown): TraceAction["box"] {
  const b = obj(value);
  const width = num(b.width);
  if (width === undefined) return undefined;
  return { x: num(b.x) ?? 0, y: num(b.y) ?? 0, width, height: num(b.height) ?? 0 };
}

function describe(method: string, params: Json): { title: string; target?: string } {
  const verb = VERBS[method] ?? humanize(method);
  const selector = str(params.selector);
  switch (method) {
    case "goto":
      return { title: verb, target: str(params.url) };
    case "fill":
    case "type":
    case "keyboardType":
    case "keyboardInsertText": {
      const value = str(params.value) ?? str(params.text);
      return { title: value === undefined ? verb : `${verb} "${value}"`, target: selector };
    }
    case "press":
    case "keyboardPress":
    case "keyboardDown":
    case "keyboardUp":
      return { title: `${verb} ${str(params.key) ?? ""}`.trim(), target: selector };
    case "waitForTimeout":
      return {
        title: `${verb} ${formatDuration(num(params.waitTimeout) ?? num(params.timeout) ?? 0, "precise")}`,
      };
    default:
      return { title: verb, target: selector };
  }
}

function errorMessage(value: unknown): string | undefined {
  const e = obj(value);
  return str(e.message) ?? str(obj(e.error).message) ?? str(obj(e.value).message);
}

function readViewport(e: Json): TraceModel["viewport"] {
  const v = obj(obj(e.options).viewport);
  const width = num(v.width);
  const height = num(v.height);
  return width && height ? { width, height } : null;
}

function beginCall(e: Json, nextGroup: () => string | undefined): Call | undefined {
  const id = str(e.callId);
  if (!id) return undefined;
  const method = str(e.method) ?? "";
  const group = method === "tracingGroup";
  const params = obj(e.params);
  const described = describe(method, params);
  const start = num(e.startTime) ?? 0;
  const point = method.startsWith("mouse") ? readPoint(params) : undefined;
  return {
    method,
    action: {
      id,
      parentId: str(e.parentId),
      group,
      nodeId: group ? nextGroup() : undefined,
      title: str(e.title) ?? described.title,
      target: described.target,
      start,
      end: start,
      count: 1,
      logs: [],
      ...(point ? { point } : {}),
    },
  };
}

function addLog(a: TraceAction, message: string): void {
  a.logs.push(message);
  const waiting = /^waiting for (.+)$/.exec(message);
  if (waiting && a.target && !a.logs.slice(0, -1).some((l) => l.startsWith("waiting for ")))
    a.target = waiting[1];
}

function addInput(a: TraceAction, e: Json): void {
  const point = readPoint(e.point, 0);
  const box = readBox(e.box);
  if (point) a.point = point;
  if (box) a.box = box;
}

function endAction(a: TraceAction, e: Json): void {
  a.end = num(e.endTime) ?? a.start;
  a.error = errorMessage(e.error);
  const point = a.point ?? readPoint(e.point, 0);
  if (point) a.point = point;
}

function readConsole(e: Json): TraceConsole | undefined {
  const text = str(e.text)?.replaceAll("%c", "");
  if (text === undefined) return undefined;
  return {
    level: LEVELS[str(e.messageType) ?? "log"] ?? "log",
    text,
    time: num(e.time) ?? 0,
    url: str(obj(e.location).url) || undefined,
  };
}

function readPageError(e: Json): TraceError | undefined {
  if (e.method !== "pageError") return undefined;
  const error = obj(obj(e.params).error);
  const inner = obj(error.error);
  return {
    message: str(inner.message) ?? str(obj(error.value).s) ?? "Page error",
    stack: str(inner.stack),
    time: num(e.time) ?? 0,
  };
}

function readFrame(
  e: Json,
  frameSrc: (file: string) => string | undefined,
): TraceFrame | undefined {
  const sha1 = str(e.sha1);
  const file = str(e.file) ?? (sha1 ? `resources/${sha1}` : undefined);
  const src = file ? frameSrc(file) : undefined;
  if (!src) return undefined;
  return {
    src,
    time: num(e.timestamp) ?? 0,
    width: num(e.width) ?? 0,
    height: num(e.height) ?? 0,
  };
}

type TraceEvents = {
  calls: Map<string, Call>;
  frames: TraceFrame[];
  console: TraceConsole[];
  errors: TraceError[];
  viewport: TraceModel["viewport"];
};

function readEvents(
  text: string,
  groups: string[],
  frameSrc: (file: string) => string | undefined,
): TraceEvents {
  const events: TraceEvents = {
    calls: new Map(),
    frames: [],
    console: [],
    errors: [],
    viewport: null,
  };
  let groupIndex = 0;
  const nextGroup = () => groups[groupIndex++];
  const keep = <T>(list: T[], item: T | undefined) => {
    if (item) list.push(item);
  };
  for (const e of parseNdjson(text)) {
    const action = events.calls.get(str(e.callId) ?? "")?.action;
    switch (e.type) {
      case "context-options":
        events.viewport = readViewport(e) ?? events.viewport;
        break;
      case "before": {
        const call = beginCall(e, nextGroup);
        if (call) events.calls.set(call.action.id, call);
        break;
      }
      case "log": {
        const message = str(e.message);
        if (action && message !== undefined) addLog(action, message);
        break;
      }
      case "input":
        if (action) addInput(action, e);
        break;
      case "after":
        if (!action) break;
        endAction(action, e);
        if (action.error)
          events.errors.push({ message: action.error, time: action.end, actionId: action.id });
        break;
      case "console":
        keep(events.console, readConsole(e));
        break;
      case "event":
        keep(events.errors, readPageError(e));
        break;
      case "screencast-frame":
        keep(events.frames, readFrame(e, frameSrc));
        break;
    }
  }
  return events;
}

function parseNetwork(text: string): TraceRequest[] {
  return parseNdjson(text)
    .filter((e) => e.type === "resource-snapshot")
    .map((e) => {
      const s = obj(e.snapshot);
      const request = obj(s.request);
      const response = obj(s.response);
      const content = obj(response.content);
      const size = num(content.size) ?? -1;
      return {
        method: str(request.method) ?? "GET",
        url: str(request.url) ?? "",
        status: num(response.status) ?? -1,
        mimeType: str(content.mimeType) ?? "",
        resourceType: str(s["_resourceType"]) ?? "",
        size: size >= 0 ? size : (num(response["_transferSize"]) ?? num(response.bodySize) ?? -1),
        start: num(s["_monotonicTime"]) ?? 0,
        duration: num(s.time) ?? 0,
        failure: str(response["_failureText"]),
      };
    })
    .sort((a, b) => a.start - b.start);
}

const absorbs = (prev: Call, next: Call): boolean =>
  COLLAPSIBLE.has(prev.method) &&
  prev.action.parentId === next.action.parentId &&
  !prev.action.error &&
  !next.action.error &&
  (next.method === prev.method ||
    (next.method === "waitForTimeout" && next.action.end - next.action.start < PAUSE_MS));

function collapseActions(calls: Iterable<Call>): TraceAction[] {
  const collapsed: Call[] = [];
  for (const call of [...calls].sort((a, b) => a.action.start - b.action.start)) {
    const prev = collapsed.at(-1);
    if (prev && absorbs(prev, call)) {
      const into = prev.action;
      if (call.method === prev.method) into.count += 1;
      into.end = Math.max(into.end, call.action.end);
      into.logs.push(...call.action.logs);
      if (call.action.point) into.point = call.action.point;
      continue;
    }
    collapsed.push({
      method: call.method,
      action: { ...call.action, logs: [...call.action.logs] },
    });
  }
  return collapsed.map(({ action: a }) =>
    a.count > 1 ? { ...a, title: `${a.title} ×${a.count}` } : a,
  );
}

function timeBounds(actions: TraceAction[], frames: TraceFrame[]): { start: number; end: number } {
  const times = [...actions.flatMap((a) => [a.start, a.end]), ...frames.map((f) => f.time)].filter(
    (t) => t > 0,
  );
  return times.length
    ? { start: Math.min(...times), end: Math.max(...times) }
    : { start: 0, end: 0 };
}

export function parseTrace(input: {
  trace: string;
  network: string;
  groups: string[];
  frameSrc: (file: string) => string | undefined;
}): ParsedTrace {
  const events = readEvents(input.trace, input.groups, input.frameSrc);
  const actions = collapseActions(events.calls.values());
  const frames = events.frames.sort((a, b) => a.time - b.time);
  return {
    ...timeBounds(actions, frames),
    viewport: events.viewport,
    actions,
    frames,
    console: events.console.sort((a, b) => a.time - b.time),
    errors: events.errors.sort((a, b) => a.time - b.time),
    network: parseNetwork(input.network),
  };
}
