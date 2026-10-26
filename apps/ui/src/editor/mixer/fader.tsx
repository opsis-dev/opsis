import { percent } from "@opsis/video/common/format";
import { MAX_DB, MIN_DB } from "@/core/mix";
import { fractionIn, onArrowKey } from "@/lib/dom";
import { usePointerDrag } from "@/lib/use-pointer-drag";
import { formatDb } from "./audio-format";
import { commitOnce, type Gesture } from "./gesture";
import { fractionToValue, snapValue, valueToFraction, type KnobRange } from "./knob-math";

const STEP_DB = 0.5;
const BIG_STEP_DB = 6;

const FADER_RANGE: KnobRange = { min: MIN_DB, max: MAX_DB, step: STEP_DB };

const faderPosition = (value: number) => valueToFraction(FADER_RANGE, value);

const valueAt = (el: HTMLElement, clientY: number): number =>
  fractionToValue(FADER_RANGE, 1 - fractionIn(el.getBoundingClientRect(), 0, clientY).y);

export function Fader({
  value,
  gesture,
  label,
}: {
  value: number;
  gesture: Gesture<number>;
  label: string;
}) {
  const drag = usePointerDrag<HTMLDivElement, void>({
    start: (e) => {
      gesture.begin();
      gesture.set(valueAt(e.currentTarget, e.clientY));
    },
    move: (e) => gesture.set(valueAt(e.currentTarget, e.clientY)),
    end: gesture.end,
  });
  return (
    <div
      role="slider"
      tabIndex={0}
      aria-label={label}
      aria-valuemin={MIN_DB}
      aria-valuemax={MAX_DB}
      aria-valuenow={value}
      aria-valuetext={`${formatDb(value)} dB`}
      title={`${label}: ${formatDb(value)} dB. Double-click for 0 dB.`}
      className="focus-visible:ring-ring relative h-full w-4 cursor-ns-resize touch-none rounded outline-none focus-visible:ring-2"
      {...drag}
      onDoubleClick={() => commitOnce(gesture, 0)}
      onKeyDown={(e) =>
        onArrowKey(e, (dir, big) => {
          const step = big ? BIG_STEP_DB : STEP_DB;
          commitOnce(gesture, snapValue(FADER_RANGE, value + dir * step));
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
