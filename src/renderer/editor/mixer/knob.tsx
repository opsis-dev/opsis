import { useRef } from "react";
import { cn } from "@/renderer/lib/utils";

export type DragPhase = "start" | "move" | "end";

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
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  reset: number;
  format: (v: number) => string;
  onChange: (value: number, phase: DragPhase) => void;
  disabled?: boolean;
  size?: number;
  log?: boolean;
}) {
  const drag = useRef<{ y: number; t0: number; started: boolean } | null>(null);
  const toT = (v: number) =>
    log ? Math.log(v / min) / Math.log(max / min) : (v - min) / (max - min);
  const fromT = (t: number) => {
    const c = Math.min(1, Math.max(0, t));
    const v = log ? min * (max / min) ** c : min + c * (max - min);
    const decimals = Math.max(0, -Math.floor(Math.log10(step)));
    return Number((Math.round(v / step) * step).toFixed(decimals));
  };
  const t = Math.min(1, Math.max(0, toT(value)));
  const angle = -135 + t * 270;
  const r = size / 2 - 3;
  const arc = (from: number, to: number) => {
    const a0 = ((from - 90) * Math.PI) / 180;
    const a1 = ((to - 90) * Math.PI) / 180;
    const x0 = size / 2 + r * Math.cos(a0);
    const y0 = size / 2 + r * Math.sin(a0);
    const x1 = size / 2 + r * Math.cos(a1);
    const y1 = size / 2 + r * Math.sin(a1);
    return `M ${x0} ${y0} A ${r} ${r} 0 ${to - from > 180 ? 1 : 0} 1 ${x1} ${y1}`;
  };
  const centre = min < 0 && max > 0 && !log ? -135 + toT(0) * 270 : -135;
  return (
    <div className={cn("flex flex-col items-center gap-0.5", disabled && "opacity-40")}>
      <button
        type="button"
        aria-label={`${label}: ${format(value)}`}
        title={`${label}: ${format(value)}. Drag up or down, double-click to reset.`}
        disabled={disabled}
        className="focus-visible:ring-ring rounded-full outline-none focus-visible:ring-2"
        style={{ width: size, height: size, touchAction: "none" }}
        onPointerDown={(e) => {
          if (disabled || e.button !== 0) return;
          e.currentTarget.setPointerCapture(e.pointerId);
          drag.current = { y: e.clientY, t0: t, started: false };
        }}
        onPointerMove={(e) => {
          const d = drag.current;
          if (!d) return;
          const dy = d.y - e.clientY;
          if (!d.started) {
            if (Math.abs(dy) < 2) return;
            d.started = true;
            onChange(value, "start");
          }
          onChange(fromT(d.t0 + dy / (e.shiftKey ? 600 : 150)), "move");
        }}
        onPointerUp={() => {
          const d = drag.current;
          drag.current = null;
          if (d?.started) onChange(value, "end");
        }}
        onDoubleClick={() => {
          onChange(value, "start");
          onChange(reset, "move");
          onChange(reset, "end");
        }}
        onWheel={(e) => {
          if (disabled) return;
          onChange(value, "start");
          onChange(fromT(t + (e.deltaY < 0 ? 0.02 : -0.02)), "move");
          onChange(value, "end");
        }}
        onKeyDown={(e) => {
          const dir =
            e.key === "ArrowUp" || e.key === "ArrowRight"
              ? 1
              : e.key === "ArrowDown" || e.key === "ArrowLeft"
                ? -1
                : 0;
          if (!dir) return;
          e.preventDefault();
          e.stopPropagation();
          onChange(value, "start");
          onChange(fromT(t + dir * (e.shiftKey ? 0.1 : 0.02)), "move");
          onChange(value, "end");
        }}
      >
        <svg width={size} height={size}>
          <path
            d={arc(-135, 135)}
            className="stroke-muted"
            strokeWidth={3}
            fill="none"
            strokeLinecap="round"
          />
          {Math.abs(angle - centre) > 0.5 ? (
            <path
              d={arc(Math.min(centre, angle), Math.max(centre, angle))}
              className="stroke-primary"
              strokeWidth={3}
              fill="none"
              strokeLinecap="round"
            />
          ) : null}
          <line
            x1={size / 2}
            y1={size / 2}
            x2={size / 2 + (r - 4) * Math.cos(((angle - 90) * Math.PI) / 180)}
            y2={size / 2 + (r - 4) * Math.sin(((angle - 90) * Math.PI) / 180)}
            className="stroke-foreground"
            strokeWidth={2}
            strokeLinecap="round"
          />
        </svg>
      </button>
      <span className="text-muted-foreground text-[10px] leading-none">{label}</span>
      <span className="text-[10px] leading-none tabular-nums">{format(value)}</span>
    </div>
  );
}
