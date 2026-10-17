import { useMemo } from "react";
import { duckGain, duckingSpans } from "@/shared/compile/duck";
import { applyMixLevels } from "@/shared/mix/levels";
import type { Mix } from "@/shared/mix/types";
import type { TimelineSpec } from "@/shared/schema/timeline";
import type { AudioBar } from "@/shared/timeline/flatten";
import { MAX_VOLUME } from "@/shared/nodes/params";
import { cn } from "@/renderer/lib/utils";
import { WaveformCanvas } from "./bar-canvas";
import { bladeSplit, nodeTitle, selectOnPress, type BarContext } from "./bar-context";
import { BarFrame, BarLabel } from "./bar-frame";
import { EnvelopeOverlay, useBarEnvelope } from "./envelope-overlay";
import { AUDIO_H, BAR_INSET, barBox, hitEdge } from "./timeline-layout";
import { useTimeline } from "./timeline-state";

type Gain = (abs: number) => number;

const AUDIO_TONE: Record<AudioBar["role"], string> = {
  music: "bg-track-music",
  effect: "bg-track-effect",
  clip: "bg-track-clip",
};

const EDGE_KIND = { start: "audio-trim-start", end: "audio-trim-end" } as const;

const HEIGHT = AUDIO_H - BAR_INSET;
const LINE_GRAB_PX = 4;

const NO_GAIN: Gain = () => 1;

type Hit = "start" | "end" | "line" | "body";

const HIT_CURSOR: Record<Hit, string> = {
  start: "ew-resize",
  end: "ew-resize",
  line: "ns-resize",
  body: "",
};

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
  const { left, width } = barBox(bar.from, bar.to, ppf);
  const nodeId = bar.nodeId;
  const isSelected = !!nodeId && ctx.selected.has(nodeId);
  const title = nodeId ? nodeTitle(workflow, nodeId) : "";
  const { target, addKey } = useBarEnvelope(
    workflow,
    nodeId,
    audioEnvelope,
    (clientX) => drag.xToFrame(clientX) - bar.from,
  );
  const lineY = HEIGHT * (1 - Math.min(MAX_VOLUME, bar.volume) / MAX_VOLUME);
  const showLine = audioEnvelope === "volume" && !target?.keys.length;

  const hitAt = (e: React.PointerEvent<HTMLElement>): Hit => {
    const edge = hitEdge(e, width);
    if (edge) return edge;
    const y = e.clientY - e.currentTarget.getBoundingClientRect().top;
    return showLine && Math.abs(y - lineY) <= LINE_GRAB_PX ? "line" : "body";
  };

  const onDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!drag.beginPointer(e)) return;
    if (tool === "blade" && bar.bar) return bladeSplit(ctx, bar.bar, e.clientX, e.altKey);
    if (nodeId) selectOnPress(ctx, nodeId, e.shiftKey);
    const hit = hitAt(e);
    if (hit === "line") drag.startVolume(e, bar, HEIGHT);
    else
      drag.startAudioEdit(
        e,
        hit === "body" ? "audio-move" : EDGE_KIND[hit],
        bar,
        tool === "ripple",
      );
  };

  return (
    <BarFrame
      ctx={ctx}
      left={left}
      width={width}
      selected={isSelected}
      title={title || undefined}
      className={cn("cursor-grab", AUDIO_TONE[bar.role])}
      cursorAt={(e) => HIT_CURSOR[hitAt(e)]}
      onPointerDown={onDown}
      onAddKey={addKey}
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
      <BarLabel>{title}</BarLabel>
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
    </BarFrame>
  );
}
