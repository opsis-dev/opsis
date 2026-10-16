import {
  useCallback,
  useEffect,
  useImperativeHandle,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { toast } from "sonner";
import { duckGain, duckingSpans } from "@/shared/compile/duck";
import { applyMixLevels } from "@/shared/mix/levels";
import { getNodeDef } from "@/shared/nodes/registry";
import type { AudioEnvelopeKey, Keyframe, VisualEnvelopeKey } from "@/shared/schema/timeline";
import type { Workflow } from "@/shared/schema/workflow";
import {
  applyRemoval,
  applyRipple,
  containerOf,
  moveAudio,
  moveBar,
  moveHandleKey,
  removalFor,
  rippleTargets,
  rollCut,
  setAudioVolume,
  slipBar,
  speedBar,
  splitBar,
  trimAudio,
  trimBar,
  type EditContext,
  type EditResult,
  type Removal,
} from "@/shared/timeline/edits";
import { envelopeAt } from "@/shared/timeline/envelope";
import type { AudioBar, FlatTimeline, TransitionMark, VideoBar } from "@/shared/timeline/flatten";
import {
  formatTime,
  rulerStep,
  snapFrame,
  snapSpan,
  type SnapTarget,
} from "@/shared/timeline/snap";
import { transitionTitle } from "@/shared/nodes/transitions";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/renderer/components/ui/alert-dialog";
import { Input } from "@/renderer/components/ui/input";
import { cn } from "@/renderer/lib/utils";
import { useMedia } from "../../store/media";
import { usePlayback } from "../../store/playback";
import { useUiStore } from "../../store/ui";
import { newId, useWorkflowStore } from "../../store/workflow";
import { useFilm } from "../film";
import { FilmstripCanvas, WaveformCanvas, type Viewport } from "./bar-canvas";
import { preloadScrub, scrubAt } from "./scrub";
import { MAX_PX_PER_FRAME, useTimeline, type TimelineTool } from "./timeline-state";

const HEADER_W = 84;
const RULER_H = 24;
const MARKERS_H = 20;
const VIDEO_H = 46;
const AUDIO_H = 46;
const EDGE_PX = 6;
const SNAP_PX = 8;
const TAIL_PX = 240;

const ENVELOPE_RANGE: Record<VisualEnvelopeKey | AudioEnvelopeKey, [number, number, number]> = {
  opacity: [0, 1, 1],
  x: [-1, 1, 0],
  y: [-1, 1, 0],
  scale: [0, 3, 1],
  volume: [0, 2, 1],
  pan: [-1, 1, 0],
};

const CATEGORY_TONE: Record<string, string> = {
  Capture: "bg-track-video",
  Camera: "bg-track-video",
  Cursor: "bg-track-video",
  "Text & overlays": "bg-track-text",
  Elements: "bg-track-graphics",
  Backgrounds: "bg-track-graphics",
  Effects: "bg-track-graphics",
  Transitions: "bg-track-graphics",
  Media: "bg-track-media",
};

const AUDIO_TONE: Record<AudioBar["role"], string> = {
  music: "bg-track-music",
  effect: "bg-track-effect",
  clip: "bg-track-clip",
};

type EditDragBase = {
  x0: number;
  base: Workflow;
  flat: FlatTimeline;
  targets: SnapTarget[];
  salt: string;
  started: boolean;
  last: EditResult | null;
  ripple: boolean;
};

type VideoEditDrag = EditDragBase & {
  kind: "move" | "trim-start" | "trim-end" | "slip" | "speed" | "roll-start" | "roll-end";
  bar: VideoBar;
};

type AudioEditDrag = EditDragBase & {
  kind: "audio-move" | "audio-trim-start" | "audio-trim-end";
  bar: AudioBar;
};

type EditDrag = VideoEditDrag | AudioEditDrag;

type Drag =
  | EditDrag
  | {
      kind: "volume";
      bar: AudioBar;
      y0: number;
      v0: number;
      base: Workflow;
      started: boolean;
      height: number;
    }
  | {
      kind: "key";
      nodeId: string;
      prop: VisualEnvelopeKey | AudioEnvelopeKey;
      index: number;
      keys: Keyframe[];
      toLocal: (abs: number) => number;
      x0: number;
      y0: number;
      height: number;
      started: boolean;
    }
  | { kind: "marker"; id: string; x0: number; at0: number; started: boolean }
  | { kind: "scrub" }
  | { kind: "marquee"; x0: number; y0: number; x1: number; y1: number; additive: boolean };

function transitionTip(wf: Workflow, t: TransitionMark): string {
  const edge = t.slot
    ? wf.edges.find((e) => e.target === t.slot!.nodeId && e.targetHandle === t.slot!.port)
    : undefined;
  const into = edge ? ` into ${nodeTitle(wf, edge.source)}` : "";
  const cover =
    t.covers > 0.4
      ? `. It covers ${Math.min(100, Math.round(t.covers * 100))}% of a clip: turn on Keep every clip whole, or shorten it`
      : "";
  return `${transitionTitle(t.kind)}${into}${cover}.`;
}

function nodeTitle(wf: Workflow, id: string): string {
  const node = wf.nodes.find((n) => n.id === id);
  if (!node) return "";
  return node.label?.trim() || getNodeDef(node.type)?.title || node.type;
}

function toneFor(wf: Workflow, bar: VideoBar): string {
  const type = wf.nodes.find((n) => n.id === bar.nodeId)?.type ?? "";
  return CATEGORY_TONE[getNodeDef(type)?.category ?? ""] ?? "bg-track-video";
}

const supportsEnvelope = (wf: Workflow, nodeId: string) => {
  const type = wf.nodes.find((n) => n.id === nodeId)?.type ?? "";
  return !!getNodeDef(type)?.params.shape.envelope;
};

const envelopeOf = (wf: Workflow, nodeId: string): Record<string, Keyframe[] | undefined> =>
  (wf.nodes.find((n) => n.id === nodeId)?.params.envelope as Record<string, Keyframe[]>) ?? {};

function setEnvelopeKeys(nodeId: string, prop: string, keys: Keyframe[]): void {
  useWorkflowStore.getState().update((wf) => ({
    ...wf,
    nodes: wf.nodes.map((n) => {
      if (n.id !== nodeId) return n;
      const env = { ...(n.params.envelope as Record<string, Keyframe[]> | undefined) };
      if (keys.length) env[prop] = keys.toSorted((a, b) => a.frame - b.frame);
      else delete env[prop];
      return { ...n, params: { ...n.params, envelope: env } };
    }),
  }));
}

export type TimelineActions = {
  split: () => void;
  remove: () => void;
  addMarker: () => void;
  zoomBy: (factor: number) => void;
  fit: () => void;
};

const NO_MARKERS: NonNullable<Workflow["markers"]> = [];

let lastDown = { at: 0, x: 0, y: 0 };

const clickCounts = new WeakMap<Event, boolean>();

function secondClick(e: React.PointerEvent): boolean {
  const seen = clickCounts.get(e.nativeEvent);
  if (seen !== undefined) return seen;
  const now = performance.now();
  const hit =
    now - lastDown.at < 400 &&
    Math.abs(e.clientX - lastDown.x) < 5 &&
    Math.abs(e.clientY - lastDown.y) < 5;
  lastDown = hit ? { at: 0, x: 0, y: 0 } : { at: now, x: e.clientX, y: e.clientY };
  clickCounts.set(e.nativeEvent, hit);
  return hit;
}

let gestureCounter = 0;
const gestureSalt = () => (++gestureCounter).toString(36);

export function TimelineView({ ref }: { ref: React.Ref<TimelineActions> }) {
  const film = useFilm();
  const flat = film.flat;
  const workflow = useWorkflowStore((s) => s.workflow)!;
  const selected = useUiStore((s) => s.selectedIds);
  const frame = usePlayback((s) => s.frame);
  const playing = usePlayback((s) => s.playing);
  const inPoint = usePlayback((s) => s.inPoint);
  const outPoint = usePlayback((s) => s.outPoint);
  const ppf = useTimeline((s) => s.pxPerFrame);
  const fit = useTimeline((s) => s.fit);
  const tool = useTimeline((s) => s.tool);
  const snapping = useTimeline((s) => s.snapping);
  const rulerMode = useTimeline((s) => s.rulerMode);
  const videoEnvelope = useTimeline((s) => s.videoEnvelope);
  const audioEnvelope = useTimeline((s) => s.audioEnvelope);
  const selectedMarker = useTimeline((s) => s.selectedMarker);
  const scroller = useRef<HTMLDivElement>(null);
  const drag = useRef<Drag | null>(null);
  const [view, setView] = useState({ left: 0, width: 800, height: 300 });
  const [marquee, setMarquee] = useState<{ x0: number; y0: number; x1: number; y1: number } | null>(
    null,
  );
  const [confirm, setConfirm] = useState<{ removal: Removal; count: number } | null>(null);
  const [renaming, setRenaming] = useState<string | null>(null);
  const [guide, setGuide] = useState<number | null>(null);

  const fps = flat?.fps ?? 30;
  const duration = flat?.durationInFrames ?? 1;
  const selectedSet = useMemo(() => new Set(selected), [selected]);
  const videoLanes = Math.max(1, flat?.lanes ?? 1);
  const audioLanes = Math.max(1, flat?.audioLanes ?? 1);
  const videoTop = RULER_H + MARKERS_H;
  const audioTop = videoTop + videoLanes * VIDEO_H;
  const totalH = audioTop + audioLanes * AUDIO_H;
  const contentW = Math.max(view.width - HEADER_W, duration * ppf + TAIL_PX);
  const viewport: Viewport = { left: view.left, width: view.width - HEADER_W, pxPerFrame: ppf };
  const markers = workflow.markers ?? NO_MARKERS;

  const fitZoom = useCallback(
    () =>
      Math.min(
        MAX_PX_PER_FRAME,
        Math.max(0.01, (view.width - HEADER_W - 32) / Math.max(1, duration)),
      ),
    [duration, view.width],
  );

  useLayoutEffect(() => {
    const el = scroller.current;
    if (!el) return;
    const update = () =>
      setView({ left: el.scrollLeft, width: el.clientWidth, height: el.clientHeight });
    update();
    const ro = new ResizeObserver(update);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  useEffect(() => {
    if (fit && !drag.current) useTimeline.getState().setZoom(fitZoom(), true);
  }, [fit, fitZoom]);

  useEffect(() => {
    const el = scroller.current;
    if (!el || !playing) return;
    const x = frame * ppf;
    const visibleW = el.clientWidth - HEADER_W;
    if (x < el.scrollLeft || x > el.scrollLeft + visibleW - 24) el.scrollLeft = Math.max(0, x - 24);
  }, [frame, playing, ppf]);

  const masterSrc = film.result?.master?.src ?? null;
  useEffect(() => preloadScrub(masterSrc), [masterSrc]);

  const liveSpec = useMemo(
    () => (film.spec ? applyMixLevels(film.spec, film.mix) : null),
    [film.spec, film.mix],
  );
  const duckFns = useMemo(() => {
    const out = new Map<string, (abs: number) => number>();
    if (!liveSpec) return out;
    for (const a of liveSpec.audio) {
      if (!a.duck) continue;
      const spans = duckingSpans(liveSpec, new Set(a.duck.keys));
      const { level, fadeFrames } = a.duck;
      out.set(a.id, (abs) => duckGain(abs, spans, level, fadeFrames));
    }
    return out;
  }, [liveSpec]);
  const noGain = useCallback(() => 1, []);

  const beats = useMedia((s) => s.waveforms);
  const beatFrames = useMemo(() => {
    if (!flat) return [] as number[];
    const out: number[] = [];
    for (const a of flat.audio) {
      if (a.role !== "music") continue;
      const w = beats[a.src];
      if (!w || typeof w !== "object" || !w.beats.length) continue;
      const len = w.durationMs / 1000;
      const offset = a.trimBefore / fps;
      for (let cycle = 0; cycle < (a.loop ? 200 : 1); cycle++) {
        let any = false;
        for (const b of w.beats) {
          const f = a.from + (b + cycle * len - offset) * fps;
          if (f < a.from) continue;
          if (f >= a.to) break;
          out.push(f);
          any = true;
        }
        if (!any && cycle > 0) break;
      }
    }
    return out;
  }, [flat, beats, fps]);

  const xToFrame = useCallback(
    (clientX: number) => {
      const el = scroller.current!;
      const rect = el.getBoundingClientRect();
      return (clientX - rect.left - HEADER_W + el.scrollLeft) / ppf;
    },
    [ppf],
  );

  const select = useCallback(
    (ids: string[], additive: boolean) => {
      const ui = useUiStore.getState();
      let next = ids;
      if (additive) {
        const set = new Set(ui.selectedIds);
        for (const id of ids) {
          if (set.has(id)) set.delete(id);
          else set.add(id);
        }
        next = [...set];
      }
      ui.selectPinned(next, film.sourceNodeId);
      if (ids.length === 1) ui.revealNode(ids[0]);
    },
    [film.sourceNodeId],
  );

  const snapTargets = useCallback(
    (exclude: string | null): SnapTarget[] => {
      if (!flat) return [];
      const t: SnapTarget[] = [{ frame: usePlayback.getState().frame, kind: "playhead" }];
      for (const b of flat.bars)
        if (b.key !== exclude)
          t.push({ frame: b.from, kind: "edge" }, { frame: b.to, kind: "edge" });
      for (const a of flat.audio)
        if (a.key !== exclude)
          t.push({ frame: a.from, kind: "edge" }, { frame: a.to, kind: "edge" });
      for (const m of markers) t.push({ frame: (m.atMs / 1000) * fps, kind: "marker" });
      for (const m of flat.marks) t.push({ frame: m.frame, kind: "action" });
      for (const b of beatFrames) t.push({ frame: b, kind: "beat" });
      const { inPoint: i, outPoint: o } = usePlayback.getState();
      if (i !== null) t.push({ frame: i, kind: "range" });
      if (o !== null) t.push({ frame: o, kind: "range" });
      t.push({ frame: 0, kind: "edge" }, { frame: duration, kind: "edge" });
      return t;
    },
    [flat, markers, fps, beatFrames, duration],
  );

  const editCtx = useCallback(
    (base: Workflow, salt: string): EditContext => {
      let n = 0;
      return {
        workflow: base,
        fps,
        sourceNodeId: film.sourceNodeId,
        makeId: (prefix) => `${prefix}_${salt}${(n++).toString(36)}`,
      };
    },
    [fps, film.sourceNodeId],
  );

  const commitResult = (r: EditResult | null, quiet = false) => {
    if (!r) return;
    if (!r.ok) {
      if (!quiet) toast.error(r.error);
      return;
    }
    useWorkflowStore.getState().update(() => r.workflow);
  };

  const neighbour = (bar: VideoBar, side: "left" | "right") => {
    if (!flat) return null;
    const container = containerOf(bar);
    return (
      flat.bars.find(
        (b) =>
          b.key !== bar.key &&
          containerOf(b) === container &&
          (side === "right" ? Math.abs(b.from - bar.to) < 1 : Math.abs(b.to - bar.from) < 1),
      ) ?? null
    );
  };

  const applyDrag = (d: EditDrag, clientX: number, shift: boolean, alt: boolean) => {
    const raw = (clientX - d.x0) / ppf;
    const threshold = snapping && !alt ? SNAP_PX / ppf : 0;
    const ctx = editCtx(d.base, d.salt);
    let delta = raw;
    let snappedAt: number | null = null;
    let result: EditResult | null = null;
    if (d.kind === "move" || d.kind === "audio-move") {
      const from = d.bar.from;
      const to = d.bar.to;
      const s = threshold
        ? snapSpan(from, to, raw, d.targets, threshold)
        : { delta: raw, target: null };
      delta = Math.round(s.delta);
      if (s.target) snappedAt = s.target.frame;
      result =
        d.kind === "move" ? moveBar(ctx, d.bar, delta) : moveAudio(ctx, d.bar as AudioBar, delta);
      if (result.ok && (d.ripple || shift)) {
        const key = moveHandleKey(d.bar);
        const targets = rippleTargets(ctx, d.flat.bars, d.flat.audio, d.bar.from + 0.5, {
          container: d.kind === "move" ? containerOf(d.bar) : undefined,
          refs: new Set(key ? [key] : []),
        });
        result = { ...result, workflow: applyRipple(ctx, result.workflow, targets, delta) };
      }
    } else {
      const edgeFrame =
        d.kind === "trim-start" || d.kind === "roll-start" || d.kind === "audio-trim-start"
          ? d.bar.from
          : d.bar.to;
      const s = threshold
        ? snapFrame(edgeFrame + raw, d.targets, threshold)
        : { frame: edgeFrame + raw, target: null };
      delta = Math.round(s.frame - edgeFrame);
      if (s.target) snappedAt = s.target.frame;
      if (d.kind === "audio-trim-start") result = trimAudio(ctx, d.bar, "start", delta);
      else if (d.kind === "audio-trim-end") result = trimAudio(ctx, d.bar, "end", delta);
      else if (d.kind === "trim-start") result = trimBar(ctx, d.bar, "start", delta);
      else if (d.kind === "trim-end") {
        result = trimBar(ctx, d.bar, "end", delta);
        if (result.ok && d.ripple) {
          const targets = rippleTargets(ctx, d.flat.bars, d.flat.audio, d.bar.to - 0.5, {
            container: containerOf(d.bar),
            refs: new Set(),
          });
          result = { ...result, workflow: applyRipple(ctx, result.workflow, targets, delta) };
        }
      } else if (d.kind === "roll-end") {
        const right = neighbour(d.bar, "right");
        result = right ? rollCut(ctx, d.bar, right, delta) : trimBar(ctx, d.bar, "end", delta);
      } else if (d.kind === "roll-start") {
        const left = neighbour(d.bar, "left");
        result = left ? rollCut(ctx, left, d.bar, delta) : trimBar(ctx, d.bar, "start", delta);
      } else if (d.kind === "slip") {
        delta = Math.round(-raw);
        snappedAt = null;
        result = slipBar(ctx, d.bar, delta);
      } else if (d.kind === "speed")
        result = speedBar(ctx, d.bar, Math.max(1, Math.round(d.bar.to - d.bar.from + delta)));
    }
    setGuide(snappedAt);
    d.last = result;
    if (result?.ok) commitResult(result, true);
  };

  const beginEdit = (
    kind: EditDrag["kind"],
    bar: VideoBar | AudioBar,
    clientX: number,
    ripple: boolean,
  ) => {
    if (!flat) return;
    drag.current = {
      kind,
      bar,
      x0: clientX,
      base: useWorkflowStore.getState().workflow!,
      flat,
      targets: snapTargets(bar.key),
      salt: gestureSalt(),
      started: false,
      last: null,
      ripple,
    } as Drag;
  };

  const hitEdge = (e: React.PointerEvent, width: number): "start" | "end" | null => {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const x = e.clientX - rect.left;
    const edge = Math.min(EDGE_PX, width / 3);
    if (x <= edge) return "start";
    if (x >= rect.width - edge) return "end";
    return null;
  };

  const onVideoDown = (e: React.PointerEvent, bar: VideoBar, width: number) => {
    if (e.button !== 0) return;
    e.stopPropagation();
    (e.currentTarget as HTMLElement).closest<HTMLElement>("[data-timeline-root]")?.focus();
    scroller.current?.setPointerCapture(e.pointerId);
    const t: TimelineTool = tool;
    if (t === "blade") {
      const at = xToFrame(e.clientX);
      const s =
        snapping && !e.altKey ? snapFrame(at, snapTargets(null), SNAP_PX / ppf) : { frame: at };
      const r = splitBar(editCtx(workflow, gestureSalt()), bar, Math.round(s.frame));
      if (r.ok) {
        useWorkflowStore.getState().update(() => r.workflow);
        if (r.message) toast.success(r.message);
      } else toast.error(r.error);
      return;
    }
    if (!selectedSet.has(bar.nodeId) || e.shiftKey)
      select([bar.nodeId], e.shiftKey && t === "select");
    const corner = (e.target as HTMLElement).dataset.corner === "speed";
    const edge = hitEdge(e, width);
    if (corner) return beginEdit("speed", bar, e.clientX, false);
    if (t === "slip") return beginEdit("slip", bar, e.clientX, false);
    if (edge && t === "rate") return beginEdit("speed", bar, e.clientX, false);
    if (edge && t === "roll")
      return beginEdit(edge === "start" ? "roll-start" : "roll-end", bar, e.clientX, false);
    if (edge)
      return beginEdit(
        edge === "start" ? "trim-start" : "trim-end",
        bar,
        e.clientX,
        t === "ripple",
      );
    beginEdit("move", bar, e.clientX, t === "ripple");
  };

  const onAudioDown = (e: React.PointerEvent, bar: AudioBar, width: number, height: number) => {
    if (e.button !== 0) return;
    e.stopPropagation();
    (e.currentTarget as HTMLElement).closest<HTMLElement>("[data-timeline-root]")?.focus();
    scroller.current?.setPointerCapture(e.pointerId);
    if (bar.nodeId && (!selectedSet.has(bar.nodeId) || e.shiftKey))
      select([bar.nodeId], e.shiftKey);
    if (tool === "blade" && bar.bar) {
      const r = splitBar(
        editCtx(workflow, gestureSalt()),
        bar.bar,
        Math.round(xToFrame(e.clientX)),
      );
      if (r.ok) useWorkflowStore.getState().update(() => r.workflow);
      else toast.error(r.error);
      return;
    }
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const y = e.clientY - rect.top;
    const lineY = height * (1 - Math.min(2, bar.volume) / 2);
    const edge = hitEdge(e, width);
    if (
      !edge &&
      Math.abs(y - lineY) <= 4 &&
      audioEnvelope === "volume" &&
      !bar.item?.envelope?.volume?.length
    ) {
      drag.current = {
        kind: "volume",
        bar,
        y0: e.clientY,
        v0: bar.volume,
        base: workflow,
        started: false,
        height,
      };
      return;
    }
    if (edge)
      return beginEdit(
        edge === "start" ? "audio-trim-start" : "audio-trim-end",
        bar,
        e.clientX,
        tool === "ripple",
      );
    beginEdit("audio-move", bar, e.clientX, tool === "ripple");
  };

  const onKeyframeDown = (
    e: React.PointerEvent,
    nodeId: string,
    prop: VisualEnvelopeKey | AudioEnvelopeKey,
    keys: Keyframe[],
    index: number,
    toLocal: (abs: number) => number,
    height: number,
  ) => {
    e.stopPropagation();
    if (e.button !== 0) return;
    scroller.current?.setPointerCapture(e.pointerId);
    drag.current = {
      kind: "key",
      nodeId,
      prop,
      index,
      keys,
      toLocal,
      x0: e.clientX,
      y0: e.clientY,
      height,
      started: false,
    };
  };

  const onEmptyDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    const rect = scroller.current!.getBoundingClientRect();
    const x = e.clientX - rect.left + scroller.current!.scrollLeft;
    const y = e.clientY - rect.top + scroller.current!.scrollTop;
    if (y < RULER_H + scroller.current!.scrollTop && y >= scroller.current!.scrollTop) return;
    scroller.current?.setPointerCapture(e.pointerId);
    drag.current = { kind: "marquee", x0: x, y0: y, x1: x, y1: y, additive: e.shiftKey };
  };

  const onRulerDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    e.stopPropagation();
    scroller.current?.setPointerCapture(e.pointerId);
    drag.current = { kind: "scrub" };
    const f = Math.max(0, xToFrame(e.clientX));
    usePlayback.getState().seek(f);
    scrubAt(masterSrc, f / fps);
  };

  const onPointerMove = (e: React.PointerEvent) => {
    const d = drag.current;
    if (!d) return;
    if (d.kind === "scrub") {
      const f = Math.max(0, Math.min(duration - 1, xToFrame(e.clientX)));
      usePlayback.getState().seek(f);
      scrubAt(masterSrc, f / fps);
      return;
    }
    if (d.kind === "marquee") {
      const rect = scroller.current!.getBoundingClientRect();
      d.x1 = e.clientX - rect.left + scroller.current!.scrollLeft;
      d.y1 = e.clientY - rect.top + scroller.current!.scrollTop;
      setMarquee({ x0: d.x0, y0: d.y0, x1: d.x1, y1: d.y1 });
      return;
    }
    if (d.kind === "volume") {
      if (!d.started) {
        if (Math.abs(e.clientY - d.y0) < 2) return;
        d.started = true;
        useWorkflowStore.getState().beginGesture();
      }
      const v = d.v0 - ((e.clientY - d.y0) / d.height) * 2;
      const r = setAudioVolume(editCtx(d.base, "v"), d.bar, v);
      if (r.ok) useWorkflowStore.getState().update(() => r.workflow);
      return;
    }
    if (d.kind === "key") {
      if (!d.started) {
        if (Math.abs(e.clientX - d.x0) + Math.abs(e.clientY - d.y0) < 3) return;
        d.started = true;
        useWorkflowStore.getState().beginGesture();
      }
      const [lo, hi] = ENVELOPE_RANGE[d.prop];
      const key = d.keys[d.index];
      const frameAt = Math.max(0, Math.round(d.toLocal(xToFrame(e.clientX))));
      const value = Math.min(
        hi,
        Math.max(lo, key.value - ((e.clientY - d.y0) / Math.max(1, d.height - 10)) * (hi - lo)),
      );
      const keys = d.keys.map((k, i) =>
        i === d.index ? { ...k, frame: frameAt, value: Math.round(value * 1000) / 1000 } : k,
      );
      setEnvelopeKeys(d.nodeId, d.prop, keys);
      return;
    }
    if (d.kind === "marker") {
      if (!d.started) {
        if (Math.abs(e.clientX - d.x0) < 3) return;
        d.started = true;
        useWorkflowStore.getState().beginGesture();
      }
      let f = (d.at0 / 1000) * fps + (e.clientX - d.x0) / ppf;
      if (snapping && !e.altKey)
        f = snapFrame(
          f,
          snapTargets(null).filter((t) => t.kind !== "marker"),
          SNAP_PX / ppf,
        ).frame;
      const atMs = Math.max(0, Math.round((f / fps) * 1000));
      useWorkflowStore
        .getState()
        .setMarkers((list) => list.map((m) => (m.id === d.id ? { ...m, atMs } : m)));
      return;
    }
    if (!d.started) {
      if (Math.abs(e.clientX - d.x0) < 3) return;
      d.started = true;
      useWorkflowStore.getState().beginGesture();
    }
    applyDrag(d, e.clientX, e.shiftKey, e.altKey);
  };

  const onPointerUp = (e: React.PointerEvent) => {
    const d = drag.current;
    drag.current = null;
    setGuide(null);
    if (useTimeline.getState().fit)
      requestAnimationFrame(() => useTimeline.getState().setZoom(fitZoom(), true));
    if (scroller.current?.hasPointerCapture(e.pointerId))
      scroller.current.releasePointerCapture(e.pointerId);
    if (!d) return;
    if (d.kind === "marquee") {
      setMarquee(null);
      if (!flat) return;
      const x0 = Math.min(d.x0, d.x1) - HEADER_W;
      const x1 = Math.max(d.x0, d.x1) - HEADER_W;
      const y0 = Math.min(d.y0, d.y1);
      const y1 = Math.max(d.y0, d.y1);
      if (x1 - x0 < 3 && y1 - y0 < 3) {
        if (!d.additive) useUiStore.getState().selectPinned([], film.sourceNodeId);
        usePlayback.getState().seek(Math.max(0, (d.x0 - HEADER_W) / ppf));
        useTimeline.getState().selectMarker(null);
        return;
      }
      const ids = new Set<string>();
      for (const b of flat.bars) {
        const top = videoTop + (videoLanes - 1 - b.lane) * VIDEO_H;
        if (b.from * ppf < x1 && b.to * ppf > x0 && top < y1 && top + VIDEO_H > y0)
          ids.add(b.nodeId);
      }
      for (const a of flat.audio) {
        const top = audioTop + a.lane * AUDIO_H;
        if (a.nodeId && a.from * ppf < x1 && a.to * ppf > x0 && top < y1 && top + AUDIO_H > y0)
          ids.add(a.nodeId);
      }
      select([...ids], d.additive);
      return;
    }
    if (d.kind === "scrub") return;
    if ("started" in d && d.started) {
      useWorkflowStore.getState().endGesture();
      if ("last" in d && d.last) {
        if (!d.last.ok) toast.error(d.last.error);
        else if (d.last.message) toast.success(d.last.message);
      }
    } else if (d.kind === "marker") {
      useTimeline.getState().selectMarker(d.id);
    } else if (d.kind === "key") {
      select([d.nodeId], false);
    }
  };

  const zoomAround = useCallback(
    (factor: number, anchorFrame?: number) => {
      const el = scroller.current;
      if (!el) return;
      const anchor = anchorFrame ?? usePlayback.getState().frame;
      const screenX = anchor * ppf - el.scrollLeft;
      const next = Math.min(MAX_PX_PER_FRAME, Math.max(0.01, ppf * factor));
      useTimeline.getState().setZoom(next);
      requestAnimationFrame(() => {
        el.scrollLeft = Math.max(0, anchor * next - screenX);
      });
    },
    [ppf],
  );

  const onWheel = (e: React.WheelEvent) => {
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      zoomAround(e.deltaY < 0 ? 1.15 : 1 / 1.15, xToFrame(e.clientX));
    } else if (!e.shiftKey && Math.abs(e.deltaY) > Math.abs(e.deltaX) && totalH <= view.height) {
      scroller.current!.scrollLeft += e.deltaY;
    }
  };

  useEffect(() => {
    const el = scroller.current;
    if (!el) return;
    const block = (e: WheelEvent) => {
      if (e.ctrlKey || e.metaKey) e.preventDefault();
    };
    el.addEventListener("wheel", block, { passive: false });
    return () => el.removeEventListener("wheel", block);
  }, []);

  const removeSelected = () => {
    if (!flat) return;
    const marker = useTimeline.getState().selectedMarker;
    if (marker) {
      useWorkflowStore.getState().setMarkers((list) => list.filter((m) => m.id !== marker));
      useTimeline.getState().selectMarker(null);
      return;
    }
    const ctx = editCtx(workflow, "d");
    const nodes = new Set<string>();
    const edges = new Set<string>();
    const bars = [
      ...flat.bars.filter((b) => selectedSet.has(b.nodeId)),
      ...flat.audio.filter((a) => a.nodeId && selectedSet.has(a.nodeId) && !a.bar),
    ];
    for (const bar of bars) {
      const r = removalFor(ctx, bar);
      r.nodeIds.forEach((id) => nodes.add(id));
      r.edgeIds.forEach((id) => edges.add(id));
    }
    if (!nodes.size && !edges.size) return;
    const removal = { nodeIds: [...nodes], edgeIds: [...edges] };
    if (nodes.size > 1) setConfirm({ removal, count: nodes.size });
    else {
      useWorkflowStore.getState().update((wf) => applyRemoval(wf, removal));
      useUiStore.getState().selectPinned([], film.sourceNodeId);
    }
  };

  const split = () => {
    if (!flat) return;
    const at = usePlayback.getState().frame;
    const under = flat.bars.filter((b) => b.from < at && b.to > at);
    const bar =
      under.find((b) => selectedSet.has(b.nodeId)) ?? under.toSorted((a, b) => b.lane - a.lane)[0];
    if (!bar) {
      toast.error("Put the playhead over a clip to split it.");
      return;
    }
    const r = splitBar(editCtx(workflow, gestureSalt()), bar, at);
    if (!r.ok) {
      toast.error(r.error);
      return;
    }
    useWorkflowStore.getState().update(() => r.workflow);
    if (r.message) toast.success(r.message);
  };

  const addMarker = () => {
    const at = usePlayback.getState().frame;
    const id = newId("marker");
    useWorkflowStore
      .getState()
      .setMarkers((list) => [
        ...list,
        { id, atMs: Math.round((at / fps) * 1000), label: `Marker ${list.length + 1}` },
      ]);
    useTimeline.getState().selectMarker(id);
  };

  useImperativeHandle(ref, () => ({
    split,
    remove: removeSelected,
    addMarker,
    zoomBy: (f) => zoomAround(f),
    fit: () => useTimeline.getState().setZoom(fitZoom(), true),
  }));

  if (!flat) return null;

  const playheadX = HEADER_W + frame * ppf;
  const { step, minor } = rulerStep(ppf, fps);
  const firstTick = Math.floor(view.left / ppf / minor) * minor;
  const lastTick = (view.left + view.width) / ppf;
  const ticks: Array<{ f: number; major: boolean }> = [];
  for (let f = firstTick; f <= lastTick && ticks.length < 600; f += minor)
    ticks.push({ f, major: Math.abs(f / step - Math.round(f / step)) < 1e-6 });
  const inView = (from: number, to: number) =>
    to * ppf >= view.left - 64 && from * ppf <= view.left + view.width + 64;

  const visualKeysFor = (bar: VideoBar) => {
    if (!videoEnvelope || !supportsEnvelope(workflow, bar.nodeId)) return null;
    return envelopeOf(workflow, bar.nodeId)[videoEnvelope] ?? [];
  };

  return (
    <div
      ref={scroller}
      className="relative h-full overflow-auto select-none"
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
      onPointerDown={onEmptyDown}
      onWheel={onWheel}
      onScroll={(e) => {
        const el = e.currentTarget;
        setView((v) => (v.left === el.scrollLeft ? v : { ...v, left: el.scrollLeft }));
      }}
    >
      <div className="relative" style={{ width: HEADER_W + contentW, height: totalH }}>
        <div
          className="bg-sidebar sticky top-0 z-30 flex border-b"
          style={{ height: RULER_H, width: HEADER_W + contentW }}
        >
          <div
            className="bg-sidebar sticky left-0 z-10 shrink-0 border-r"
            style={{ width: HEADER_W }}
          />
          <div className="relative flex-1 cursor-text" onPointerDown={onRulerDown}>
            {inPoint !== null || outPoint !== null ? (
              <div
                className="bg-primary/15 absolute inset-y-0"
                style={{
                  left: (inPoint ?? 0) * ppf,
                  width: ((outPoint ?? duration) - (inPoint ?? 0)) * ppf,
                }}
              />
            ) : null}
            {ticks.map(({ f, major }) => (
              <div
                key={f}
                className={cn(
                  "absolute bottom-0 w-px",
                  major ? "bg-muted-foreground/60 h-2.5" : "bg-muted-foreground/30 h-1.5",
                )}
                style={{ left: f * ppf }}
              >
                {major ? (
                  <span className="text-muted-foreground absolute bottom-2.5 left-1 text-[10px] whitespace-nowrap tabular-nums">
                    {formatTime(f, fps, rulerMode)}
                  </span>
                ) : null}
              </div>
            ))}
            <div
              className="bg-border absolute inset-y-0"
              style={{ left: duration * ppf, width: 1 }}
            />
          </div>
        </div>

        <Row top={RULER_H} height={MARKERS_H} label="Markers" width={contentW}>
          {beatFrames
            .filter((f) => inView(f, f))
            .map((f, i) => (
              <div
                key={`b${i}`}
                className="bg-track-music absolute bottom-0 h-1.5 w-px opacity-80"
                style={{ left: f * ppf }}
              />
            ))}
          {flat.marks
            .filter((m) => m.kind === "chapter" && inView(m.frame, m.frame))
            .map((m) => (
              <div
                key={m.key}
                title={m.label}
                className="text-muted-foreground absolute top-0.5 flex h-4 items-center gap-1 text-[10px] whitespace-nowrap"
                style={{ left: m.frame * ppf }}
              >
                <span className="bg-muted-foreground/70 h-3 w-0.5" />
                {m.label}
              </div>
            ))}
          {markers.map((m) => {
            const x = (m.atMs / 1000) * fps * ppf;
            return (
              <div
                key={m.id}
                className={cn(
                  "absolute top-0.5 flex h-4 cursor-grab items-center rounded-r px-1 text-[10px] whitespace-nowrap",
                  selectedMarker === m.id
                    ? "bg-playhead text-background"
                    : "bg-accent text-foreground",
                )}
                style={{ left: x }}
                onPointerDown={(e) => {
                  e.stopPropagation();
                  if (e.button !== 0) return;
                  if (secondClick(e)) {
                    setRenaming(m.id);
                    return;
                  }
                  scroller.current?.setPointerCapture(e.pointerId);
                  drag.current = {
                    kind: "marker",
                    id: m.id,
                    x0: e.clientX,
                    at0: m.atMs,
                    started: false,
                  };
                }}
              >
                {renaming === m.id ? (
                  <Input
                    autoFocus
                    defaultValue={m.label}
                    className="h-4 w-28 px-1 text-[10px]"
                    onPointerDown={(e) => e.stopPropagation()}
                    onKeyDown={(e) => {
                      e.stopPropagation();
                      if (e.key === "Enter" || e.key === "Escape")
                        (e.target as HTMLInputElement).blur();
                    }}
                    onBlur={(e) => {
                      const label = e.target.value.trim() || m.label;
                      setRenaming(null);
                      if (label !== m.label)
                        useWorkflowStore
                          .getState()
                          .setMarkers((list) =>
                            list.map((x2) => (x2.id === m.id ? { ...x2, label } : x2)),
                          );
                    }}
                  />
                ) : (
                  m.label
                )}
              </div>
            );
          })}
        </Row>

        {Array.from({ length: videoLanes }, (_, i) => videoLanes - 1 - i).map((lane, row) => (
          <Row
            key={`v${lane}`}
            top={videoTop + row * VIDEO_H}
            height={VIDEO_H}
            label={`Video ${lane + 1}`}
            width={contentW}
          >
            {flat.bars
              .filter((b) => b.lane === lane && inView(b.from, b.to))
              .map((bar) => {
                const left = bar.from * ppf;
                const width = Math.max(2, (bar.to - bar.from) * ppf);
                const isSelected = selectedSet.has(bar.nodeId);
                const related =
                  !isSelected && bar.chain.some((l) => l.nodeId && selectedSet.has(l.nodeId));
                const wrappers = bar.chain
                  .slice(1)
                  .flatMap((l) =>
                    l.nodeId && l.nodeId !== bar.nodeId ? [nodeTitle(workflow, l.nodeId)] : [],
                  );
                const keys = visualKeysFor(bar);
                return (
                  <div
                    key={bar.key}
                    className={cn(
                      "absolute top-1 bottom-1 overflow-hidden rounded-md border border-foreground/10",
                      toneFor(workflow, bar),
                      isSelected && "ring-primary ring-2",
                      related && "ring-1 ring-foreground/40",
                      tool === "blade"
                        ? "cursor-crosshair"
                        : tool === "slip"
                          ? "cursor-ew-resize"
                          : "cursor-grab",
                    )}
                    style={{ left, width }}
                    title={[nodeTitle(workflow, bar.nodeId), ...wrappers].join(" · ")}
                    onPointerDown={(e) => onVideoDown(e, bar, width)}
                    onPointerMove={(e) => {
                      if (drag.current || tool === "blade" || tool === "slip") return;
                      const edge = hitEdge(e, width);
                      (e.currentTarget as HTMLElement).style.cursor = edge ? "ew-resize" : "";
                    }}
                    onPointerDownCapture={(e) => {
                      if (!keys || !videoEnvelope || e.button !== 0 || !secondClick(e)) return;
                      e.stopPropagation();
                      const local = Math.round(bar.localAt(xToFrame(e.clientX)));
                      const [, , def] = ENVELOPE_RANGE[videoEnvelope];
                      setEnvelopeKeys(bar.nodeId, videoEnvelope, [
                        ...keys,
                        { frame: local, value: envelopeAt(keys, local, def), ease: "linear" },
                      ]);
                    }}
                  >
                    <FilmstripCanvas
                      bar={bar}
                      left={left}
                      width={width}
                      height={VIDEO_H - 8}
                      view={viewport}
                      fps={fps}
                    />
                    <div className="pointer-events-none relative flex h-full flex-col justify-between px-1.5 py-0.5">
                      <span className="truncate text-[11px] leading-tight font-medium text-foreground drop-shadow">
                        {nodeTitle(workflow, bar.nodeId)}
                        {wrappers.length ? (
                          <span className="text-foreground/60"> · {wrappers.join(" · ")}</span>
                        ) : null}
                      </span>
                    </div>
                    {flat.marks
                      .filter((m) => m.barKey === bar.key && m.kind !== "chapter")
                      .map((m) => (
                        <div
                          key={m.key}
                          title={m.label}
                          className="absolute bottom-0 h-2 w-0.5 bg-foreground/80 hover:h-3"
                          style={{ left: m.frame * ppf - left }}
                        />
                      ))}
                    {keys && videoEnvelope ? (
                      <EnvelopeOverlay
                        keys={keys}
                        prop={videoEnvelope}
                        width={width}
                        height={VIDEO_H - 8}
                        toX={(local) => {
                          const abs =
                            bar.from + (local - bar.contentStart) / Math.max(1e-6, bar.rate);
                          return (abs - bar.from) * ppf;
                        }}
                        onDown={(e, i) =>
                          onKeyframeDown(
                            e,
                            bar.nodeId,
                            videoEnvelope,
                            keys,
                            i,
                            (abs) => bar.localAt(abs),
                            VIDEO_H - 8,
                          )
                        }
                        onRemove={(i) =>
                          setEnvelopeKeys(
                            bar.nodeId,
                            videoEnvelope,
                            keys.filter((_key, k) => k !== i),
                          )
                        }
                      />
                    ) : null}
                    <div
                      data-corner="speed"
                      title="Drag to change the speed"
                      className="absolute top-0 right-0 size-2.5 cursor-nesw-resize bg-foreground/30 [clip-path:polygon(0_0,100%_0,100%_100%)] hover:bg-foreground/70"
                    />
                  </div>
                );
              })}
          </Row>
        ))}

        {flat.transitions
          .filter((t) => inView(t.from, t.to) && t.lanes.length)
          .map((t) => {
            const top = Math.max(...t.lanes);
            const bottom = Math.min(...t.lanes);
            const heavy = t.covers > 0.4;
            return (
              <div
                key={t.key}
                className={cn(
                  "pointer-events-none absolute z-10 flex items-end justify-center overflow-hidden rounded-sm border bg-[repeating-linear-gradient(135deg,var(--transition-hatch)_0,var(--transition-hatch)_4px,transparent_4px,transparent_8px)]",
                  heavy ? "border-warning" : "border-foreground/30",
                )}
                style={{
                  left: HEADER_W + t.from * ppf,
                  width: Math.max(4, (t.to - t.from) * ppf),
                  top: videoTop + (videoLanes - 1 - top) * VIDEO_H + 4,
                  height: (top - bottom + 1) * VIDEO_H - 8,
                }}
              >
                {(t.to - t.from) * ppf > 40 ? (
                  <button
                    type="button"
                    className="bg-background/70 hover:bg-background pointer-events-auto mb-0.5 rounded px-1 text-[10px] whitespace-nowrap"
                    title={transitionTip(workflow, t)}
                    onPointerDown={(e) => e.stopPropagation()}
                    onClick={() => t.slot && select([t.slot.nodeId], false)}
                  >
                    {transitionTitle(t.kind)}
                  </button>
                ) : null}
              </div>
            );
          })}

        {Array.from({ length: audioLanes }, (_, lane) => (
          <Row
            key={`a${lane}`}
            top={audioTop + lane * AUDIO_H}
            height={AUDIO_H}
            label={`Sound ${lane + 1}`}
            width={contentW}
          >
            {flat.audio
              .filter((a) => a.lane === lane && inView(a.from, a.to))
              .map((a) => {
                const left = a.from * ppf;
                const width = Math.max(2, (a.to - a.from) * ppf);
                const h = AUDIO_H - 8;
                const isSelected = !!a.nodeId && selectedSet.has(a.nodeId);
                const keys =
                  a.nodeId && supportsEnvelope(workflow, a.nodeId)
                    ? (envelopeOf(workflow, a.nodeId)[audioEnvelope] ?? [])
                    : null;
                const lineY = h * (1 - Math.min(2, a.volume) / 2);
                return (
                  <div
                    key={a.key}
                    className={cn(
                      "absolute top-1 bottom-1 overflow-hidden rounded-md border border-foreground/10",
                      AUDIO_TONE[a.role],
                      isSelected && "ring-primary ring-2",
                      "cursor-grab",
                    )}
                    style={{ left, width }}
                    title={a.nodeId ? nodeTitle(workflow, a.nodeId) : undefined}
                    onPointerDown={(e) => onAudioDown(e, a, width, h)}
                    onPointerMove={(e) => {
                      if (drag.current) return;
                      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
                      const y = e.clientY - rect.top;
                      const edge = hitEdge(e, width);
                      (e.currentTarget as HTMLElement).style.cursor = edge
                        ? "ew-resize"
                        : Math.abs(y - lineY) <= 4 && !keys?.length && audioEnvelope === "volume"
                          ? "ns-resize"
                          : "";
                    }}
                    onPointerDownCapture={(e) => {
                      if (!keys || !a.nodeId || e.button !== 0 || !secondClick(e)) return;
                      e.stopPropagation();
                      const local = Math.round(xToFrame(e.clientX) - a.from);
                      const [, , def] = ENVELOPE_RANGE[audioEnvelope];
                      setEnvelopeKeys(a.nodeId, audioEnvelope, [
                        ...keys,
                        { frame: local, value: envelopeAt(keys, local, def), ease: "linear" },
                      ]);
                    }}
                  >
                    <WaveformCanvas
                      bar={a}
                      left={left}
                      width={width}
                      height={h}
                      view={viewport}
                      fps={fps}
                      gain={duckFns.get(a.id) ?? noGain}
                    />
                    <span className="pointer-events-none relative truncate px-1.5 text-[11px] font-medium text-foreground drop-shadow">
                      {a.nodeId ? nodeTitle(workflow, a.nodeId) : ""}
                    </span>
                    {audioEnvelope === "volume" && !keys?.length ? (
                      <div
                        className="pointer-events-none absolute inset-x-0 h-px bg-keyframe/80"
                        style={{ top: lineY }}
                      />
                    ) : null}
                    {keys && a.nodeId ? (
                      <EnvelopeOverlay
                        keys={keys}
                        prop={audioEnvelope}
                        width={width}
                        height={h}
                        toX={(local) => local * ppf}
                        onDown={(e, i) =>
                          onKeyframeDown(
                            e,
                            a.nodeId!,
                            audioEnvelope,
                            keys,
                            i,
                            (abs) => abs - a.from,
                            h,
                          )
                        }
                        onRemove={(i) =>
                          setEnvelopeKeys(
                            a.nodeId!,
                            audioEnvelope,
                            keys.filter((_key, k) => k !== i),
                          )
                        }
                      />
                    ) : null}
                  </div>
                );
              })}
          </Row>
        ))}

        {guide !== null ? (
          <div
            className="pointer-events-none absolute top-0 z-30 w-px bg-keyframe"
            style={{ left: HEADER_W + guide * ppf, height: totalH }}
          />
        ) : null}
        {marquee ? (
          <div
            className="border-primary bg-primary/10 pointer-events-none absolute z-30 border"
            style={{
              left: Math.min(marquee.x0, marquee.x1),
              top: Math.min(marquee.y0, marquee.y1),
              width: Math.abs(marquee.x1 - marquee.x0),
              height: Math.abs(marquee.y1 - marquee.y0),
            }}
          />
        ) : null}
        <div
          className="bg-playhead pointer-events-none absolute top-0 z-30 w-px"
          style={{ left: playheadX, height: Math.max(totalH, view.height) }}
        >
          <div className="bg-playhead absolute -left-1.5 top-0 h-2.5 w-3 [clip-path:polygon(0_0,100%_0,50%_100%)]" />
        </div>
      </div>
      <AlertDialog open={!!confirm} onOpenChange={(open) => !open && setConfirm(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Remove {confirm?.count} nodes?</AlertDialogTitle>
            <AlertDialogDescription>
              These nodes only feed the selected clips. Undo brings them back.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={() => {
                if (confirm)
                  useWorkflowStore.getState().update((wf) => applyRemoval(wf, confirm.removal));
                useUiStore.getState().selectPinned([], film.sourceNodeId);
                setConfirm(null);
              }}
            >
              Remove
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

function Row({
  top,
  height,
  label,
  width,
  children,
}: {
  top: number;
  height: number;
  label: string;
  width: number;
  children: React.ReactNode;
}) {
  return (
    <div
      className="absolute left-0 flex border-b border-foreground/5"
      style={{ top, height, width: HEADER_W + width }}
    >
      <div
        className="bg-sidebar text-muted-foreground sticky left-0 z-20 flex shrink-0 items-center border-r px-2 text-[11px]"
        style={{ width: HEADER_W }}
      >
        {label}
      </div>
      <div className="relative flex-1">{children}</div>
    </div>
  );
}

const KEY_NAMES: Record<VisualEnvelopeKey | AudioEnvelopeKey, string> = {
  opacity: "Opacity",
  x: "Move across",
  y: "Move up and down",
  scale: "Size",
  volume: "Volume",
  pan: "Balance",
};

function EnvelopeOverlay({
  keys,
  prop,
  width,
  height,
  toX,
  onDown,
  onRemove,
}: {
  keys: Keyframe[];
  prop: VisualEnvelopeKey | AudioEnvelopeKey;
  width: number;
  height: number;
  toX: (local: number) => number;
  onDown: (e: React.PointerEvent, index: number) => void;
  onRemove: (index: number) => void;
}) {
  const [lo, hi, def] = ENVELOPE_RANGE[prop];
  const pad = 5;
  const y = (v: number) => pad + (1 - (v - lo) / (hi - lo)) * (height - pad * 2);
  const sorted = keys.map((k, i) => ({ k, i })).toSorted((a, b) => a.k.frame - b.k.frame);
  const points: string[] = [];
  if (sorted.length === 0) points.push(`0,${y(def)}`, `${width},${y(def)}`);
  else {
    points.push(`0,${y(sorted[0].k.value)}`);
    for (let s = 0; s < sorted.length; s++) {
      const a = sorted[s].k;
      const b = sorted[s + 1]?.k;
      points.push(`${toX(a.frame)},${y(a.value)}`);
      if (b && a.ease !== "linear") {
        for (let j = 1; j < 8; j++) {
          const f = a.frame + ((b.frame - a.frame) * j) / 8;
          points.push(`${toX(f)},${y(envelopeAt([a, b], f, a.value))}`);
        }
      }
    }
    points.push(`${width},${y(sorted[sorted.length - 1].k.value)}`);
  }
  return (
    <svg className="absolute inset-0 overflow-visible" width={width} height={height}>
      <polyline
        points={points.join(" ")}
        fill="none"
        className="stroke-keyframe pointer-events-none"
        strokeWidth={1.25}
      />
      {sorted.map(({ k, i }) => (
        <circle
          key={i}
          cx={toX(k.frame)}
          cy={y(k.value)}
          r={4}
          className="fill-keyframe stroke-background cursor-move"
          aria-label={`${KEY_NAMES[prop]} ${Math.round(k.value * 1000) / 1000}`}
          onPointerDown={(e) => {
            if (e.button === 0 && secondClick(e)) {
              e.stopPropagation();
              onRemove(i);
              return;
            }
            onDown(e, i);
          }}
        >
          <title>
            {`${KEY_NAMES[prop]} ${Math.round(k.value * 1000) / 1000}. Drag to change, double-click to delete. Click to type exact values in the Inspector.`}
          </title>
        </circle>
      ))}
    </svg>
  );
}
