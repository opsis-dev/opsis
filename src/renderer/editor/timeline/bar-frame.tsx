import { cn } from "@/renderer/lib/utils";
import type { BarContext } from "./bar-context";
import { EnvelopeOverlay, type BarEnvelope } from "./envelope-overlay";

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
  box: { left: number; width: number };
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
  return (
    <div
      className={cn(
        "absolute top-1 bottom-1 overflow-hidden rounded-md border border-foreground/10",
        className,
        selected && "ring-primary ring-2",
      )}
      style={{ left: box.left, width: box.width }}
      title={title}
      onPointerDown={onPointerDown}
      onPointerMove={(e) => {
        if (cursorAt && !ctx.drag.isDragging()) e.currentTarget.style.cursor = cursorAt(e);
      }}
      onPointerDownCapture={envelope.addKey}
    >
      {children}
      {envelope.target ? (
        <EnvelopeOverlay
          target={envelope.target}
          width={box.width}
          height={height}
          toX={envelope.toX}
          toLocal={envelope.toLocal}
          startKey={ctx.drag.startKey}
        />
      ) : null}
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
