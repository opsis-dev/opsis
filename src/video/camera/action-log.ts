import type { Action, ActionLog } from "../../shared/schema/actions";
import type { AnnotationSpec } from "../../shared/compile/recording";
import { POINTER_KINDS } from "../../shared/compile/recording";
import { chordGlyphs, type ClickEvent, type ClickLog, type KeyEvent } from "./lib/click-log";
import type { Annotation } from "./annotations";

export function toClickLog(log: ActionLog, name: string, extraBeats: Action[] = []): ClickLog {
  const actions = [...log.actions, ...extraBeats].sort((a, b) => a.t - b.t);
  const clicks: ClickEvent[] = [];
  const keys: KeyEvent[] = [];
  for (const a of actions) {
    if (a.kind === "key" && a.text) {
      keys.push({ tMs: a.t, chord: a.text, label: a.label });
      continue;
    }
    if (!POINTER_KINDS.has(a.kind) || !a.point) continue;
    if (a.kind === "drag") {
      clicks.push(...dragEvents(a, log.viewport));
      continue;
    }
    const c: ClickEvent = { tMs: a.t, x: a.point.x, y: a.point.y, label: a.label };
    if (a.tDepart != null) c.tDepartMs = a.tDepart;
    if (a.tDown != null) c.tDownMs = a.tDown;
    if (a.kind === "type" && a.tEnd != null) c.typeEndMs = a.tEnd;
    if (a.rect) c.rect = a.rect;
    if (a.cluster) c.cluster = a.cluster;
    if (a.zoom === false) c.zoom = false;
    if (a.zoomScale) c.zoomScale = a.zoomScale;
    clicks.push(c);
  }
  return {
    name,
    viewport: log.viewport,
    durationMs: log.durationMs,
    cursorTrack: log.cursorTrack,
    clicks,
    keys,
    offsetMs: 0,
  };
}

export const DRAG_FIT_FRACTION = 0.5;

type Rect = { x: number; y: number; w: number; h: number };

const union = (a: Rect, b: Rect): Rect => {
  const x = Math.min(a.x, b.x);
  const y = Math.min(a.y, b.y);
  return {
    x,
    y,
    w: Math.max(a.x + a.w, b.x + b.w) - x,
    h: Math.max(a.y + a.h, b.y + b.h) - y,
  };
};

function dragEvents(a: Action, vp: { width: number; height: number }): ClickEvent[] {
  const point = a.point!;
  const tDown = a.tDown ?? a.t;
  const shared: Partial<ClickEvent> = {
    label: a.label,
    ...(a.cluster ? { cluster: a.cluster } : {}),
    ...(a.zoom === false ? { zoom: false } : {}),
    ...(a.zoomScale ? { zoomScale: a.zoomScale } : {}),
    ...(a.path ? { path: a.path } : {}),
  };
  const pick = a.rect ?? { x: point.x - 60, y: point.y - 24, w: 120, h: 48 };
  const drop = a.drop ?? { x: point.x - 60, y: point.y - 24, w: 120, h: 48 };
  const both = union(pick, drop);
  if (both.w <= vp.width * DRAG_FIT_FRACTION && both.h <= vp.height * DRAG_FIT_FRACTION) {
    return [
      {
        ...shared,
        tMs: a.t,
        ...(a.tDepart != null ? { tDepartMs: a.tDepart } : {}),
        tDownMs: tDown,
        tUpMs: a.t,
        x: both.x + both.w / 2,
        y: both.y + both.h / 2,
        rect: both,
        drop,
      },
    ];
  }
  return [
    {
      ...shared,
      tMs: tDown,
      ...(a.tDepart != null ? { tDepartMs: a.tDepart } : {}),
      tDownMs: tDown,
      tUpMs: a.t,
      x: pick.x + pick.w / 2,
      y: pick.y + pick.h / 2,
      rect: pick,
    },
    {
      ...shared,
      tMs: a.t,
      tDepartMs: tDown,
      x: point.x,
      y: point.y,
      rect: drop,
      drop,
    },
  ];
}

export function resolveAnnotations(log: ActionLog, specs: AnnotationSpec[]): Annotation[] {
  const out: Annotation[] = [];
  for (const spec of specs) {
    const needle = spec.match.trim().toLowerCase();
    for (const a of log.actions) {
      if (!POINTER_KINDS.has(a.kind) || !a.rect) continue;
      if (needle && !(a.label ?? "").toLowerCase().includes(needle)) continue;
      const startMs = (a.tDepart ?? a.t) - spec.leadMs;
      const endMs = (a.tEnd ?? a.t) + spec.holdMs;
      out.push({
        kind: spec.kind,
        rect: a.rect,
        fromS: Math.max(0, startMs) / 1000,
        toS: endMs / 1000,
        text: spec.text,
        color: spec.color,
        padding: spec.padding,
      });
    }
  }
  return out;
}

export function typedBeats(log: ActionLog): Array<{ text: string; fromS: number; toS: number }> {
  return log.actions
    .filter((a) => a.kind === "type" && a.text)
    .map((a) => ({ text: a.text!, fromS: a.t / 1000, toS: (a.tEnd ?? a.t + 1000) / 1000 + 0.6 }));
}

export { chordGlyphs };
