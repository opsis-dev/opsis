import { z } from "zod";
import { formatPreciseDuration, humanize } from "../shared/format";
import { isOneOf, isRecord, isString, tryParseJson } from "../shared/guards";
import {
  CONSOLE_LEVELS,
  type TraceAction,
  type TraceConsole,
  type TraceError,
  type TraceFrame,
  type TraceModel,
  type TraceHeader,
  type TraceRequest,
} from "../shared/trace";

type Json = Record<string, unknown>;

type ParsedTrace = Omit<TraceModel, keyof TraceHeader>;

type Call = { method: string; action: TraceAction };

const VERBS: Partial<Record<string, string>> = {
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

const CONSOLE_ALIASES: Partial<Record<string, TraceConsole["level"]>> = { warn: "warning" };

const consoleLevel = (type = "log"): TraceConsole["level"] =>
  isOneOf(CONSOLE_LEVELS, type) ? type : (CONSOLE_ALIASES[type] ?? "log");

const TEXT_PARTS = ["trace", "network"] as const;
export type TraceTextPart = (typeof TEXT_PARTS)[number];
const TEXT_PART_NAME = /(?:^|\/)(?:\d+-)?trace\.(\w+)$/;

export const traceTextPart = (name: string): TraceTextPart | undefined => {
  const ext = TEXT_PART_NAME.exec(name)?.[1];
  return TEXT_PARTS.find((part) => part === ext);
};

const parseLine = (line: string): Json[] => {
  const json = tryParseJson(line);
  return json.ok && isRecord(json.value) ? [json.value] : [];
};

export const parseNdjson = (ndjson: string): Json[] =>
  ndjson
    .split("\n")
    .filter((line) => line.trim())
    .flatMap(parseLine);

const text = z.string().optional().catch(undefined);
const count = z.number().optional().catch(undefined);
const lenient = <S extends z.ZodRawShape>(shape: S) =>
  z.object(shape).catch(() => z.object(shape).parse({}));

const pointShape = lenient({ x: count, y: count });
const boxShape = lenient({ x: count, y: count, width: count, height: count });
const messageShape = lenient({ message: text, stack: text, s: text });
const errorShape = lenient({ message: text, error: messageShape, value: messageShape });

const callParams = lenient({
  selector: text,
  url: text,
  value: text,
  text,
  key: text,
  waitTimeout: count,
  timeout: count,
  x: count,
  y: count,
});

const EVENTS = {
  "context-options": z.object({
    options: lenient({ viewport: lenient({ width: count, height: count }) }),
  }),
  before: z.object({
    callId: text,
    method: text,
    title: text,
    parentId: text,
    params: callParams,
    startTime: count,
  }),
  log: z.object({ callId: text, message: text }),
  input: z.object({ callId: text, point: pointShape, box: boxShape }),
  after: z.object({ callId: text, endTime: count, error: errorShape, point: pointShape }),
  console: z.object({
    text,
    messageType: text,
    time: count,
    location: lenient({ url: text }),
  }),
  event: z.object({ method: text, params: lenient({ error: errorShape }), time: count }),
  "screencast-frame": z.object({
    sha1: text,
    file: text,
    timestamp: count,
    width: count,
    height: count,
  }),
};

type Event<K extends keyof typeof EVENTS> = z.infer<(typeof EVENTS)[K]>;

const pointOf = (p: z.infer<typeof pointShape>, fallbackY?: number): TraceAction["point"] => {
  const y = p.y ?? fallbackY;
  return p.x === undefined || y === undefined ? undefined : { x: p.x, y };
};

const boxOf = (b: z.infer<typeof boxShape>): TraceAction["box"] =>
  b.width === undefined
    ? undefined
    : { x: b.x ?? 0, y: b.y ?? 0, width: b.width, height: b.height ?? 0 };

function describeCall(
  method: string,
  params: z.infer<typeof callParams>,
): { title: string; target?: string } {
  const verb = VERBS[method] ?? humanize(method);
  const { selector } = params;
  switch (method) {
    case "goto":
      return { title: verb, target: params.url };
    case "fill":
    case "type":
    case "keyboardType":
    case "keyboardInsertText": {
      const value = params.value ?? params.text;
      return { title: value === undefined ? verb : `${verb} "${value}"`, target: selector };
    }
    case "press":
    case "keyboardPress":
    case "keyboardDown":
    case "keyboardUp":
      return { title: `${verb} ${params.key ?? ""}`.trim(), target: selector };
    case "waitForTimeout":
      return {
        title: `${verb} ${formatPreciseDuration(params.waitTimeout ?? params.timeout ?? 0)}`,
      };
    default:
      return { title: verb, target: selector };
  }
}

const errorMessageOf = (e: z.infer<typeof errorShape>): string | undefined =>
  e.message ?? e.error.message ?? e.value.message;

function viewportOf(e: Event<"context-options">): TraceModel["viewport"] {
  const { width, height } = e.options.viewport;
  return width && height ? { width, height } : null;
}

function beginCall(e: Event<"before">, nextGroup: () => string | undefined): Call | undefined {
  const id = e.callId;
  if (!id) return undefined;
  const method = e.method ?? "";
  const group = method === "tracingGroup";
  const described = describeCall(method, e.params);
  const start = e.startTime ?? 0;
  const point = method.startsWith("mouse") ? pointOf(e.params) : undefined;
  return {
    method,
    action: {
      id,
      parentId: e.parentId,
      group,
      nodeId: group ? nextGroup() : undefined,
      title: e.title ?? described.title,
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

function addInput(a: TraceAction, e: Event<"input">): void {
  const point = pointOf(e.point, 0);
  const box = boxOf(e.box);
  if (point) a.point = point;
  if (box) a.box = box;
}

function endAction(a: TraceAction, e: Event<"after">): void {
  a.end = e.endTime ?? a.start;
  a.error = errorMessageOf(e.error);
  const point = a.point ?? pointOf(e.point, 0);
  if (point) a.point = point;
}

function consoleOf(e: Event<"console">): TraceConsole | undefined {
  const message = e.text?.replaceAll("%c", "");
  if (message === undefined) return undefined;
  return {
    level: consoleLevel(e.messageType),
    text: message,
    time: e.time ?? 0,
    url: e.location.url || undefined,
  };
}

function pageErrorOf(e: Event<"event">): TraceError | undefined {
  if (e.method !== "pageError") return undefined;
  const { error } = e.params;
  return {
    message: error.error.message ?? error.value.s ?? "Page error",
    stack: error.error.stack,
    time: e.time ?? 0,
  };
}

function frameOf(
  e: Event<"screencast-frame">,
  frameSrc: (file: string) => string | undefined,
): TraceFrame | undefined {
  const file = e.file ?? (e.sha1 ? `resources/${e.sha1}` : undefined);
  const src = file ? frameSrc(file) : undefined;
  if (!src) return undefined;
  return { src, time: e.timestamp ?? 0, width: e.width ?? 0, height: e.height ?? 0 };
}

type TraceEvents = {
  calls: Map<string, Call>;
  frames: TraceFrame[];
  console: TraceConsole[];
  errors: TraceError[];
  viewport: TraceModel["viewport"];
};

function readEvents(
  traceText: string,
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
  for (const e of parseNdjson(traceText)) {
    const action = events.calls.get(isString(e.callId) ? e.callId : "")?.action;
    switch (e.type) {
      case "context-options":
        events.viewport = viewportOf(EVENTS["context-options"].parse(e)) ?? events.viewport;
        break;
      case "before": {
        const call = beginCall(EVENTS.before.parse(e), nextGroup);
        if (call) events.calls.set(call.action.id, call);
        break;
      }
      case "log": {
        const { message } = EVENTS.log.parse(e);
        if (action && message !== undefined) addLog(action, message);
        break;
      }
      case "input":
        if (action) addInput(action, EVENTS.input.parse(e));
        break;
      case "after":
        if (!action) break;
        endAction(action, EVENTS.after.parse(e));
        if (action.error)
          events.errors.push({ message: action.error, time: action.end, actionId: action.id });
        break;
      case "console":
        keep(events.console, consoleOf(EVENTS.console.parse(e)));
        break;
      case "event":
        keep(events.errors, pageErrorOf(EVENTS.event.parse(e)));
        break;
      case "screencast-frame":
        keep(events.frames, frameOf(EVENTS["screencast-frame"].parse(e), frameSrc));
        break;
    }
  }
  return events;
}

const resourceSnapshot = z.object({
  snapshot: lenient({
    request: lenient({ method: text, url: text }),
    response: lenient({
      status: count,
      bodySize: count,
      _transferSize: count,
      _failureText: text,
      content: lenient({ size: count, mimeType: text }),
    }),
    _resourceType: text,
    _monotonicTime: count,
    time: count,
  }),
});

function requestOf(e: z.infer<typeof resourceSnapshot>): TraceRequest {
  const { request, response } = e.snapshot;
  const size = response.content.size ?? -1;
  return {
    method: request.method ?? "GET",
    url: request.url ?? "",
    status: response.status ?? -1,
    mimeType: response.content.mimeType ?? "",
    resourceType: e.snapshot["_resourceType"] ?? "",
    size: size >= 0 ? size : (response["_transferSize"] ?? response.bodySize ?? -1),
    start: e.snapshot["_monotonicTime"] ?? 0,
    duration: e.snapshot.time ?? 0,
    failure: response["_failureText"],
  };
}

function parseNetwork(networkText: string): TraceRequest[] {
  return parseNdjson(networkText)
    .filter((e) => e.type === "resource-snapshot")
    .map((e) => requestOf(resourceSnapshot.parse(e)))
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
  if (!times.length) return { start: 0, end: 0 };
  let start = Infinity;
  let end = -Infinity;
  for (const t of times) {
    start = Math.min(start, t);
    end = Math.max(end, t);
  }
  return { start, end };
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
