import { useMemo, useRef } from "react";
import { round3 } from "@/shared/math";
import { FieldShell } from "./field-shell";
import type { FieldProps } from "./types";
import { fractionIn } from "@/renderer/lib/dom";

const yKeyOf = (xKey: string) => (xKey === "x" ? "y" : xKey.replace(/X$/, "Y"));

export function SpotField({ label, name, all, onPatch, ctx, meta }: FieldProps<number>) {
  const yKey = yKeyOf(name);
  const x = Number(all[name] ?? 0.5);
  const y = Number(all[yKey] ?? 0.5);
  const box = useRef<HTMLDivElement>(null);
  const time = Number(all.atMs ?? 0);
  const bg = ctx.spotBackground;
  const set = (e: React.PointerEvent) => {
    const r = box.current?.getBoundingClientRect();
    if (!r) return;
    const at = fractionIn(r, e.clientX, e.clientY);
    onPatch({ [name]: round3(at.x), [yKey]: round3(at.y) });
  };
  const src = useMemo(() => (bg ? `${bg.src}#t=${(time || bg.timeMs) / 1000}` : null), [bg, time]);
  return (
    <FieldShell label={label} meta={meta}>
      <div
        ref={box}
        role="button"
        tabIndex={0}
        aria-label={`${label}: ${Math.round(x * 100)} %, ${Math.round(y * 100)} %`}
        className="bg-muted relative w-full cursor-crosshair overflow-hidden rounded-md border"
        style={{ aspectRatio: String(ctx.aspect) }}
        onPointerDown={(e) => {
          e.currentTarget.setPointerCapture(e.pointerId);
          set(e);
        }}
        onPointerMove={(e) => e.buttons === 1 && set(e)}
      >
        {src ? (
          <video
            src={src}
            muted
            preload="auto"
            className="pointer-events-none absolute inset-0 size-full object-cover"
          />
        ) : null}
        <span
          className="border-background bg-primary pointer-events-none absolute size-3 -translate-1/2 rounded-full border-2 shadow"
          style={{ left: `${x * 100}%`, top: `${y * 100}%` }}
        />
      </div>
    </FieldShell>
  );
}
