import type { Action, ActionLog } from "../schema/actions";
import type { AssetRef } from "../schema/values";
import { DEFAULT_BACKDROP } from "../backdrops";

export const CAMERA_STYLES = ["classic", "proof", "ledger", "stage"] as const;
type CameraStyle = (typeof CAMERA_STYLES)[number];

export type AnnotationKind = "highlight" | "spotlight" | "tooltip" | "callout";

export type AnnotationSpec = {
  kind: AnnotationKind;
  match: string;
  text: string;
  color: string;
  padding: number;
  leadMs: number;
  holdMs: number;
};

export type RecordingProps = {
  src: AssetRef;
  proxySrc?: AssetRef;
  log: ActionLog;
  name: string;
  speed: number;
  zoom: boolean;
  tuning: { sMax?: number; holdMinS?: number };
  motionBlur: boolean;
  style: CameraStyle;
  chrome: boolean;
  windowStyle?: "macos" | "windows" | "linux";
  frameTheme?: "light" | "dark";
  addressBar?: boolean;
  backdrop: string;
  cursor: boolean;
  cursorScale: number;
  ripple: boolean;
  extraBeats: Action[];
  keystrokes?: { position: "bottom" | "top"; typed: boolean; chords: boolean };
  annotations: AnnotationSpec[];
};

export const DEFAULT_RECORDING_PROPS: Omit<RecordingProps, "src" | "proxySrc" | "log" | "name"> = {
  speed: 1.25,
  zoom: true,
  tuning: {},
  motionBlur: true,
  style: "classic",
  chrome: false,
  backdrop: DEFAULT_BACKDROP,
  cursor: true,
  cursorScale: 1,
  ripple: true,
  extraBeats: [],
  annotations: [],
};

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
