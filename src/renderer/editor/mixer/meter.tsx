import { clamp01 } from "@/shared/math";
import type { MixResult, StemLevels } from "@/shared/mix/result";
import { cn } from "@/renderer/lib/utils";
import { frameIn } from "@/shared/timeline/time";
import { usePlayback } from "../../store/playback";

const SILENCE_DB = -120;
const METER_MIN = -60;
const METER_MAX = 6;
const MAX_REDUCTION_DB = 24;
const CLIP_DB = -0.5;

const toPct = (db: number) => clamp01((db - METER_MIN) / (METER_MAX - METER_MIN)) * 100;

export const sampleAt = (values: number[] | null | undefined, frame: number): number | null =>
  values?.length ? values[frameIn(frame, values.length)] : null;

const reductionAt = (reduction: number[] | null | undefined, frame: number): number | null =>
  reduction ? (sampleAt(reduction, frame) ?? 0) : null;

export function share(
  result: MixResult | null,
  levels: Pick<StemLevels, "rms"> | undefined,
  frame: number,
): number {
  const master = result?.levels.master;
  if (!master?.rms.length || !levels) return 1;
  const f = frameIn(frame, master.rms.length);
  const pc = 10 ** ((levels.rms[f] ?? SILENCE_DB) / 10);
  const pm = 10 ** (master.rms[f] / 10);
  return pm <= 1e-12 ? 1 : Math.min(1, pc / pm);
}

type Level = { peak: number | null; rms: number | null; hold: number | null };

function levelAt(
  levels: Pick<StemLevels, "peak" | "rms"> | null | undefined,
  frame: number,
  holdFrames: number,
): Level {
  if (!levels?.peak.length) return { peak: null, rms: null, hold: null };
  const f = frameIn(frame, levels.peak.length);
  const hold = Math.max(...levels.peak.slice(Math.max(0, f - holdFrames), f + 1));
  return { peak: levels.peak[f], rms: levels.rms[f], hold };
}

function Meter({ level, reduction }: { level: Level; reduction?: number }) {
  const { peak, rms, hold } = level;
  return (
    <div className="flex h-full gap-0.5">
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
              hold > CLIP_DB ? "bg-meter-high" : "bg-foreground",
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

type Levels = Pick<StemLevels, "peak" | "rms"> | null | undefined;

export type Reduction = number[] | null | undefined;

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
  return lowered === null ? null : <span className="tabular-nums">−{lowered.toFixed(1)}</span>;
}
