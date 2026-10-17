import { getNodeDef } from "@/shared/nodes/registry";
import type { NodeCategory } from "@/shared/nodes/define";
import { splitBar } from "@/shared/timeline/edits";
import type { VideoBar } from "@/shared/timeline/flatten";
import { cn } from "@/renderer/lib/utils";
import { FilmstripCanvas } from "./bar-canvas";
import { nodeTitle, type BarContext } from "./bar-context";
import { BarFrame, BarLabel } from "./bar-frame";
import { EnvelopeOverlay, useBarEnvelope } from "./envelope-overlay";
import { commitEdit } from "./timeline-actions";
import type { VideoEditKind } from "./timeline-drag";
import { BAR_INSET, VIDEO_H, barBox, hitEdge } from "./timeline-layout";
import { useTimeline, type TimelineTool } from "./timeline-state";

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
  const node = ctx.workflow.nodes.find((n) => n.id === bar.nodeId);
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

export function VideoBarView({ bar, ctx }: { bar: VideoBar; ctx: BarContext }) {
  const { workflow, tool, drag } = ctx;
  const ppf = ctx.viewport.pxPerFrame;
  const videoEnvelope = useTimeline((s) => s.videoEnvelope);
  const { left, width } = barBox(bar.from, bar.to, ppf);
  const isSelected = ctx.selected.has(bar.nodeId);
  const related = !isSelected && bar.chain.some((l) => l.nodeId && ctx.selected.has(l.nodeId));
  const title = nodeTitle(workflow, bar.nodeId);
  const wrappers = bar.chain
    .slice(1)
    .flatMap((l) => (l.nodeId && l.nodeId !== bar.nodeId ? [nodeTitle(workflow, l.nodeId)] : []));
  const { target, addKey } = useBarEnvelope(workflow, bar.nodeId, videoEnvelope, (clientX) =>
    bar.localAt(drag.xToFrame(clientX)),
  );

  const onDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!drag.beginPointer(e)) return;
    if (tool === "blade") {
      const at = drag.snapAt(drag.xToFrame(e.clientX), e.altKey);
      commitEdit(splitBar(ctx.splitContext(), bar, Math.round(at)));
      return;
    }
    if (!isSelected || e.shiftKey) ctx.select([bar.nodeId], e.shiftKey && tool === "select");
    const [kind, ripple] = editFor(tool, hitEdge(e, width), onSpeedCorner(e));
    drag.startVideoEdit(e, kind, bar, ripple);
  };

  return (
    <BarFrame
      ctx={ctx}
      left={left}
      width={width}
      selected={isSelected}
      title={[title, ...wrappers].join(" · ")}
      className={cn(
        toneFor(ctx, bar),
        related && "ring-1 ring-foreground/40",
        TOOL_CURSOR[tool] ?? "cursor-grab",
      )}
      cursorAt={TOOL_CURSOR[tool] ? null : (e) => (hitEdge(e, width) ? "ew-resize" : "")}
      onPointerDown={onDown}
      onAddKey={addKey}
    >
      <FilmstripCanvas
        bar={bar}
        left={left}
        width={width}
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
      {ctx.flat.marks
        .filter((m) => m.barKey === bar.key && m.kind !== "chapter")
        .map((m) => (
          <div
            key={m.key}
            title={m.label}
            className="absolute bottom-0 h-2 w-0.5 bg-foreground/80 hover:h-3"
            style={{ left: m.frame * ppf - left }}
          />
        ))}
      {target ? (
        <EnvelopeOverlay
          target={target}
          width={width}
          height={HEIGHT}
          toX={(local) => ((local - bar.contentStart) / Math.max(1e-6, bar.rate)) * ppf}
          toLocal={(abs) => bar.localAt(abs)}
          startKey={drag.startKey}
        />
      ) : null}
      <div
        data-corner="speed"
        title="Drag to change the speed"
        className="absolute top-0 right-0 size-2.5 cursor-nesw-resize bg-foreground/30 [clip-path:polygon(0_0,100%_0,100%_100%)] hover:bg-foreground/70"
      />
    </BarFrame>
  );
}
