import { clamp, clamp01 } from "@/shared/math";
import { MAX_DB, MIN_DB } from "@/shared/mix/types";
import { db } from "./audio-format";
import { commitOnce, type DragPhase } from "./knob";
import { arrowDirection, roundToStep } from "./knob-math";

const STEP_DB = 0.5;
const BIG_STEP_DB = 6;

const faderPosition = (value: number) =>
  (clamp(value, MIN_DB, MAX_DB) - MIN_DB) / (MAX_DB - MIN_DB);

function valueAt(el: HTMLElement, clientY: number): number {
  const rect = el.getBoundingClientRect();
  const k = 1 - (clientY - rect.top) / Math.max(1, rect.height);
  return roundToStep(MIN_DB + clamp01(k) * (MAX_DB - MIN_DB), STEP_DB);
}

export function Fader({
  value,
  onChange,
  label,
}: {
  value: number;
  onChange: (v: number, phase: DragPhase) => void;
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
      onPointerUp={(e) => {
        if (!e.currentTarget.hasPointerCapture(e.pointerId)) return;
        e.currentTarget.releasePointerCapture(e.pointerId);
        onChange(value, "end");
      }}
      onDoubleClick={() => commitOnce(onChange, 0)}
      onKeyDown={(e) => {
        const dir = arrowDirection(e.key);
        if (!dir) return;
        e.preventDefault();
        e.stopPropagation();
        const step = e.shiftKey ? BIG_STEP_DB : STEP_DB;
        commitOnce(onChange, clamp(value + dir * step, MIN_DB, MAX_DB));
      }}
    >
      <div className="bg-muted absolute inset-y-1 left-1/2 w-1 -translate-x-1/2 rounded-full" />
      <div
        className="bg-foreground/40 absolute left-0 h-px w-full"
        style={{ bottom: `${faderPosition(0) * 100}%` }}
      />
      <div
        className="bg-primary border-background absolute left-1/2 h-2.5 w-4 -translate-x-1/2 translate-y-1/2 rounded-sm border"
        style={{ bottom: `${faderPosition(value) * 100}%` }}
      />
    </div>
  );
}
