import { useState } from "react";
import type { VideoBar } from "@/generated/bindings";
import { cn } from "@/lib/utils";
import type { BarContext } from "./bar-context";
import { EnvelopeOverlay, type BarEnvelope } from "./envelope-overlay";
import { BAR_INSET, type BarBox } from "./timeline-layout";

function bladeSplit(ctx: BarContext, bar: VideoBar, clientX: number, alt: boolean): void {
  const at = ctx.drag.snapAt(ctx.drag.xToFrame(clientX), alt);
  ctx.splitAt(bar, Math.round(at));
}

function selectOnPress(ctx: BarContext, nodeId: string, shift: boolean): void {
  if (!ctx.selected.has(nodeId) || shift) ctx.select([nodeId], shift && ctx.tool === "select");
}

export function BarFrame({
  ctx,
  box,
  height,
  nodeId,
  split,
  title,
  className,
  cursorAt,
  onPress,
  envelope,
  above,
  children,
}: {
  ctx: BarContext;
  box: BarBox;
  height: number;
  nodeId?: string;
  split?: VideoBar;
  title?: string;
  className: string;
  cursorAt?: (e: React.PointerEvent<HTMLDivElement>) => string;
  onPress: (e: React.PointerEvent<HTMLDivElement>) => void;
  envelope: BarEnvelope;
  above?: React.ReactNode;
  children: React.ReactNode;
}) {
  const [hoverCursor, setHoverCursor] = useState("");
  const selected = !!nodeId && ctx.selected.has(nodeId);
  const onPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!ctx.drag.beginPointer(e)) return;
    if (ctx.tool === "blade" && split) return bladeSplit(ctx, split, e.clientX, e.altKey);
    if (nodeId) selectOnPress(ctx, nodeId, e.shiftKey);
    onPress(e);
  };
  return (
    <div
      className={cn(
        "absolute overflow-hidden rounded-md border border-foreground/10",
        className,
        selected && "ring-primary ring-2",
      )}
      style={{
        left: box.left,
        width: box.width,
        top: BAR_INSET / 2,
        bottom: BAR_INSET / 2,
        cursor: (cursorAt && hoverCursor) || undefined,
      }}
      title={title}
      onPointerDown={onPointerDown}
      onPointerMove={(e) => {
        if (cursorAt && !ctx.drag.isDragging()) setHoverCursor(cursorAt(e));
      }}
      onPointerDownCapture={envelope.addKey}
    >
      {children}
      <EnvelopeOverlay
        envelope={envelope}
        width={box.width}
        height={height}
        startKey={ctx.drag.startKey}
      />
      {above}
    </div>
  );
}

export function BarLabel({
  className,
  children,
}: {
  className?: string;
  children: React.ReactNode;
}) {
  return (
    <span
      className={cn(
        "pointer-events-none relative truncate px-1.5 text-[11px] font-medium text-foreground drop-shadow",
        className,
      )}
    >
      {children}
    </span>
  );
}
