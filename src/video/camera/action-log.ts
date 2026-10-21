import { POINTER_KINDS } from "../../shared/compile/recording";
import {
  pointRect,
  rectCenter,
  rectUnion,
  type Action,
  type ActionLog,
  type Point,
  type Rect,
  type Viewport,
} from "../../shared/schema/actions";
import type { AnnotationKind, AnnotationSpec, HighlightLook } from "../../shared/schema/recording";
import type { ClickEvent, ClickLog, KeyEvent } from "./lib/click-log";
import { designScaleFor } from "./lib/design-width";

export type Annotation = {
  id: string;
  kind: AnnotationKind;
  rect: Rect;
  fromS: number;
  toS: number;
  text: string;
  color: string;
  padding: number;
  look: HighlightLook;
  roughness: number;
  seed: number;
  label: Pick<AnnotationSpec, "textColor" | "fontSize" | "font">;
};

const DEFAULT_ROUGHNESS = 1;

export const TYPED_LINGER_S = 0.6;
const TYPED_FALLBACK_MS = 1000;
const DRAG_FIT_FRACTION = 0.5;

const carried = (a: Action) => ({
  label: a.label,
  cluster: a.cluster || undefined,
  zoom: a.zoom,
  zoomScale: a.zoomScale,
});

export function toClickLog(
  log: ActionLog,
  name: string,
  extraBeats: Action[] = [],
  { zoomActions = true }: { zoomActions?: boolean } = {},
): ClickLog {
  const taken = zoomActions ? log.actions : log.actions.map((a) => ({ ...a, zoom: false }));
  const actions = [...taken, ...extraBeats].sort((a, b) => a.t - b.t);
  const clicks: ClickEvent[] = [];
  const keys: KeyEvent[] = [];
  for (const a of actions) {
    if (a.kind === "key" && a.text) {
      keys.push({ tMs: a.t, chord: a.text, label: a.label });
      continue;
    }
    if (!POINTER_KINDS.has(a.kind) || !a.point) continue;
    if (a.kind === "drag") {
      clicks.push(...dragEvents(a, a.point, log.viewport));
      continue;
    }
    clicks.push({
      ...carried(a),
      tMs: a.t,
      x: a.point.x,
      y: a.point.y,
      tDepartMs: a.tDepart,
      tDownMs: a.tDown,
      typeEndMs: a.kind === "type" ? a.tEnd : undefined,
      rect: a.rect,
    });
  }
  return {
    name,
    viewport: log.viewport,
    durationMs: log.durationMs,
    cursorTrack: log.cursorTrack,
    clicks,
    keys,
  };
}

function dragEvents(a: Action, point: Point, vp: Viewport): ClickEvent[] {
  const tDown = a.tDown ?? a.t;
  const fallback = pointRect(point, designScaleFor(vp.width));
  const pick = a.rect ?? fallback;
  const drop = a.drop ?? fallback;
  const both = rectUnion(pick, drop);
  const shared = { ...carried(a), path: a.path };
  const press = { ...shared, tDepartMs: a.tDepart, tDownMs: tDown, tUpMs: a.t };
  if (both.w <= vp.width * DRAG_FIT_FRACTION && both.h <= vp.height * DRAG_FIT_FRACTION)
    return [{ ...press, tMs: a.t, ...rectCenter(both), rect: both, drop }];
  return [
    { ...press, tMs: tDown, ...rectCenter(pick), rect: pick },
    { ...shared, tMs: a.t, tDepartMs: tDown, ...point, rect: drop, drop },
  ];
}

export function resolveAnnotations(log: ActionLog, specs: AnnotationSpec[]): Annotation[] {
  const out: Annotation[] = [];
  for (const [specIndex, spec] of specs.entries()) {
    const needle = spec.match.trim().toLowerCase();
    for (const a of log.actions) {
      if (!POINTER_KINDS.has(a.kind) || !a.rect) continue;
      if (needle && !(a.label ?? "").toLowerCase().includes(needle)) continue;
      const startMs = (a.tDepart ?? a.t) - spec.leadMs;
      const endMs = (a.tEnd ?? a.t) + spec.holdMs;
      out.push({
        id: `${specIndex}:${a.id}`,
        kind: spec.kind,
        rect: a.rect,
        fromS: Math.max(0, startMs) / 1000,
        toS: endMs / 1000,
        text: spec.text,
        color: spec.color,
        padding: spec.padding,
        look: spec.look ?? "clean",
        roughness: spec.roughness ?? DEFAULT_ROUGHNESS,
        seed: out.length + 1,
        label: { textColor: spec.textColor, fontSize: spec.fontSize, font: spec.font },
      });
    }
  }
  return out;
}

const isTyped = (a: Action): a is Action & { text: string } => a.kind === "type" && !!a.text;

export const typedBeats = (log: ActionLog): Array<{ text: string; fromS: number; toS: number }> =>
  log.actions.filter(isTyped).map((a) => ({
    text: a.text,
    fromS: a.t / 1000,
    toS: (a.tEnd ?? a.t + TYPED_FALLBACK_MS) / 1000 + TYPED_LINGER_S,
  }));
