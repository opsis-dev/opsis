import { useRef, type PointerEvent } from "react";
import { spotYKey } from "@/shared/elements/introspect";
import { percent, percentText } from "@/shared/format";
import { clamp01, finiteNumber, numberOr, round3 } from "@/shared/math";
import { fractionIn, usePointerDrag } from "@/renderer/lib/dom";
import { FieldShell } from "./field-shell";
import { SpotFrame } from "./spot-frame";
import type { ParamFieldProps } from "./types";

const NUDGES: Partial<Record<string, [number, number]>> = {
  ArrowLeft: [-1, 0],
  ArrowRight: [1, 0],
  ArrowUp: [0, -1],
  ArrowDown: [0, 1],
};
const NUDGE_STEP = 0.01;
const SHIFT_NUDGE_STEP = 0.1;
const SPOT_CENTRE = 0.5;

export function SpotField({
  label,
  name,
  value: x,
  all,
  onPatch,
  ctx,
  meta,
}: ParamFieldProps<number>) {
  const yKey = spotYKey(name);
  const y = numberOr(all[yKey], SPOT_CENTRE);
  const box = useRef<HTMLDivElement>(null);
  const place = (nextX: number, nextY: number) =>
    onPatch({ [name]: round3(clamp01(nextX)), [yKey]: round3(clamp01(nextY)) });
  const placeAt = (e: PointerEvent) => {
    const r = box.current?.getBoundingClientRect();
    if (!r) return;
    const at = fractionIn(r, e.clientX, e.clientY);
    place(at.x, at.y);
  };
  const drag = usePointerDrag({ onStart: placeAt, onMove: placeAt });
  return (
    <FieldShell label={label} meta={meta}>
      <SpotFrame
        ref={box}
        ctx={ctx}
        timeMs={meta.timeKey ? finiteNumber(all[meta.timeKey]) : undefined}
        role="button"
        tabIndex={0}
        aria-label={`${label}: ${percentText(x)}, ${percentText(y)}`}
        className="cursor-crosshair"
        {...drag}
        onKeyDown={(e) => {
          const nudge = NUDGES[e.key];
          if (!nudge) return;
          e.preventDefault();
          const step = e.shiftKey ? SHIFT_NUDGE_STEP : NUDGE_STEP;
          place(x + nudge[0] * step, y + nudge[1] * step);
        }}
      >
        <span
          className="border-background bg-primary pointer-events-none absolute size-3 -translate-1/2 rounded-full border-2 shadow"
          style={{ left: percent(x), top: percent(y) }}
        />
      </SpotFrame>
    </FieldShell>
  );
}
