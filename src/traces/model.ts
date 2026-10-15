import type {
  TraceAction,
  TraceConsole,
  TraceError,
  TraceFrame,
  TraceModel,
  TraceRequest,
} from "../shared/trace";

type Json = Record<string, unknown>;

export type ParsedTrace = Omit<TraceModel, "id" | "nodeId" | "label" | "ok" | "createdAt">;

const VERBS: Record<string, string> = {
  goto: "Navigate",
  reload: "Reload",
  goBack: "Go back",
  goForward: "Go forward",
  newPage: "New page",
  click: "Click",
  dblclick: "Double click",
  tap: "Tap",
  fill: "Fill",
  press: "Press",
  type: "Type",
  hover: "Hover",
  check: "Check",
  uncheck: "Uncheck",
  selectOption: "Select option",
  setInputFiles: "Upload files",
  focus: "Focus",
  dragAndDrop: "Drag and drop",
  scrollIntoViewIfNeeded: "Scroll into view",
  waitForSelector: "Wait for",
  waitForTimeout: "Wait",
  waitForFunction: "Wait for function",
  evaluateExpression: "Evaluate",
  evaluateExpressionHandle: "Evaluate",
  screenshot: "Screenshot",
  setViewportSize: "Set viewport",
  mouseMove: "Move mouse",
  mouseDown: "Mouse down",
  mouseUp: "Mouse up",
  mouseClick: "Mouse click",
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
  close: "Close",
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

function lines(text: string): Json[] {
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

function humanize(method: string): string {
  const words = method.replace(/([a-z])([A-Z])/g, "$1 $2").toLowerCase();
  return words.charAt(0).toUpperCase() + words.slice(1);
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
      return { title: `${verb} ${num(params.waitTimeout) ?? num(params.timeout) ?? 0}ms` };
    default:
      return { title: verb, target: selector };
  }
}

function errorMessage(value: unknown): string | undefined {
  const e = obj(value);
  return str(e.message) ?? str(obj(e.error).message) ?? str(obj(e.value).message);
}

export function parseTrace(input: {
  trace: string;
  network: string;
  groups: string[];
  frameSrc: (file: string) => string | undefined;
}): ParsedTrace {
  const actions = new Map<string, TraceAction & { method: string }>();
  const frames: TraceFrame[] = [];
  const consoleMessages: TraceConsole[] = [];
  const errors: TraceError[] = [];
  let viewport: TraceModel["viewport"] = null;
  let groupIndex = 0;

  for (const e of lines(input.trace)) {
    switch (e.type) {
      case "context-options": {
        const v = obj(obj(e.options).viewport);
        const width = num(v.width);
        const height = num(v.height);
        if (width && height) viewport = { width, height };
        break;
      }
      case "before": {
        const id = str(e.callId);
        if (!id) break;
        const method = str(e.method) ?? "";
        const group = method === "tracingGroup";
        const described = describe(method, obj(e.params));
        const start = num(e.startTime) ?? 0;
        const params = obj(e.params);
        const pointer =
          method.startsWith("mouse") && num(params.x) !== undefined && num(params.y) !== undefined
            ? { x: num(params.x)!, y: num(params.y)! }
            : undefined;
        actions.set(id, {
          id,
          method,
          parentId: str(e.parentId),
          group,
          nodeId: group ? input.groups[groupIndex++] : undefined,
          title: str(e.title) ?? described.title,
          target: described.target,
          start,
          end: start,
          count: 1,
          logs: [],
          ...(pointer ? { point: pointer } : {}),
        });
        break;
      }
      case "log": {
        const a = actions.get(str(e.callId) ?? "");
        const message = str(e.message);
        if (!a || message === undefined) break;
        a.logs.push(message);
        const waiting = /^waiting for (.+)$/.exec(message);
        if (waiting && a.target && !a.logs.slice(0, -1).some((l) => l.startsWith("waiting for ")))
          a.target = waiting[1];
        break;
      }
      case "input": {
        const a = actions.get(str(e.callId) ?? "");
        if (!a) break;
        const p = obj(e.point);
        const b = obj(e.box);
        if (num(p.x) !== undefined) a.point = { x: num(p.x)!, y: num(p.y) ?? 0 };
        if (num(b.width) !== undefined)
          a.box = {
            x: num(b.x) ?? 0,
            y: num(b.y) ?? 0,
            width: num(b.width) ?? 0,
            height: num(b.height) ?? 0,
          };
        break;
      }
      case "after": {
        const a = actions.get(str(e.callId) ?? "");
        if (!a) break;
        a.end = num(e.endTime) ?? a.start;
        a.error = errorMessage(e.error);
        const p = obj(e.point);
        if (!a.point && num(p.x) !== undefined) a.point = { x: num(p.x)!, y: num(p.y) ?? 0 };
        if (a.error) errors.push({ message: a.error, time: a.end, actionId: a.id });
        break;
      }
      case "console": {
        const text = str(e.text)?.replaceAll("%c", "");
        if (text === undefined) break;
        consoleMessages.push({
          level: LEVELS[str(e.messageType) ?? "log"] ?? "log",
          text,
          time: num(e.time) ?? 0,
          url: str(obj(e.location).url) || undefined,
        });
        break;
      }
      case "event": {
        if (e.method !== "pageError") break;
        const error = obj(obj(e.params).error);
        const inner = obj(error.error);
        errors.push({
          message: str(inner.message) ?? str(obj(error.value).s) ?? "Page error",
          stack: str(inner.stack),
          time: num(e.time) ?? 0,
        });
        break;
      }
      case "screencast-frame": {
        const file = str(e.file) ?? (str(e.sha1) ? `resources/${str(e.sha1)}` : undefined);
        const src = file ? input.frameSrc(file) : undefined;
        if (!src) break;
        frames.push({
          src,
          time: num(e.timestamp) ?? 0,
          width: num(e.width) ?? 0,
          height: num(e.height) ?? 0,
        });
        break;
      }
    }
  }

  const network: TraceRequest[] = [];
  for (const e of lines(input.network)) {
    if (e.type !== "resource-snapshot") continue;
    const s = obj(e.snapshot);
    const request = obj(s.request);
    const response = obj(s.response);
    const content = obj(response.content);
    const size = num(content.size) ?? -1;
    network.push({
      method: str(request.method) ?? "GET",
      url: str(request.url) ?? "",
      status: num(response.status) ?? -1,
      mimeType: str(content.mimeType) ?? "",
      resourceType: str(s["_resourceType"]) ?? "",
      size: size >= 0 ? size : (num(response["_transferSize"]) ?? num(response.bodySize) ?? -1),
      start: num(s["_monotonicTime"]) ?? 0,
      duration: num(s.time) ?? 0,
      failure: str(response["_failureText"]),
    });
  }

  const ordered = [...actions.values()].sort((a, b) => a.start - b.start);
  const collapsed: Array<TraceAction & { method: string }> = [];
  for (const a of ordered) {
    const prev = collapsed.at(-1);
    if (
      prev &&
      COLLAPSIBLE.has(prev.method) &&
      prev.parentId === a.parentId &&
      !prev.error &&
      !a.error &&
      (a.method === prev.method || (a.method === "waitForTimeout" && a.end - a.start < PAUSE_MS))
    ) {
      if (a.method === prev.method) prev.count += 1;
      prev.end = Math.max(prev.end, a.end);
      prev.logs.push(...a.logs);
      if (a.point) prev.point = a.point;
      continue;
    }
    collapsed.push({ ...a, logs: [...a.logs] });
  }

  frames.sort((a, b) => a.time - b.time);
  network.sort((a, b) => a.start - b.start);
  const times = [
    ...collapsed.flatMap((a) => [a.start, a.end]),
    ...frames.map((f) => f.time),
  ].filter((t) => t > 0);

  return {
    start: times.length ? Math.min(...times) : 0,
    end: times.length ? Math.max(...times) : 0,
    viewport,
    actions: collapsed.map(({ method: _method, ...a }) =>
      a.count > 1 ? { ...a, title: `${a.title} ×${a.count}` } : a,
    ),
    frames,
    console: consoleMessages.sort((a, b) => a.time - b.time),
    errors: errors.sort((a, b) => a.time - b.time),
    network,
  };
}
