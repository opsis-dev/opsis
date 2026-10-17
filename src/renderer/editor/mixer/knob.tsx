import { useRef, useState } from "react";
import { usePointerDrag } from "@/renderer/lib/dom";
import { parseNumberText } from "@/shared/format";
import { clamp01 } from "@/shared/math";
import { cn } from "@/renderer/lib/utils";
import { commitOnce, onArrowKey, type Change, type DragPhase } from "./gesture";
import {
  START_DEG,
  SWEEP_DEG,
  angleOf,
  arcPath,
  fromT,
  polar,
  snapValue,
  toT,
  type KnobRange,
} from "./knob-math";

export type KnobSpec<F extends string> = KnobRange & {
  field: F;
  label: string;
  reset: number;
  format: (v: number) => string;
};

const DRAG_START_PX = 2;
const DRAG_PX = 150;
const FINE_DRAG_PX = 600;
const NUDGE = 0.02;
const BIG_NUDGE = 0.1;
const RING_INSET = 3;
const POINTER_INSET = 4;

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
  const commit = (text: string) => {
    setTyping(false);
    const v = parseNumberText(text, { lenient: true });
    if (v !== null) onCommit(v);
  };
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
          if (e.key === "Enter") commit(e.currentTarget.value);
          if (e.key === "Escape") setTyping(false);
        }}
        onBlur={(e) => commit(e.currentTarget.value)}
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

function KnobDial({ size, t, centreT }: { size: number; t: number; centreT: number }) {
  const c = size / 2;
  const r = c - RING_INSET;
  const angle = angleOf(t);
  const centre = angleOf(centreT);
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
      {Math.abs(angle - centre) > 0.5 ? (
        <path
          d={arcPath(c, r, Math.min(centre, angle), Math.max(centre, angle))}
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

export function Knob({
  label,
  value,
  min,
  max,
  step,
  reset,
  format,
  onChange,
  disabled,
  size = 30,
  log,
}: KnobRange & {
  label: string;
  value: number;
  reset: number;
  format: (v: number) => string;
  onChange: Change<number>;
  disabled?: boolean;
  size?: number;
}) {
  const range = { min, max, step, log };
  const drag = useRef<{ y: number; t0: number; started: boolean } | null>(null);
  const t = clamp01(toT(range, value));
  const nudge = (by: number) => commitOnce(onChange, fromT(range, t + by));
  const centreT = min < 0 && max > 0 && !log ? toT(range, 0) : 0;
  const dragProps = usePointerDrag<HTMLButtonElement>({
    onStart: (e) => {
      if (disabled) return false;
      drag.current = { y: e.clientY, t0: t, started: false };
    },
    onMove: (e) => {
      const d = drag.current;
      if (!d) return;
      const dy = d.y - e.clientY;
      if (!d.started) {
        if (Math.abs(dy) < DRAG_START_PX) return;
        d.started = true;
        onChange(value, "start");
      }
      onChange(fromT(range, d.t0 + dy / (e.shiftKey ? FINE_DRAG_PX : DRAG_PX)), "move");
    },
    onEnd: () => {
      const d = drag.current;
      drag.current = null;
      if (d?.started) onChange(value, "end");
    },
  });
  return (
    <div className={cn("flex flex-col items-center gap-0.5", disabled && "opacity-40")}>
      <button
        type="button"
        aria-label={`${label}: ${format(value)}`}
        title={`${label}: ${format(value)}. Drag up or down, double-click to reset, click the number to type.`}
        disabled={disabled}
        className="focus-visible:ring-ring rounded-full outline-none focus-visible:ring-2"
        style={{ width: size, height: size, touchAction: "none" }}
        {...dragProps}
        onDoubleClick={() => commitOnce(onChange, reset)}
        onWheel={(e) => {
          if (!disabled) nudge(e.deltaY < 0 ? NUDGE : -NUDGE);
        }}
        onKeyDown={(e) => onArrowKey(e, (dir, big) => nudge(dir * (big ? BIG_NUDGE : NUDGE)))}
      >
        <KnobDial size={size} t={t} centreT={centreT} />
      </button>
      <span className="text-muted-foreground text-[10px] leading-none">{label}</span>
      <KnobValue
        label={label}
        value={value}
        disabled={disabled}
        format={format}
        onCommit={(v) => commitOnce(onChange, snapValue(range, v))}
      />
    </div>
  );
}

export function Knobs<F extends string>({
  specs,
  values,
  disabled,
  onChange,
}: {
  specs: Array<KnobSpec<F>>;
  values: Record<F, number>;
  disabled: boolean;
  onChange: (field: F, value: number, phase: DragPhase) => void;
}) {
  return specs.map(({ field, ...spec }) => (
    <Knob
      key={field}
      {...spec}
      value={values[field]}
      disabled={disabled}
      onChange={(v, phase) => onChange(field, v, phase)}
    />
  ));
}
