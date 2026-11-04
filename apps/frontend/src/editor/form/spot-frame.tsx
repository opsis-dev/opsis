import type { ComponentProps } from "react";
import { cn } from "@/lib/utils";
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
  const at = timeMs ?? bg?.timeMs ?? 0;
  return (
    <div
      {...props}
      className={cn("bg-muted relative w-full overflow-hidden rounded-md border", className)}
      style={{ aspectRatio: String(bg?.aspect ?? ctx.aspect), ...style }}
    >
      {bg?.still ? (
        <img
          src={bg.src}
          alt=""
          className="pointer-events-none absolute inset-0 size-full object-cover"
        />
      ) : bg ? (
        <video
          src={`${bg.src}#t=${at / 1000}`}
          muted
          preload="auto"
          className="pointer-events-none absolute inset-0 size-full object-cover"
        />
      ) : null}
      {children}
    </div>
  );
}
