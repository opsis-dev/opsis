import { findById } from "@opsis/core/lists";
import type { NodeCategory } from "@opsis/core/nodes/define";
import { getNodeDef, nodeNameById } from "@opsis/core/nodes/registry";
import type { Workflow } from "@opsis/core/schema/workflow";
import type { FlatTimeline, VideoBar } from "@opsis/core/timeline/flatten";
import { cn } from "@/lib/utils";
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

const SPEED_CORNER = "speed";

const WRAPPER_SEPARATOR = " · ";

const onSpeedCorner = (e: React.PointerEvent) =>
  e.target instanceof HTMLElement && e.target.dataset.corner === SPEED_CORNER;

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
      data-corner={SPEED_CORNER}
      title="Drag to change the speed"
      className="absolute top-0 right-0 size-2.5 cursor-nesw-resize bg-foreground/30 [clip-path:polygon(0_0,100%_0,100%_100%)] hover:bg-foreground/70"
    />
  );
}

const CAMERA_KEY_PX = 9;

function CameraKeys({ bar, ctx }: { bar: VideoBar; ctx: BarContext }) {
  const ppf = ctx.viewport.pxPerFrame;
  const owners = new Set([bar.itemId, ...bar.chain.map((l) => l.itemId)]);
  const marks = ctx.flat.cameraKeys.filter(
    (m) => owners.has(m.itemId) && m.frame >= bar.from && m.frame <= bar.to,
  );
  return marks.map((m) => (
    <button
      key={m.key}
      type="button"
      aria-label="Camera key"
      title="Camera key. Drag to move it; select the node to set it."
      className="bg-keyframe border-background absolute bottom-1 z-10 rotate-45 cursor-ew-resize rounded-[2px] border"
      style={{
        left: (m.frame - bar.from) * ppf - CAMERA_KEY_PX / 2,
        width: CAMERA_KEY_PX,
        height: CAMERA_KEY_PX,
      }}
      onPointerDown={(e) => ctx.drag.startCameraKey(e, m)}
    />
  ));
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
      title={[title, ...wrappers].join(WRAPPER_SEPARATOR)}
      className={cn(
        toneFor(ctx, bar),
        related && "ring-1 ring-foreground/40",
        TOOL_CURSOR[tool] ?? "cursor-grab",
      )}
      cursorAt={TOOL_CURSOR[tool] ? undefined : (e) => (hitEdge(e, box.width) ? "ew-resize" : "")}
      onPress={onPress}
      envelope={envelope}
      above={
        <>
          <SpeedCorner />
          <CameraKeys bar={bar} ctx={ctx} />
        </>
      }
    >
      <FilmstripCanvas bar={bar} box={box} height={HEIGHT} view={ctx.viewport} fps={ctx.flat.fps} />
      <BarLabel className="block py-0.5 leading-tight">
        {title}
        {wrappers.length ? (
          <span className="text-foreground/60">
            {WRAPPER_SEPARATOR}
            {wrappers.join(WRAPPER_SEPARATOR)}
          </span>
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
