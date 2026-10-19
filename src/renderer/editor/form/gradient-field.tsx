import { Trash2 } from "lucide-react";
import { useEffect, useRef, useState, type PointerEvent } from "react";
import { Button } from "@/renderer/components/ui/button";
import { Field, FieldLabel } from "@/renderer/components/ui/field";
import { Input } from "@/renderer/components/ui/input";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { fractionIn } from "@/renderer/lib/dom";
import { useCommitDraft } from "@/renderer/lib/use-commit-draft";
import { usePointerDrag } from "@/renderer/lib/use-pointer-drag";
import { percentText } from "@/shared/format";
import {
  formatGradient,
  insertStop,
  MIN_STOPS,
  parseGradient,
  sortStops,
  type Gradient,
} from "@/shared/gradient";
import { clamp } from "@/shared/math";
import { parseNumberText } from "@/shared/text-input";
import { useWorkflowStore } from "../../store/workflow";
import { ColorInput } from "./color-field";
import { CHECKER } from "./color-picker";
import { trimNumber, useDecimalInput } from "./decimal";
import { Help } from "./field-shell";
import type { FieldProps } from "./types";

function StopBar({
  gradient,
  selected,
  onSelect,
  onChange,
}: {
  gradient: Gradient;
  selected: number;
  onSelect: (index: number) => void;
  onChange: (gradient: Gradient) => void;
}) {
  const bar = useRef<HTMLDivElement>(null);
  const latest = useRef(gradient);
  useEffect(() => {
    latest.current = gradient;
  });
  const atOf = (e: PointerEvent) => {
    const rect = bar.current?.getBoundingClientRect();
    return rect ? fractionIn(rect, e.clientX, e.clientY).x * 100 : 0;
  };
  const drag = usePointerDrag<HTMLButtonElement, number>({
    start: (e) => {
      const index = Number(e.currentTarget.dataset.stop);
      onSelect(index);
      useWorkflowStore.getState().beginGesture();
      return index;
    },
    move: (e, index) =>
      onChange({
        ...gradient,
        stops: gradient.stops.map((s, i) => (i === index ? { ...s, at: Math.round(atOf(e)) } : s)),
      }),
    end: (index) => {
      const sorted = sortStops(latest.current);
      onChange(sorted);
      onSelect(sorted.stops.indexOf(latest.current.stops[index]));
      useWorkflowStore.getState().endGesture();
    },
  });
  const linear = formatGradient({ ...gradient, type: "linear", angle: 90 });
  return (
    <div className="relative px-2 pb-3">
      <div
        ref={bar}
        className="h-6 w-full cursor-copy rounded-md border"
        style={CHECKER}
        onPointerDown={(e) => {
          if (e.target !== e.currentTarget) return;
          const next = insertStop(gradient, atOf(e));
          if (next === gradient) return;
          onChange(next);
          onSelect(next.stops.length - 1);
        }}
      >
        <div
          className="pointer-events-none h-full w-full rounded-md"
          style={{ background: linear }}
        />
      </div>
      {gradient.stops.map((stop, i) => (
        <button
          key={i}
          type="button"
          data-stop={i}
          aria-label={`Colour stop ${i + 1} at ${percentText(stop.at / 100)}`}
          aria-pressed={i === selected}
          className="absolute top-4 size-4 -translate-x-1/2 cursor-ew-resize touch-none rounded-sm border-2 border-white shadow-[0_0_0_1px_rgba(0,0,0,0.5)] aria-pressed:ring-2 aria-pressed:ring-ring"
          style={{
            left: `calc(0.5rem + (100% - 1rem) * ${stop.at / 100})`,
            background: stop.color,
          }}
          onKeyDown={(e) => {
            const step = e.key === "ArrowLeft" ? -1 : e.key === "ArrowRight" ? 1 : 0;
            if (!step) return;
            e.preventDefault();
            onChange({
              ...gradient,
              stops: gradient.stops.map((s, j) =>
                j === i ? { ...s, at: clamp(s.at + step * (e.shiftKey ? 10 : 1), 0, 100) } : s,
              ),
            });
          }}
          {...drag}
        />
      ))}
    </div>
  );
}

