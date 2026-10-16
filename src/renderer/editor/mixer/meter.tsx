import { cn } from "@/renderer/lib/utils";
import { clamp } from "@/shared/math";

export const METER_MIN = -60;
export const METER_MAX = 6;

const toPct = (db: number) => clamp((db - METER_MIN) / (METER_MAX - METER_MIN), 0, 1) * 100;

export function Meter({
  peak,
  rms,
  hold,
  reduction,
  className,
  dim,
}: {
  peak: number | null;
  rms: number | null;
  hold: number | null;
  reduction?: number | null;
  className?: string;
  dim?: boolean;
}) {
  return (
    <div className={cn("flex h-full gap-0.5", dim && "opacity-50", className)}>
      <div
        className="bg-muted relative h-full w-2.5 overflow-hidden rounded-sm"
        title={
          peak === null
            ? "No level yet"
            : `Peak ${peak.toFixed(1)} dB, average ${rms?.toFixed(1)} dB`
        }
      >
        <div
          className="from-meter-low via-meter-mid to-meter-high absolute inset-x-0 bottom-0 bg-gradient-to-t opacity-40"
          style={{ height: `${toPct(peak ?? METER_MIN)}%`, backgroundSize: "100% 100%" }}
        />
        <div
          className="bg-meter-low absolute inset-x-0 bottom-0"
          style={{ height: `${toPct(rms ?? METER_MIN)}%` }}
        />
        {hold !== null && hold > METER_MIN ? (
          <div
            className={cn(
              "absolute inset-x-0 h-px",
              hold > -0.5 ? "bg-meter-high" : "bg-foreground",
            )}
            style={{ bottom: `${toPct(hold)}%` }}
          />
        ) : null}
        <div
          className="bg-foreground/40 absolute inset-x-0 h-px"
          style={{ bottom: `${toPct(0)}%` }}
        />
      </div>
      {reduction !== undefined ? (
        <div
          className="bg-muted relative h-full w-1 overflow-hidden rounded-sm"
          title={`Compressor lowers by ${(reduction ?? 0).toFixed(1)} dB`}
        >
          <div
            className="bg-keyframe absolute inset-x-0 top-0"
            style={{ height: `${Math.min(100, ((reduction ?? 0) / 24) * 100)}%` }}
          />
        </div>
      ) : null}
    </div>
  );
}

export function levelAt(
  levels: { peak: number[]; rms: number[] } | undefined,
  frame: number,
  holdFrames: number,
): { peak: number | null; rms: number | null; hold: number | null } {
  if (!levels || levels.peak.length === 0) return { peak: null, rms: null, hold: null };
  const f = clamp(Math.round(frame), 0, levels.peak.length - 1);
  let hold = -120;
  for (let i = Math.max(0, f - holdFrames); i <= f; i++) hold = Math.max(hold, levels.peak[i]);
  return { peak: levels.peak[f], rms: levels.rms[f], hold };
}
