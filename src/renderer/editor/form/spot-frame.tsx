import type { ComponentProps } from "react";
import { cn } from "@/renderer/lib/utils";
import type { FormContext } from "./types";

export function SpotFrame({
  ctx,
  timeMs,
  className,
  style,
  children,
  ...props
}: ComponentProps<"div"> & { ctx: FormContext; timeMs?: number }) {
  const bg = ctx.spotBackground;
  return (
    <div
      {...props}
      className={cn("bg-muted relative w-full overflow-hidden rounded-md border", className)}
      style={{ aspectRatio: String(ctx.aspect), ...style }}
    >
      {bg ? (
        <video
          src={`${bg.src}#t=${(timeMs || bg.timeMs) / 1000}`}
          muted
          preload="auto"
          className="pointer-events-none absolute inset-0 size-full object-cover"
        />
      ) : null}
      {children}
    </div>
  );
}
