import type { AudioBar } from "@/generated/bindings";
import { cn } from "@/lib/utils";
import { nodeNameById } from "@/store/catalog";
import { WaveformCanvas, type Gain } from "./bar-canvas";
import type { BarContext } from "./bar-context";
import { BarFrame, BarLabel } from "./bar-frame";
import { envelopeY } from "./envelope";
import { useBarEnvelope } from "./envelope-overlay";
import { AUDIO_H, barBox, barHeight, hitEdge } from "./timeline-layout";
import { useTimeline } from "./timeline-state";

const AUDIO_TONE: Record<AudioBar["role"], string> = {
  music: "bg-track-music",
  effect: "bg-track-effect",
  clip: "bg-track-clip",
};

const EDGE_KIND = { start: "audio-trim-start", end: "audio-trim-end" } as const;

const HEIGHT = barHeight(AUDIO_H);
const LINE_GRAB_PX = 4;

type Hit = "start" | "end" | "line" | "body";

const HIT_CURSOR: Record<Hit, string> = {
  start: "ew-resize",
  end: "ew-resize",
  line: "ns-resize",
  body: "",
};

function audioHit(e: React.PointerEvent<HTMLElement>, width: number, lineY: number | null): Hit {
  const edge = hitEdge(e, width);
  if (edge) return edge;
  if (lineY === null) return "body";
  const y = e.clientY - e.currentTarget.getBoundingClientRect().top;
  return Math.abs(y - lineY) <= LINE_GRAB_PX ? "line" : "body";
}

function HitTicks({ hits, from, ppf }: { hits: number[]; from: number; ppf: number }) {
  return hits.map((frame, i) => (
    <div
      key={i}
      className="pointer-events-none absolute inset-y-1/4 w-px bg-foreground/60"
      style={{ left: (frame - from) * ppf }}
    />
  ));
}

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
  const title = nodeId ? nodeNameById(ctx.workflow.nodes, nodeId, "") : "";
  const envelope = useBarEnvelope(ctx, nodeId, audioEnvelope, {
    toX: (local) => local * ppf,
    toLocal: (abs) => abs - bar.from,
    scale: audioEnvelope === "volume" ? bar.volume : 1,
  });
  const showLine = audioEnvelope === "volume" && !envelope.target?.keys.length;
  const lineY = showLine ? envelopeY("volume", bar.volume, HEIGHT) : null;
  const hitAt = (e: React.PointerEvent<HTMLElement>): Hit => {
    const hit = audioHit(e, box.width, lineY);
    return bar.hits && hit !== "line" ? "body" : hit;
  };

  const onPress = (e: React.PointerEvent<HTMLDivElement>) => {
    const hit = hitAt(e);
    if (hit === "line") drag.startVolume(e, bar, HEIGHT);
    else if (!bar.hits)
      drag.startEdit(
        e,
        { domain: "audio", kind: hit === "body" ? "audio-move" : EDGE_KIND[hit], bar },
        tool === "ripple",
      );
  };

  return (
    <BarFrame
      ctx={ctx}
      span={bar}
      box={box}
      height={HEIGHT}
      nodeId={nodeId}
      split={bar.bar}
      title={title || undefined}
      className={cn(!bar.hits && "cursor-grab", AUDIO_TONE[bar.role])}
      cursorAt={(e) => HIT_CURSOR[hitAt(e)]}
      onPress={onPress}
      envelope={envelope}
    >
      {bar.hits ? (
        <HitTicks hits={bar.hits} from={bar.from} ppf={ppf} />
      ) : (
        <WaveformCanvas
          bar={bar}
          box={box}
          height={HEIGHT}
          view={ctx.viewport}
          fps={ctx.flat.fps}
          gain={gain}
        />
      )}
      <BarLabel>{title}</BarLabel>
      {lineY === null ? null : <VolumeLine y={lineY} />}
    </BarFrame>
  );
}
