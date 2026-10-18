import { cn } from "@/renderer/lib/utils";
import { rulerStep } from "@/shared/timeline/snap";
import { HEADER_W, RULER_H } from "./timeline-layout";
import { useFormatTime, useLoopRange } from "./timeline-state";

const MAX_TICKS = 600;

function ticksIn(left: number, width: number, ppf: number, fps: number) {
  const { step, minor } = rulerStep(ppf, fps);
  const first = Math.floor(left / ppf / minor) * minor;
  const last = (left + width) / ppf;
  const ticks: Array<{ f: number; major: boolean }> = [];
  for (let f = first; f <= last && ticks.length < MAX_TICKS; f += minor)
    ticks.push({ f, major: Math.abs(f / step - Math.round(f / step)) < 1e-6 });
  return ticks;
}

export function Ruler({
  left,
  width,
  contentW,
  ppf,
  fps,
  duration,
  onPointerDown,
}: {
  left: number;
  width: number;
  contentW: number;
  ppf: number;
  fps: number;
  duration: number;
  onPointerDown: (e: React.PointerEvent) => void;
}) {
  const loop = useLoopRange(duration);
  const timeAt = useFormatTime(fps);
  return (
    <div
      className="bg-sidebar sticky top-0 z-30 flex border-b"
      style={{ height: RULER_H, width: HEADER_W + contentW }}
    >
      <div
        className="bg-sidebar sticky left-0 z-10 shrink-0 border-r"
        style={{ width: HEADER_W }}
      />
      <div className="relative flex-1 cursor-text" onPointerDown={onPointerDown}>
        {loop ? (
          <div
            className="bg-primary/15 absolute inset-y-0"
            style={{ left: loop.from * ppf, width: (loop.to - loop.from) * ppf }}
          />
        ) : null}
        {ticksIn(left, width, ppf, fps).map(({ f, major }) => (
          <div
            key={f}
            className={cn(
              "absolute bottom-0 w-px",
              major ? "bg-muted-foreground/60 h-2.5" : "bg-muted-foreground/30 h-1.5",
            )}
            style={{ left: f * ppf }}
          >
            {major ? (
              <span className="text-muted-foreground absolute bottom-2.5 left-1 text-[10px] whitespace-nowrap tabular-nums">
                {timeAt(f)}
              </span>
            ) : null}
          </div>
        ))}
        <div className="bg-border absolute inset-y-0" style={{ left: duration * ppf, width: 1 }} />
      </div>
    </div>
  );
}
