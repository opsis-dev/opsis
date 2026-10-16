import { useMemo } from "react";
import { duckGain, duckingSpans } from "@/shared/compile/duck";
import { applyMixLevels } from "@/shared/mix/levels";
import type { Mix } from "@/shared/mix/types";
import type { TimelineSpec } from "@/shared/schema/timeline";
import { splitBar } from "@/shared/timeline/edits";
import type { AudioBar } from "@/shared/timeline/flatten";
import { cn } from "@/renderer/lib/utils";
import { WaveformCanvas } from "./bar-canvas";
import { nodeTitle, type BarContext } from "./bar-context";
import { envelopeKeysFor } from "./envelope";
import { EnvelopeOverlay, useKeyframeAdder } from "./envelope-overlay";
import { commitEdit } from "./timeline-actions";
import { AUDIO_H, BAR_INSET, hitEdge } from "./timeline-layout";
import { useTimeline } from "./timeline-state";

type Gain = (abs: number) => number;

const AUDIO_TONE: Record<AudioBar["role"], string> = {
  music: "bg-track-music",
  effect: "bg-track-effect",
  clip: "bg-track-clip",
};

const EDGE_KIND = { start: "audio-trim-start", end: "audio-trim-end" } as const;

const HEIGHT = AUDIO_H - BAR_INSET;
const MAX_VOLUME = 2;
const LINE_GRAB_PX = 4;

const NO_GAIN: Gain = () => 1;

export function useDuckGains(spec: TimelineSpec | null, mix: Mix): (id: string) => Gain {
  const gains = useMemo(() => {
    const out = new Map<string, Gain>();
    if (!spec) return out;
    const live = applyMixLevels(spec, mix);
    for (const a of live.audio) {
      if (!a.duck) continue;
      const spans = duckingSpans(live, new Set(a.duck.keys));
      const { level, fadeFrames } = a.duck;
      out.set(a.id, (abs) => duckGain(abs, spans, level, fadeFrames));
    }
    return out;
  }, [spec, mix]);
  return (id) => gains.get(id) ?? NO_GAIN;
}

export function AudioBarView({ bar, ctx, gain }: { bar: AudioBar; ctx: BarContext; gain: Gain }) {
  const { workflow, tool, drag } = ctx;
  const ppf = ctx.viewport.pxPerFrame;
  const audioEnvelope = useTimeline((s) => s.audioEnvelope);
  const left = bar.from * ppf;
  const width = Math.max(2, (bar.to - bar.from) * ppf);
  const nodeId = bar.nodeId;
  const isSelected = !!nodeId && ctx.selected.has(nodeId);
  const keys = nodeId ? envelopeKeysFor(workflow, nodeId, audioEnvelope) : null;
  const target = keys && nodeId ? { nodeId, prop: audioEnvelope, keys } : null;
  const addKey = useKeyframeAdder(target, (clientX) => drag.xToFrame(clientX) - bar.from);
  const lineY = HEIGHT * (1 - Math.min(MAX_VOLUME, bar.volume) / MAX_VOLUME);
  const showLine = audioEnvelope === "volume" && !keys?.length;

  const onLine = (e: React.PointerEvent<HTMLElement>) =>
    showLine &&
    Math.abs(e.clientY - e.currentTarget.getBoundingClientRect().top - lineY) <= LINE_GRAB_PX;

  const hoverCursor = (e: React.PointerEvent<HTMLElement>) => {
    if (hitEdge(e, width)) return "ew-resize";
    return onLine(e) ? "ns-resize" : "";
  };

  const onDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!drag.beginPointer(e)) return;
    if (nodeId && (!isSelected || e.shiftKey)) ctx.select([nodeId], e.shiftKey);
    if (tool === "blade" && bar.bar) {
      commitEdit(splitBar(ctx.splitContext(), bar.bar, Math.round(drag.xToFrame(e.clientX))));
      return;
    }
    const edge = hitEdge(e, width);
    if (!edge && onLine(e)) {
      drag.startVolume(e, bar, HEIGHT);
      return;
    }
    drag.startAudioEdit(e, edge ? EDGE_KIND[edge] : "audio-move", bar, tool === "ripple");
  };

  return (
    <div
      className={cn(
        "absolute top-1 bottom-1 cursor-grab overflow-hidden rounded-md border border-foreground/10",
        AUDIO_TONE[bar.role],
        isSelected && "ring-primary ring-2",
      )}
      style={{ left, width }}
      title={nodeId ? nodeTitle(workflow, nodeId) : undefined}
      onPointerDown={onDown}
      onPointerMove={(e) => {
        if (!drag.isDragging()) e.currentTarget.style.cursor = hoverCursor(e);
      }}
      onPointerDownCapture={addKey}
    >
      <WaveformCanvas
        bar={bar}
        left={left}
        width={width}
        height={HEIGHT}
        view={ctx.viewport}
        fps={ctx.flat.fps}
        gain={gain}
      />
      <span className="pointer-events-none relative truncate px-1.5 text-[11px] font-medium text-foreground drop-shadow">
        {nodeId ? nodeTitle(workflow, nodeId) : ""}
      </span>
      {showLine ? (
        <div
          className="pointer-events-none absolute inset-x-0 h-px bg-keyframe/80"
          style={{ top: lineY }}
        />
      ) : null}
      {target ? (
        <EnvelopeOverlay
          target={target}
          width={width}
          height={HEIGHT}
          toX={(local) => local * ppf}
          toLocal={(abs) => abs - bar.from}
          startKey={drag.startKey}
        />
      ) : null}
    </div>
  );
}
