import { cn } from "@/renderer/lib/utils";
import type { BarContext } from "./bar-context";

export function BarFrame({
  ctx,
  left,
  width,
  selected,
  title,
  className,
  cursorAt,
  onPointerDown,
  onAddKey,
  children,
}: {
  ctx: BarContext;
  left: number;
  width: number;
  selected: boolean;
  title: string | undefined;
  className: string;
  cursorAt: ((e: React.PointerEvent<HTMLDivElement>) => string) | null;
  onPointerDown: (e: React.PointerEvent<HTMLDivElement>) => void;
  onAddKey: (e: React.PointerEvent) => void;
  children: React.ReactNode;
}) {
  return (
    <div
      className={cn(
        "absolute top-1 bottom-1 overflow-hidden rounded-md border border-foreground/10",
        className,
        selected && "ring-primary ring-2",
      )}
      style={{ left, width }}
      title={title}
      onPointerDown={onPointerDown}
      onPointerMove={(e) => {
        if (cursorAt && !ctx.drag.isDragging()) e.currentTarget.style.cursor = cursorAt(e);
      }}
      onPointerDownCapture={onAddKey}
    >
      {children}
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
