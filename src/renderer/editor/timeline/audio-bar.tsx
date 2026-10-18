import { useMemo } from "react";
import { duckGain, duckingSpans } from "@/shared/mix/duck";
import { applyMixLevels } from "@/shared/mix/levels";
import type { Mix } from "@/shared/mix/types";
import type { TimelineSpec } from "@/shared/schema/timeline";
import { flattenSpec, type AudioBar } from "@/shared/timeline/flatten";
import { MAX_VOLUME } from "@/shared/nodes/params";
import { cn } from "@/renderer/lib/utils";
import { nodeNameById } from "@/shared/nodes/registry";
import { WaveformCanvas, type Gain } from "./bar-canvas";
import { bladeSplit, selectOnPress, type BarContext } from "./bar-context";
import { BarFrame, BarLabel } from "./bar-frame";
import { useBarEnvelope } from "./envelope-overlay";
import { AUDIO_H, BAR_INSET, barBox, hitEdge } from "./timeline-layout";
import { useTimeline } from "./timeline-state";

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
    if (!live.audio.some((a) => a.duck)) return out;
    const flat = flattenSpec(live);
    for (const a of live.audio) {
      if (!a.duck) continue;
      const spans = duckingSpans(live, new Set(a.duck.keys), flat);
      const { level, fadeFrames } = a.duck;
      out.set(a.id, (abs) => duckGain(abs, spans, level, fadeFrames));
    }
    return out;
  }, [spec, mix]);
  return (id) => gains.get(id) ?? NO_GAIN;
}

function audioHit(e: React.PointerEvent<HTMLElement>, width: number, lineY: number | null): Hit {
  const edge = hitEdge(e, width);
  if (edge) return edge;
  if (lineY === null) return "body";
  const y = e.clientY - e.currentTarget.getBoundingClientRect().top;
  return Math.abs(y - lineY) <= LINE_GRAB_PX ? "line" : "body";
}

const volumeLineY = (volume: number) => HEIGHT * (1 - Math.min(MAX_VOLUME, volume) / MAX_VOLUME);

function VolumeLine({ y }: { y: number }) {
  return (
    <div
      className="pointer-events-none absolute inset-x-0 h-px bg-keyframe/80"
      style={{ top: y }}
    />
  );
}

export function AudioBarView({ bar, ctx, gain }: { bar: AudioBar; ctx: BarContext; gain: Gain }) {
  const { tool, drag } = ctx;
  const ppf = ctx.viewport.pxPerFrame;
  const audioEnvelope = useTimeline((s) => s.audioEnvelope);
  const box = barBox(bar.from, bar.to, ppf);
  const nodeId = bar.nodeId;
  const title = nodeId ? nodeNameById(ctx.workflow, nodeId, "") : "";
  const envelope = useBarEnvelope(ctx, nodeId, audioEnvelope, {
    toX: (local) => local * ppf,
    toLocal: (abs) => abs - bar.from,
  });
  const showLine = audioEnvelope === "volume" && !envelope.target?.keys.length;
  const lineY = showLine ? volumeLineY(bar.volume) : null;
  const hitAt = (e: React.PointerEvent<HTMLElement>) => audioHit(e, box.width, lineY);

  const onDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!drag.beginPointer(e)) return;
    if (tool === "blade" && bar.bar) return bladeSplit(ctx, bar.bar, e.clientX, e.altKey);
    if (nodeId) selectOnPress(ctx, nodeId, e.shiftKey);
    const hit = hitAt(e);
    if (hit === "line") drag.startVolume(e, bar, HEIGHT);
    else
      drag.startEdit(
        e,
        { kind: hit === "body" ? "audio-move" : EDGE_KIND[hit], bar },
        tool === "ripple",
      );
  };

  return (
    <BarFrame
      ctx={ctx}
      box={box}
      height={HEIGHT}
      selected={!!nodeId && ctx.selected.has(nodeId)}
      title={title || undefined}
      className={cn("cursor-grab", AUDIO_TONE[bar.role])}
      cursorAt={(e) => HIT_CURSOR[hitAt(e)]}
      onPointerDown={onDown}
      envelope={envelope}
    >
      <WaveformCanvas
        bar={bar}
        left={box.left}
        width={box.width}
        height={HEIGHT}
        view={ctx.viewport}
        fps={ctx.flat.fps}
        gain={gain}
      />
      <BarLabel>{title}</BarLabel>
      {lineY === null ? null : <VolumeLine y={lineY} />}
    </BarFrame>
  );
}
