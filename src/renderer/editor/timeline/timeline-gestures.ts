import { endIfStarted, startOnceMoved, type ThresholdDrag } from "@/renderer/lib/dom";
import { randomId } from "@/shared/ids";
import { clamp, round3 } from "@/shared/math";
import type { Marker, Workflow } from "@/shared/schema/workflow";
import { moveCameraKeyMark, setAudioVolume } from "@/shared/timeline/edits";
import type { AudioBar, CameraKeyMark, FlatTimeline } from "@/shared/timeline/flatten";
import { snapFrame, type SnapTarget } from "@/shared/timeline/snap";
import { clampFrame, framesToMs, wholeFrames } from "@/shared/timeline/time";
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
  type EditTarget,
  type KeyDrag,
  type KeyStart,
  type MarkerDrag,
  type MarqueeBox,
  type VolumeDrag,
} from "./timeline-drag";
import { SNAP_PX, frameAtX, markerFrame, type LaneLayout } from "./timeline-layout";
import { useTimeline } from "./timeline-state";

export type ActiveDrag = { move: (e: React.PointerEvent) => void; finish?: () => void };

export type GestureEnv = {
  workflow: Workflow;
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

const startEditOnceMoved = (d: ThresholdDrag, distance: number): boolean =>
  startOnceMoved(d, distance, () => useWorkflowStore.getState().beginGesture());

const endEdit = (d: ThresholdDrag): boolean =>
  endIfStarted(d, () => useWorkflowStore.getState().endGesture());

export function editGesture(
  env: GestureEnv,
  e: React.PointerEvent,
  target: EditTarget,
  ripple: boolean,
  showGuide: (frame: number | null) => void,
): ActiveDrag {
  const d: EditDrag = {
    ...target,
    x0: e.clientX,
    base: env.workflow,
    flat: env.flat,
    targets: env.snapTargets(target.bar.key),
    salt: randomId(),
    started: false,
    last: null,
    ripple,
  };
  return {
    move: (ev) => {
      if (!startEditOnceMoved(d, Math.abs(ev.clientX - d.x0))) return;
      const ctx = editContext(d.base, env.flat.fps, env.sourceNodeId, d.salt);
      const snap = {
        raw: (ev.clientX - d.x0) / env.ppf,
        threshold: env.snapOn(ev.altKey) ? SNAP_PX / env.ppf : 0,
      };
      const { result, snappedAt } = editOutcome(ctx, d, snap, ev.shiftKey);
      showGuide(snappedAt);
      d.last = result;
      applyEdit(result);
    },
    finish: () => {
      if (endEdit(d) && d.last) reportEdit(d.last);
    },
  };
}

export function volumeGesture(
  env: GestureEnv,
  e: React.PointerEvent,
  bar: AudioBar,
  height: number,
): ActiveDrag {
  const d: VolumeDrag = {
    bar,
    y0: e.clientY,
    v0: bar.volume,
    base: env.workflow,
    salt: randomId(),
    started: false,
    height,
  };
  return {
    move: (ev) => {
      if (!startEditOnceMoved(d, Math.abs(ev.clientY - d.y0))) return;
      const v = d.v0 - envelopeShift("volume", ev.clientY - d.y0, d.height);
      const ctx = editContext(d.base, env.flat.fps, env.sourceNodeId, d.salt);
      applyEdit(setAudioVolume(ctx, d.bar, v));
    },
    finish: () => endEdit(d),
  };
}

export function keyGesture(env: GestureEnv, e: React.PointerEvent, key: KeyStart): ActiveDrag {
  const d: KeyDrag = { ...key, x0: e.clientX, y0: e.clientY, started: false };
  return {
    move: (ev) => {
      const distance = Math.abs(ev.clientX - d.x0) + Math.abs(ev.clientY - d.y0);
      if (!startEditOnceMoved(d, distance)) return;
      const { min, max } = ENVELOPE_INFO[d.prop];
      const frame = wholeFrames(d.toLocal(env.xToFrame(ev.clientX)));
      const dragged = envelopeShift(d.prop, ev.clientY - d.y0, d.height) / (d.scale || 1);
      const value = round3(clamp(d.keys[d.index].value - dragged, min, max));
      setEnvelopeKeys(
        d.nodeId,
        d.prop,
        d.keys.map((k, i) => (i === d.index ? { ...k, frame, value } : k)),
      );
    },
    finish: () => {
      if (!endEdit(d)) selectNodes([d.nodeId], false, env.sourceNodeId);
    },
  };
}

export function cameraKeyGesture(
  env: GestureEnv,
  e: React.PointerEvent,
  mark: CameraKeyMark,
): ActiveDrag {
  const d = { x0: e.clientX, base: env.workflow, salt: randomId(), started: false };
  return {
    move: (ev) => {
      if (!startEditOnceMoved(d, Math.abs(ev.clientX - d.x0))) return;
      const ctx = editContext(d.base, env.flat.fps, env.sourceNodeId, d.salt);
      applyEdit(moveCameraKeyMark(ctx, mark, Math.round((ev.clientX - d.x0) / env.ppf)));
    },
    finish: () => {
      if (!endEdit(d)) selectNodes([mark.nodeId], false, env.sourceNodeId);
    },
  };
}

export function markerGesture(env: GestureEnv, e: React.PointerEvent, marker: Marker): ActiveDrag {
  const { fps } = env.flat;
  const d: MarkerDrag = {
    id: marker.id,
    x0: e.clientX,
    from: markerFrame(marker, fps),
    started: false,
  };
  return {
    move: (ev) => {
      if (!startEditOnceMoved(d, Math.abs(ev.clientX - d.x0))) return;
      let f = d.from + (ev.clientX - d.x0) / env.ppf;
      if (env.snapOn(ev.altKey)) {
        const targets = env.snapTargets(null).filter((t) => t.kind !== "marker");
        f = snapFrame(f, targets, SNAP_PX / env.ppf).frame;
      }
      patchMarker(d.id, { atMs: Math.max(0, framesToMs(f, fps)) });
    },
    finish: () => {
      if (!endEdit(d)) useTimeline.getState().selectMarker(d.id);
    },
  };
}

const timelineFrame = (env: GestureEnv, frame: number): number =>
  clampFrame(frame, env.flat.durationInFrames);

export function scrubTo(env: GestureEnv, frame: number): void {
  const at = timelineFrame(env, frame);
  usePlayback.getState().seek(at);
  scrubAt(env.masterSrc, at / env.flat.fps);
}

export const scrubGesture = (env: GestureEnv): ActiveDrag => ({
  move: (e) => scrubTo(env, env.xToFrame(e.clientX)),
});

export function marqueeGesture(
  env: GestureEnv,
  start: { x: number; y: number },
  additive: boolean,
  showBox: (box: MarqueeBox | null) => void,
): ActiveDrag {
  const d: MarqueeBox = { x0: start.x, y0: start.y, x1: start.x, y1: start.y };
  return {
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
        selectNodes(marqueeIds(env.flat, box, env.layout, env.ppf), additive, env.sourceNodeId);
        return;
      }
      if (!additive) useUiStore.getState().selectPinned([], env.sourceNodeId);
      usePlayback.getState().seek(timelineFrame(env, frameAtX(d.x0, env.ppf)));
      useTimeline.getState().selectMarker(null);
    },
  };
}
