import { getNodeDef, nodeNameById } from "@/shared/nodes/registry";
import type { NodeCategory } from "@/shared/nodes/define";
import type { Workflow } from "@/shared/schema/workflow";
import type { FlatTimeline, VideoBar } from "@/shared/timeline/flatten";
import { cn } from "@/renderer/lib/utils";
import { FilmstripCanvas } from "./bar-canvas";
import { bladeSplit, selectOnPress, type BarContext } from "./bar-context";
import { BarFrame, BarLabel } from "./bar-frame";
import { useBarEnvelope } from "./envelope-overlay";
import type { VideoEditKind } from "./timeline-drag";
import { BAR_INSET, VIDEO_H, barBox, hitEdge } from "./timeline-layout";
import { useTimeline, type TimelineTool } from "./timeline-state";
import { findById } from "@/shared/lists";

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

const HEIGHT = VIDEO_H - BAR_INSET;

function toneFor(ctx: BarContext, bar: VideoBar): string {
  const node = findById(ctx.workflow.nodes, bar.nodeId);
  const def = node && getNodeDef(node.type);
  return (def && CATEGORY_TONE[def.category]) ?? DEFAULT_TONE;
}

function editFor(
  tool: TimelineTool,
  edge: "start" | "end" | null,
  corner: boolean,
): [VideoEditKind, boolean] {
  if (corner || (edge && tool === "rate")) return ["speed", false];
  if (tool === "slip") return ["slip", false];
  if (edge && tool === "roll") return [edge === "start" ? "roll-start" : "roll-end", false];
  const ripple = tool === "ripple";
  if (edge) return [edge === "start" ? "trim-start" : "trim-end", ripple];
  return ["move", ripple];
}

const onSpeedCorner = (e: React.PointerEvent) =>
  e.target instanceof HTMLElement && e.target.dataset.corner === "speed";

function barTitles(wf: Workflow, bar: VideoBar) {
  const wrappers = bar.chain
    .slice(1)
    .flatMap((l) => (l.nodeId && l.nodeId !== bar.nodeId ? [nodeNameById(wf, l.nodeId, "")] : []));
  return { title: nodeNameById(wf, bar.nodeId, ""), wrappers };
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
  const isSelected = ctx.selected.has(bar.nodeId);
  const related = !isSelected && bar.chain.some((l) => l.nodeId && ctx.selected.has(l.nodeId));
  const { title, wrappers } = barTitles(ctx.workflow, bar);
  const envelope = useBarEnvelope(ctx, bar.nodeId, videoEnvelope, {
    toX: (local) => ((local - bar.contentStart) / Math.max(1e-6, bar.rate)) * ppf,
    toLocal: (abs) => bar.localAt(abs),
  });

  const onDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!drag.beginPointer(e)) return;
    if (tool === "blade") return bladeSplit(ctx, bar, e.clientX, e.altKey);
    selectOnPress(ctx, bar.nodeId, e.shiftKey);
    const [kind, ripple] = editFor(tool, hitEdge(e, box.width), onSpeedCorner(e));
    drag.startEdit(e, { kind, bar }, ripple);
  };

  return (
    <BarFrame
      ctx={ctx}
      box={box}
      height={HEIGHT}
      selected={isSelected}
      title={[title, ...wrappers].join(" · ")}
      className={cn(
        toneFor(ctx, bar),
        related && "ring-1 ring-foreground/40",
        TOOL_CURSOR[tool] ?? "cursor-grab",
      )}
      cursorAt={TOOL_CURSOR[tool] ? null : (e) => (hitEdge(e, box.width) ? "ew-resize" : "")}
      onPointerDown={onDown}
      envelope={envelope}
      above={<SpeedCorner />}
    >
      <FilmstripCanvas
        bar={bar}
        left={box.left}
        width={box.width}
        height={HEIGHT}
        view={ctx.viewport}
        fps={ctx.flat.fps}
      />
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
