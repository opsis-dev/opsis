import { percent } from "@/shared/format";
import { clamp } from "@/shared/math";
import { MAX_DB, MIN_DB } from "@/shared/mix/types";
import { db } from "./audio-format";
import { commitOnce, onArrowKey, type Change } from "./gesture";
import { roundToStep } from "./knob-math";
import { fractionIn } from "@/renderer/lib/dom";

const STEP_DB = 0.5;
const BIG_STEP_DB = 6;

const faderPosition = (value: number) =>
  (clamp(value, MIN_DB, MAX_DB) - MIN_DB) / (MAX_DB - MIN_DB);

function valueAt(el: HTMLElement, clientY: number): number {
  const k = 1 - fractionIn(el.getBoundingClientRect(), 0, clientY).y;
  return roundToStep(MIN_DB + k * (MAX_DB - MIN_DB), STEP_DB);
}

export function Fader({
  value,
  onChange,
  label,
}: {
  value: number;
  onChange: Change<number>;
  label: string;
}) {
  return (
    <div
      role="slider"
      tabIndex={0}
      aria-label={label}
      aria-valuemin={MIN_DB}
      aria-valuemax={MAX_DB}
      aria-valuenow={value}
      aria-valuetext={`${db(value)} dB`}
      title={`${label}: ${db(value)} dB. Double-click for 0 dB.`}
      className="focus-visible:ring-ring relative h-full w-4 cursor-ns-resize touch-none rounded outline-none focus-visible:ring-2"
      onPointerDown={(e) => {
        if (e.button !== 0) return;
        e.currentTarget.setPointerCapture(e.pointerId);
        onChange(value, "start");
        onChange(valueAt(e.currentTarget, e.clientY), "move");
      }}
      onPointerMove={(e) => {
        if (e.currentTarget.hasPointerCapture(e.pointerId))
          onChange(valueAt(e.currentTarget, e.clientY), "move");
      }}
      onLostPointerCapture={() => onChange(value, "end")}
      onDoubleClick={() => commitOnce(onChange, 0)}
      onKeyDown={(e) =>
        onArrowKey(e, (dir, big) => {
          const step = big ? BIG_STEP_DB : STEP_DB;
          commitOnce(onChange, clamp(value + dir * step, MIN_DB, MAX_DB));
        })
      }
    >
      <div className="bg-muted absolute inset-y-1 left-1/2 w-1 -translate-x-1/2 rounded-full" />
      <div
        className="bg-foreground/40 absolute left-0 h-px w-full"
        style={{ bottom: percent(faderPosition(0)) }}
      />
      <div
        className="bg-primary border-background absolute left-1/2 h-2.5 w-4 -translate-x-1/2 translate-y-1/2 rounded-sm border"
        style={{ bottom: percent(faderPosition(value)) }}
      />
    </div>
  );
}
