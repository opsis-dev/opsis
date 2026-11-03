import { frameAt } from "@opsis/video/terminal/frames";
import { screenSize } from "@opsis/video/terminal/view";
import type { ComponentProps } from "react";
import { TerminalPicture } from "@/app/terminal-picture";
import { PLAYER_FPS, PLAYER_LOOK, useTerminalFrames } from "@/editor/terminal-player";
import type { TerminalTheme } from "@/generated/bindings";
import { cn } from "@/lib/utils";
import type { FormContext } from "./types";

function useTerminalSpot(cast: string | null, theme: TerminalTheme | null, timeMs: number) {
  const { frames } = useTerminalFrames(cast, PLAYER_FPS, theme);
  const frame = frames ? frameAt(frames, timeMs / 1000) : undefined;
  if (!frames || !frame) return null;
  const size = screenSize(PLAYER_LOOK, frame.cols, frame.rows);
  return { frames, frame, aspect: size.width / size.height };
}

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
  const terminal = useTerminalSpot(bg?.terminal ? bg.src : null, bg?.terminal ?? null, at);
  return (
    <div
      {...props}
      className={cn("bg-muted relative w-full overflow-hidden rounded-md border", className)}
      style={{ aspectRatio: String(terminal?.aspect ?? ctx.aspect), ...style }}
    >
      {terminal ? (
        <TerminalPicture
          frames={terminal.frames}
          frame={terminal.frame}
          look={PLAYER_LOOK}
          cursorOn={false}
        />
      ) : bg?.still ? (
        <img
          src={bg.src}
          alt=""
          className="pointer-events-none absolute inset-0 size-full object-cover"
        />
      ) : bg && !bg.terminal ? (
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
