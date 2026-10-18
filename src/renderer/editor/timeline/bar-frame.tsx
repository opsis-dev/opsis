import { useState } from "react";
import { cn } from "@/renderer/lib/utils";
import type { BarContext } from "./bar-context";
import { EnvelopeOverlay, type BarEnvelope } from "./envelope-overlay";
import { BAR_INSET, type BarBox } from "./timeline-layout";

export function BarFrame({
  ctx,
  box,
  height,
  selected,
  title,
  className,
  cursorAt,
  onPointerDown,
  envelope,
  above,
  children,
}: {
  ctx: BarContext;
  box: BarBox;
  height: number;
  selected: boolean;
  title: string | undefined;
  className: string;
  cursorAt: ((e: React.PointerEvent<HTMLDivElement>) => string) | null;
  onPointerDown: (e: React.PointerEvent<HTMLDivElement>) => void;
  envelope: BarEnvelope;
  above?: React.ReactNode;
  children: React.ReactNode;
}) {
  const [hoverCursor, setHoverCursor] = useState("");
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
