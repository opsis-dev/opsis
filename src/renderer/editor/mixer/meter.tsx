import { cn } from "@/renderer/lib/utils";
import { percent, signedText } from "@/shared/format";
import { clamp01, normalize } from "@/shared/math";
import { usePlayback } from "../../store/playback";
import { useFilm } from "../film";
import { formatLevelDb, formatSignedDb } from "./audio-format";
import { levelAt, reductionAt, type Level, type Levels, type Reduction } from "./meter-levels";

const METER_MIN = -60;
const METER_MAX = 6;
const MAX_REDUCTION_DB = 24;
const CLIP_DB = -0.5;
const HOLD_SECONDS = 1.5;

const levelHeight = (level: number) => percent(normalize(level, METER_MIN, METER_MAX));

function Meter({ level, reduction }: { level: Level; reduction: number | null }) {
  const peak = level?.peak ?? METER_MIN;
  const rms = level?.rms ?? METER_MIN;
  return (
    <div className="flex h-full gap-0.5">
      <div
        className="bg-muted relative h-full w-2.5 overflow-hidden rounded-sm"
        title={
          level
            ? `Peak ${formatLevelDb(level.peak)} dB, average ${formatLevelDb(level.rms)} dB`
            : "No level yet"
        }
      >
        <div
          className="from-meter-low via-meter-mid to-meter-high absolute inset-x-0 bottom-0 bg-gradient-to-t opacity-40"
          style={{ height: levelHeight(peak), backgroundSize: "100% 100%" }}
        />
        <div
          className="bg-meter-low absolute inset-x-0 bottom-0"
          style={{ height: levelHeight(rms) }}
        />
        {level && level.hold > METER_MIN ? (
          <div
            className={cn(
              "absolute inset-x-0 h-px",
              level.hold > CLIP_DB ? "bg-meter-high" : "bg-foreground",
            )}
            style={{ bottom: levelHeight(level.hold) }}
          />
        ) : null}
        <div
          className="bg-foreground/40 absolute inset-x-0 h-px"
          style={{ bottom: levelHeight(0) }}
        />
      </div>
      {reduction !== null ? (
        <div
          className="bg-muted relative h-full w-1 overflow-hidden rounded-sm"
          title={`Compressor lowers by ${signedText(reduction, { digits: 1 })} dB`}
        >
          <div
            className="bg-keyframe absolute inset-x-0 top-0"
            style={{ height: percent(clamp01(reduction / MAX_REDUCTION_DB)) }}
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
}: {
  levels: Levels;
  reduction: Reduction;
  compOn: boolean;
}) {
  const frame = usePlayback((s) => s.frame);
  const fps = useFilm().flat?.fps ?? 0;
  const lowered = compOn ? reductionAt(reduction, frame) : null;
  return (
    <Meter level={levelAt(levels, frame, Math.round(fps * HOLD_SECONDS))} reduction={lowered} />
  );
}

export function ReductionReadout({ reduction }: { reduction: Reduction }) {
  const frame = usePlayback((s) => s.frame);
  const lowered = reductionAt(reduction, frame);
  return lowered === null ? null : <span className="tabular-nums">{formatSignedDb(-lowered)}</span>;
}
