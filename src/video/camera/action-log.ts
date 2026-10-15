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
