import { useRef, useState } from "react";
import { endIfStarted, onArrowKey, startOnceMoved, type ThresholdDrag } from "@/renderer/lib/dom";
import { usePointerDrag } from "@/renderer/lib/use-pointer-drag";
import { cn } from "@/renderer/lib/utils";
import { clamp01 } from "@/shared/math";
import { parseNumberText } from "@/shared/text-input";
import { commitOnce, type Gesture } from "./gesture";
import {
  START_DEG,
  SWEEP_DEG,
  angleOf,
  arcPath,
  fractionToValue,
  polar,
  snapValue,
  valueToFraction,
  type KnobRange,
} from "./knob-math";

type KnobLook = {
  label: string;
  reset: number;
  format: (v: number) => string;
  unit?: string;
};

export type KnobSpec<F extends string> = KnobRange & KnobLook & { field: F };

const DRAG_PX = 150;
const FINE_DRAG_PX = 600;
const NUDGE = 0.02;
const BIG_NUDGE = 0.1;
const RING_INSET = 3;
const POINTER_INSET = 4;

const withUnit = (text: string, unit: string | undefined) => (unit ? `${text} ${unit}` : text);

function KnobValue({
  label,
  value,
  disabled,
  format,
  onCommit,
}: {
  label: string;
  value: number;
  disabled?: boolean;
  format: (v: number) => string;
  onCommit: (v: number) => void;
}) {
  const [typing, setTyping] = useState(false);
  const cancelled = useRef(false);
  if (typing)
    return (
      <input
        autoFocus
        aria-label={`${label} value`}
        defaultValue={String(value)}
        className="bg-background w-12 rounded-sm border px-0.5 text-center text-[10px] leading-none tabular-nums"
        onFocus={(e) => e.currentTarget.select()}
        onKeyDown={(e) => {
          e.stopPropagation();
          if (e.key === "Escape") cancelled.current = true;
          if (e.key === "Enter" || e.key === "Escape") e.currentTarget.blur();
        }}
        onBlur={(e) => {
          const keep = cancelled.current;
          cancelled.current = false;
          setTyping(false);
          const v = keep ? null : parseNumberText(e.currentTarget.value, { lenient: true });
          if (v !== null) onCommit(v);
        }}
      />
    );
  return (
    <button
      type="button"
      disabled={disabled}
      title="Click to type a value"
      className="hover:bg-accent rounded-sm px-0.5 text-[10px] leading-none tabular-nums"
      onClick={() => setTyping(true)}
    >
      {format(value)}
    </button>
  );
}

function KnobDial({ size, t, centerT }: { size: number; t: number; centerT: number }) {
  const c = size / 2;
  const r = c - RING_INSET;
  const angle = angleOf(t);
  const center = angleOf(centerT);
  const tip = polar(c, r - POINTER_INSET, angle);
  return (
    <svg width={size} height={size}>
      <path
        d={arcPath(c, r, START_DEG, START_DEG + SWEEP_DEG)}
        className="stroke-muted"
        strokeWidth={3}
        fill="none"
        strokeLinecap="round"
      />
      {Math.abs(angle - center) > 0.5 ? (
        <path
          d={arcPath(c, r, Math.min(center, angle), Math.max(center, angle))}
          className="stroke-primary"
          strokeWidth={3}
          fill="none"
          strokeLinecap="round"
        />
      ) : null}
      <line
        x1={c}
        y1={c}
        x2={tip.x}
        y2={tip.y}
        className="stroke-foreground"
        strokeWidth={2}
        strokeLinecap="round"
      />
    </svg>
  );
}

function useKnobDrag(
  range: KnobRange,
  fraction: number,
  gesture: Gesture<number>,
  disabled: boolean | undefined,
) {
  return usePointerDrag<HTMLButtonElement, ThresholdDrag & { y: number; from: number }>({
    disabled,
    start: (e) => ({ y: e.clientY, from: fraction, started: false }),
    move: (e, d) => {
      const dy = d.y - e.clientY;
      if (!startOnceMoved(d, Math.abs(dy), gesture.begin)) return;
      gesture.set(fractionToValue(range, d.from + dy / (e.shiftKey ? FINE_DRAG_PX : DRAG_PX)));
    },
    end: (d) => endIfStarted(d, gesture.end),
  });
}

export function Knob({
  label,
  value,
  min,
  max,
  step,
  reset,
  format,
  unit,
  gesture,
  disabled,
  size = 30,
  log,
}: KnobRange &
  KnobLook & {
    value: number;
    gesture: Gesture<number>;
    disabled?: boolean;
    size?: number;
  }) {
  const range = { min, max, step, log };
  const fraction = clamp01(valueToFraction(range, value));
  const nudge = (by: number) => commitOnce(gesture, fractionToValue(range, fraction + by));
  const centerT = min < 0 && max > 0 && !log ? valueToFraction(range, 0) : 0;
  const dragProps = useKnobDrag(range, fraction, gesture, disabled);
  const spoken = `${label}: ${withUnit(format(value), unit)}`;
  return (
    <div className={cn("flex flex-col items-center gap-0.5", disabled && "opacity-40")}>
      <button
        type="button"
        aria-label={spoken}
        title={`${spoken}. Drag up or down, double-click to reset, click the number to type.`}
        disabled={disabled}
        className="focus-visible:ring-ring rounded-full outline-none focus-visible:ring-2"
        style={{ width: size, height: size, touchAction: "none" }}
        {...dragProps}
        onDoubleClick={() => commitOnce(gesture, reset)}
        onWheel={(e) => {
          if (!disabled) nudge(e.deltaY < 0 ? NUDGE : -NUDGE);
        }}
        onKeyDown={(e) => onArrowKey(e, (dir, big) => nudge(dir * (big ? BIG_NUDGE : NUDGE)))}
      >
        <KnobDial size={size} t={fraction} centerT={centerT} />
      </button>
      <span className="text-muted-foreground text-[10px] leading-none">{label}</span>
      <KnobValue
        label={label}
        value={value}
        disabled={disabled}
        format={format}
        onCommit={(v) => commitOnce(gesture, snapValue(range, v))}
      />
    </div>
  );
}

export function Knobs<F extends string>({
  specs,
  values,
  disabled,
  gestureFor,
}: {
  specs: Array<KnobSpec<F>>;
  values: Record<F, number>;
  disabled: boolean;
  gestureFor: (field: F) => Gesture<number>;
}) {
  return specs.map(({ field, ...spec }) => (
    <Knob
      key={field}
      {...spec}
      value={values[field]}
      disabled={disabled}
      gesture={gestureFor(field)}
    />
  ));
}
