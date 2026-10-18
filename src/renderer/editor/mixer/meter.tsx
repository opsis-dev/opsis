import { cn } from "@/renderer/lib/utils";
import { clamp01 } from "@/shared/math";
import { usePlayback } from "../../store/playback";
import { levelDb, signedDb } from "./audio-format";
import { levelAt, reductionAt, type Level, type Levels, type Reduction } from "./meter-levels";

const METER_MIN = -60;
const METER_MAX = 6;
const MAX_REDUCTION_DB = 24;
const CLIP_DB = -0.5;

const toPct = (level: number) => clamp01((level - METER_MIN) / (METER_MAX - METER_MIN)) * 100;

function Meter({ level, reduction }: { level: Level; reduction?: number }) {
  const peak = level?.peak ?? METER_MIN;
  const rms = level?.rms ?? METER_MIN;
  return (
    <div className="flex h-full gap-0.5">
      <div
        className="bg-muted relative h-full w-2.5 overflow-hidden rounded-sm"
        title={
          level
            ? `Peak ${levelDb(level.peak)} dB, average ${levelDb(level.rms)} dB`
            : "No level yet"
        }
      >
        <div
          className="from-meter-low via-meter-mid to-meter-high absolute inset-x-0 bottom-0 bg-gradient-to-t opacity-40"
          style={{ height: `${toPct(peak)}%`, backgroundSize: "100% 100%" }}
        />
        <div
          className="bg-meter-low absolute inset-x-0 bottom-0"
          style={{ height: `${toPct(rms)}%` }}
        />
        {level && level.hold > METER_MIN ? (
          <div
            className={cn(
              "absolute inset-x-0 h-px",
              level.hold > CLIP_DB ? "bg-meter-high" : "bg-foreground",
            )}
            style={{ bottom: `${toPct(level.hold)}%` }}
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
          title={`Compressor lowers by ${reduction.toFixed(1)} dB`}
        >
          <div
            className="bg-keyframe absolute inset-x-0 top-0"
            style={{ height: `${Math.min(100, (reduction / MAX_REDUCTION_DB) * 100)}%` }}
          />
        </div>
      ) : null}
    </div>
  );
}

export function LiveMeter({
  levels,
  reduction,
  compOn,
  holdFrames,
}: {
  levels: Levels;
  reduction: Reduction;
  compOn: boolean;
  holdFrames: number;
}) {
  const frame = usePlayback((s) => s.frame);
  const lowered = compOn ? reductionAt(reduction, frame) : null;
  return <Meter level={levelAt(levels, frame, holdFrames)} reduction={lowered ?? undefined} />;
}

export function ReductionReadout({ reduction }: { reduction: Reduction }) {
  const frame = usePlayback((s) => s.frame);
  const lowered = reductionAt(reduction, frame);
  return lowered === null ? null : <span className="tabular-nums">{signedDb(-lowered)}</span>;
}
