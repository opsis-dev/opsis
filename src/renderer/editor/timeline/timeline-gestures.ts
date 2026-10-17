import { round3, clamp } from "@/shared/math";
import { MAX_VOLUME } from "@/shared/nodes/params";
import { setAudioVolume } from "@/shared/timeline/edits";
import type { FlatTimeline } from "@/shared/timeline/flatten";
import { snapFrame, type SnapTarget } from "@/shared/timeline/snap";
import { framesToMs, msToExactFrames, wholeFrames } from "@/shared/timeline/time";
import { DRAG_START_PX } from "@/renderer/lib/dom";
import { usePlayback } from "../../store/playback";
import { useUiStore } from "../../store/ui";
import { useWorkflowStore } from "../../store/workflow";
import { ENVELOPE_INFO, envelopeShift, setEnvelopeKeys } from "./envelope";
import { scrubAt } from "./scrub";
import { applyEdit, patchMarker, reportEdit, selectNodes } from "./timeline-actions";
import {
  editContext,
  editOutcome,
  isClick,
  marqueeIds,
  normalizeMarquee,
  type EditDrag,
  type KeyDrag,
  type MarkerDrag,
  type MarqueeBox,
  type MarqueeDrag,
  type VolumeDrag,
} from "./timeline-drag";
import { SNAP_PX, frameAtX, type LaneLayout } from "./timeline-layout";
import { useTimeline } from "./timeline-state";

export type ActiveDrag = { move: (e: React.PointerEvent) => void; finish?: () => void };

export type GestureEnv = {
  flat: FlatTimeline;
  layout: LaneLayout;
  ppf: number;
  sourceNodeId: string | null;
  masterSrc: string | null;
  xToFrame: (clientX: number) => number;
  point: (clientX: number, clientY: number) => { x: number; y: number };
  snapOn: (alt: boolean) => boolean;
  snapTargets: (exclude: string | null) => SnapTarget[];
};

type Gesture = { started: boolean };

function startGestureIfMoved(d: Gesture, distance: number): boolean {
  if (d.started) return true;
  if (distance < DRAG_START_PX) return false;
  d.started = true;
  useWorkflowStore.getState().beginGesture();
  return true;
}

function endGesture(d: Gesture): boolean {
  if (d.started) useWorkflowStore.getState().endGesture();
  return d.started;
}

export const editGesture = (
  env: GestureEnv,
  d: EditDrag,
  showGuide: (frame: number | null) => void,
): ActiveDrag => ({
  move: (e) => {
    if (!startGestureIfMoved(d, Math.abs(e.clientX - d.x0))) return;
    const ctx = editContext(d.base, env.flat.fps, env.sourceNodeId, d.salt);
    const snap = {
      raw: (e.clientX - d.x0) / env.ppf,
      threshold: env.snapOn(e.altKey) ? SNAP_PX / env.ppf : 0,
    };
    const { result, snappedAt } = editOutcome(ctx, d, snap, e.shiftKey);
    showGuide(snappedAt);
    d.last = result;
    applyEdit(result);
  },
  finish: () => {
    if (endGesture(d)) reportEdit(d.last);
  },
});

export const volumeGesture = (env: GestureEnv, d: VolumeDrag): ActiveDrag => ({
  move: (e) => {
    if (!startGestureIfMoved(d, Math.abs(e.clientY - d.y0))) return;
    const v = d.v0 - ((e.clientY - d.y0) / d.height) * MAX_VOLUME;
    applyEdit(setAudioVolume(editContext(d.base, env.flat.fps, env.sourceNodeId, "v"), d.bar, v));
  },
  finish: () => endGesture(d),
});

export const keyGesture = (env: GestureEnv, d: KeyDrag): ActiveDrag => ({
  move: (e) => {
    const distance = Math.abs(e.clientX - d.x0) + Math.abs(e.clientY - d.y0);
    if (!startGestureIfMoved(d, distance)) return;
    const { min, max } = ENVELOPE_INFO[d.prop];
    const frame = wholeFrames(d.toLocal(env.xToFrame(e.clientX)));
    const dragged = envelopeShift(d.prop, e.clientY - d.y0, d.height);
    const value = round3(clamp(d.keys[d.index].value - dragged, min, max));
    setEnvelopeKeys(
      d.nodeId,
      d.prop,
      d.keys.map((k, i) => (i === d.index ? { ...k, frame, value } : k)),
    );
  },
  finish: () => {
    if (!endGesture(d)) selectNodes([d.nodeId], false, env.sourceNodeId);
  },
});

export const markerGesture = (env: GestureEnv, d: MarkerDrag): ActiveDrag => ({
  move: (e) => {
    if (!startGestureIfMoved(d, Math.abs(e.clientX - d.x0))) return;
    const { fps } = env.flat;
    let f = msToExactFrames(d.at0, fps) + (e.clientX - d.x0) / env.ppf;
    if (env.snapOn(e.altKey)) {
      const targets = env.snapTargets(null).filter((t) => t.kind !== "marker");
      f = snapFrame(f, targets, SNAP_PX / env.ppf).frame;
    }
    patchMarker(d.id, { atMs: Math.max(0, framesToMs(f, fps)) });
  },
  finish: () => {
    if (!endGesture(d)) useTimeline.getState().selectMarker(d.id);
  },
});

export function scrubTo(env: GestureEnv, frame: number): void {
  usePlayback.getState().seek(frame);
  scrubAt(env.masterSrc, frame / env.flat.fps);
}

export const scrubGesture = (env: GestureEnv): ActiveDrag => ({
  move: (e) =>
    scrubTo(env, clamp(env.xToFrame(e.clientX), 0, Math.max(0, env.flat.durationInFrames - 1))),
});

export const marqueeGesture = (
  env: GestureEnv,
  d: MarqueeDrag,
  showBox: (box: MarqueeBox | null) => void,
): ActiveDrag => ({
  move: (e) => {
    const { x, y } = env.point(e.clientX, e.clientY);
    d.x1 = x;
    d.y1 = y;
    showBox({ x0: d.x0, y0: d.y0, x1: x, y1: y });
  },
  finish: () => {
    showBox(null);
    const box = normalizeMarquee(d);
    if (!isClick(box)) {
      selectNodes(marqueeIds(env.flat, box, env.layout, env.ppf), d.additive, env.sourceNodeId);
      return;
    }
    if (!d.additive) useUiStore.getState().selectPinned([], env.sourceNodeId);
    usePlayback.getState().seek(Math.max(0, frameAtX(d.x0, env.ppf)));
    useTimeline.getState().selectMarker(null);
  },
});
