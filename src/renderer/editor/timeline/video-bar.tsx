import { cn } from "@/renderer/lib/utils";
import { findById } from "@/shared/lists";
import type { NodeCategory } from "@/shared/nodes/define";
import { getNodeDef, nodeNameById } from "@/shared/nodes/registry";
import type { Workflow } from "@/shared/schema/workflow";
import type { FlatTimeline, VideoBar } from "@/shared/timeline/flatten";
import { FilmstripCanvas } from "./bar-canvas";
import type { BarContext } from "./bar-context";
import { BarFrame, BarLabel } from "./bar-frame";
import { useBarEnvelope } from "./envelope-overlay";
import type { VideoEditKind } from "./timeline-drag";
import { VIDEO_H, barBox, barHeight, hitEdge } from "./timeline-layout";
import { useTimeline } from "./timeline-state";
import type { TimelineTool } from "./timeline-tools";

const DEFAULT_TONE = "bg-track-video";

const CATEGORY_TONE: Partial<Record<NodeCategory, string>> = {
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

const TOOL_CURSOR: Partial<Record<TimelineTool, string>> = {
  blade: "cursor-crosshair",
  slip: "cursor-ew-resize",
};

const HEIGHT = barHeight(VIDEO_H);

function toneFor(ctx: BarContext, bar: VideoBar): string {
  const node = findById(ctx.workflow.nodes, bar.nodeId);
  const def = node && getNodeDef(node.type);
  return (def && CATEGORY_TONE[def.category]) ?? DEFAULT_TONE;
}

function editFor(
  tool: TimelineTool,
  edge: "start" | "end" | null,
  corner: boolean,
): { kind: VideoEditKind; ripple: boolean } {
  if (corner || (edge && tool === "rate")) return { kind: "speed", ripple: false };
  if (tool === "slip") return { kind: "slip", ripple: false };
  if (edge && tool === "roll")
    return { kind: edge === "start" ? "roll-start" : "roll-end", ripple: false };
  const ripple = tool === "ripple";
  if (edge) return { kind: edge === "start" ? "trim-start" : "trim-end", ripple };
  return { kind: "move", ripple };
}

const onSpeedCorner = (e: React.PointerEvent) =>
  e.target instanceof HTMLElement && e.target.dataset.corner === "speed";

function barTitles(wf: Workflow, bar: VideoBar) {
  const wrappers = bar.chain
    .slice(1)
    .flatMap((l) =>
      l.nodeId && l.nodeId !== bar.nodeId ? [nodeNameById(wf.nodes, l.nodeId, "")] : [],
    );
  return { title: nodeNameById(wf.nodes, bar.nodeId, ""), wrappers };
}

function ActionTicks({
  marks,
  left,
  ppf,
}: {
  marks: FlatTimeline["marks"];
  left: number;
  ppf: number;
}) {
  return marks.map((m) => (
    <div
      key={m.key}
      title={m.label}
      className="absolute bottom-0 h-2 w-0.5 bg-foreground/80 hover:h-3"
      style={{ left: m.frame * ppf - left }}
    />
  ));
}

function SpeedCorner() {
  return (
    <div
      data-corner="speed"
      title="Drag to change the speed"
      className="absolute top-0 right-0 size-2.5 cursor-nesw-resize bg-foreground/30 [clip-path:polygon(0_0,100%_0,100%_100%)] hover:bg-foreground/70"
    />
  );
}

export function VideoBarView({ bar, ctx }: { bar: VideoBar; ctx: BarContext }) {
  const { tool, drag } = ctx;
  const ppf = ctx.viewport.pxPerFrame;
  const videoEnvelope = useTimeline((s) => s.videoEnvelope);
  const box = barBox(bar.from, bar.to, ppf);
  const related =
    !ctx.selected.has(bar.nodeId) && bar.chain.some((l) => l.nodeId && ctx.selected.has(l.nodeId));
  const { title, wrappers } = barTitles(ctx.workflow, bar);
  const envelope = useBarEnvelope(ctx, bar.nodeId, videoEnvelope, {
    toX: (local) => ((local - bar.contentStart) / bar.rate) * ppf,
    toLocal: (abs) => bar.localAt(abs),
  });

  const onPress = (e: React.PointerEvent<HTMLDivElement>) => {
    const { kind, ripple } = editFor(tool, hitEdge(e, box.width), onSpeedCorner(e));
    drag.startEdit(e, { domain: "video", kind, bar }, ripple);
  };

  return (
    <BarFrame
      ctx={ctx}
      box={box}
      height={HEIGHT}
      nodeId={bar.nodeId}
      split={bar}
      title={[title, ...wrappers].join(" · ")}
      className={cn(
        toneFor(ctx, bar),
        related && "ring-1 ring-foreground/40",
        TOOL_CURSOR[tool] ?? "cursor-grab",
      )}
      cursorAt={TOOL_CURSOR[tool] ? null : (e) => (hitEdge(e, box.width) ? "ew-resize" : "")}
      onPress={onPress}
      envelope={envelope}
      above={<SpeedCorner />}
    >
      <FilmstripCanvas bar={bar} box={box} height={HEIGHT} view={ctx.viewport} fps={ctx.flat.fps} />
      <BarLabel className="block py-0.5 leading-tight">
        {title}
        {wrappers.length ? (
          <span className="text-foreground/60"> · {wrappers.join(" · ")}</span>
        ) : null}
      </BarLabel>
      <ActionTicks
        marks={ctx.flat.marks.filter((m) => m.barKey === bar.key && m.kind !== "chapter")}
        left={box.left}
        ppf={ppf}
      />
    </BarFrame>
  );
}
