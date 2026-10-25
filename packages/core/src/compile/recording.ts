import { DEFAULT_BACKDROP } from "#core/backdrops";
import type { Action, ActionLog } from "#core/schema/actions";
import type { ArrowCursorLook, RecordingProps, RippleLook } from "#core/schema/recording";

export const DEFAULT_CURSOR_LOOK: ArrowCursorLook = {
  shape: "arrow",
  fill: "#111418",
  edge: "#ffffff",
};

export const DEFAULT_RIPPLE: RippleLook = { color: "#0f172a", size: 1, width: 2 };

export const DEFAULT_KEYCAP = { keycap: "#0a0a0a", ink: "#ffffff" } as const;

export const PREVIEW_FILE_WIDTH = 960;

export const recordingSpeed = (props: Pick<RecordingProps, "speed">): number => props.speed || 1;

export const defaultRecordingProps = (): Omit<
  RecordingProps,
  "src" | "proxySrc" | "log" | "name"
> => ({
  speed: 1.25,
  zoom: true,
  tuning: {},
  motionBlur: true,
  style: "classic",
  chrome: false,
  backdrop: DEFAULT_BACKDROP,
  cursor: true,
  cursorScale: 1,
  cursorLook: DEFAULT_CURSOR_LOOK,
  ripple: DEFAULT_RIPPLE,
  extraBeats: [],
  annotations: [],
});

export const POINTER_KINDS = new Set<Action["kind"]>(["click", "type", "hover", "focus", "drag"]);

export function idleSpans(
  log: ActionLog,
  thresholdMs: number,
  padMs = 400,
): Array<[number, number]> {
  const beats = [...log.actions].sort((a, b) => a.t - b.t);
  const spans: Array<[number, number]> = [];
  let cursor = 0;
  for (const a of beats) {
    const start = (a.tDepart ?? a.t) - padMs;
    if (start - cursor >= thresholdMs) spans.push([cursor, start]);
    cursor = Math.max(cursor, (a.tEnd ?? a.t) + padMs);
  }
  const end = log.durationMs - padMs;
  if (end - cursor >= thresholdMs) spans.push([cursor, end]);
  return spans;
}

export function followCursorBeats(
  log: ActionLog,
  scale: number,
  everyMs: number,
  nodeId: string,
): Action[] {
  const track = log.cursorTrack ?? [];
  const beats: Action[] = [];
  let next = 0;
  for (const s of track) {
    if (s.t < next) continue;
    beats.push({
      id: `${nodeId}:f${beats.length}`,
      kind: "focus",
      t: s.t,
      point: { x: s.x, y: s.y },
      cluster: `${nodeId}:follow`,
      zoomScale: scale,
      nodeId,
    });
    next = s.t + everyMs;
  }
  return beats;
}