function SmallNumber({
  label,
  value,
  unit,
  min,
  max,
  onChange,
}: {
  label: string;
  value: number;
  unit: string;
  min: number;
  max: number;
  onChange: (value: number) => void;
}) {
  const input = useDecimalInput(trimNumber(value), "edit", (text) => {
    const n = parseNumberText(text);
    if (n !== null) onChange(clamp(n, min, max));
  });
  return (
    <label className="text-muted-foreground flex items-center gap-1 text-xs">
      {label}
      <Input aria-label={label} className="h-7 w-16 text-right tabular-nums" {...input} />
      {unit}
    </label>
  );
}

function RawGradient({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  const draft = useCommitDraft(value, onChange);
  return (
    <Input
      aria-label="Gradient as text"
      value={draft.text}
      spellCheck={false}
      onChange={(e) => draft.change(e.target.value)}
      onBlur={draft.commit}
      onKeyDown={(e) => {
        if (e.key === "Enter") draft.commit();
      }}
      className="h-7 font-mono text-xs"
    />
  );
}

export function GradientField({ id, label, meta, value, onChange, ctx }: FieldProps<string>) {
  const gradient = parseGradient(value);
  const [selected, setSelected] = useState(0);
  const update = (next: Gradient) => onChange(formatGradient(next));
  const index = gradient ? clamp(selected, 0, gradient.stops.length - 1) : 0;
  const stop = gradient?.stops[index];
  return (
    <Field>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      {gradient && stop ? (
        <>
          <StopBar gradient={gradient} selected={index} onSelect={setSelected} onChange={update} />
          <div className="flex flex-wrap items-center gap-2">
            <ToggleGroup
              variant="outline"
              size="sm"
              value={[gradient.type]}
              onValueChange={(v) => {
                if (v[0] === "linear" || v[0] === "radial") update({ ...gradient, type: v[0] });
              }}
            >
              <ToggleGroupItem value="linear">Straight</ToggleGroupItem>
              <ToggleGroupItem value="radial">Round</ToggleGroupItem>
            </ToggleGroup>
            {gradient.type === "linear" ? (
              <SmallNumber
                label="Angle"
                unit="°"
                min={0}
                max={360}
                value={gradient.angle}
                onChange={(angle) => update({ ...gradient, angle })}
              />
            ) : null}
          </div>
          <div className="flex items-center gap-2">
            <div className="min-w-0 flex-1">
              <ColorInput
                label={`Stop ${index + 1} colour`}
                value={stop.color}
                swatches={ctx.swatches}
                onChange={(color) =>
                  update({
                    ...gradient,
                    stops: gradient.stops.map((s, i) => (i === index ? { ...s, color } : s)),
                  })
                }
              />
            </div>
            <SmallNumber
              label="At"
              unit="%"
              min={0}
              max={100}
              value={stop.at}
              onChange={(at) =>
                update({
                  ...gradient,
                  stops: gradient.stops.map((s, i) => (i === index ? { ...s, at } : s)),
                })
              }
            />
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label="Remove this colour stop"
              disabled={gradient.stops.length <= MIN_STOPS}
              onClick={() => {
                update({ ...gradient, stops: gradient.stops.toSpliced(index, 1) });
                setSelected(Math.max(0, index - 1));
              }}
            >
              <Trash2 />
            </Button>
          </div>
        </>
      ) : (
        <p className="text-muted-foreground text-xs">
          This gradient can only be edited as text. Use two or more colours in a straight or round
          gradient to edit it here.
        </p>
      )}
      <RawGradient value={value} onChange={onChange} />
      <Help meta={meta} />
    </Field>
  );
}
