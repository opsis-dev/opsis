import { Field, FieldLabel } from "@/renderer/components/ui/field";
import { Input } from "@/renderer/components/ui/input";
import { clamp, type MinMax, type StepRange } from "@/shared/math";
import { SingleSlider } from "../../app/single-slider";
import { parseShown, shownText, unitScale, useDecimalInput } from "./decimal";
import { Help, LabelledChoices } from "./field-shell";
import type { FieldProps } from "./types";

const MAX_SLIDER_RANGE = 5000;

function sliderRange(min: number | undefined, max: number | undefined): StepRange | null {
  if (min == null || max == null || max - min > MAX_SLIDER_RANGE) return null;
  return { min, max, step: max - min <= 1 ? 0.01 : 1 };
}

type NumberProps = FieldProps<number> & Partial<MinMax>;

export function PresetNumber({
  label,
  meta,
  value,
  onChange,
  presets,
}: FieldProps<number> & { presets: Readonly<Record<string, number>> }) {
  const options = Object.entries(presets).map(([name, n]) => ({ label: name, value: n }));
  const nearest = options.reduce((best, o) =>
    Math.abs(o.value - value) < Math.abs(best.value - value) ? o : best,
  );
  return (
    <LabelledChoices
      label={label}
      meta={meta}
      options={options}
      current={nearest.label}
      onPick={onChange}
    />
  );
}

export function NumberField({ id, label, value, min, max, meta, ctx, onChange }: NumberProps) {
  const scale = unitScale(meta.unit, ctx.fps);
  const shown = meta.zeroLabel && value === 0 ? "" : shownText(value, scale);
  const input = useDecimalInput(shown, "edit", (text) => {
    if (text.trim() === "") {
      if (meta.zeroLabel) onChange(0);
      return;
    }
    const n = parseShown(text, scale);
    if (n !== null) onChange(clamp(n, min ?? -Infinity, max ?? Infinity));
  });
  const slider = scale.time ? null : sliderRange(min, max);
  return (
    <Field>
      <div className="flex items-center justify-between gap-2">
        <FieldLabel htmlFor={id}>{label}</FieldLabel>
        <div className="flex items-center gap-1">
          <Input
            id={id}
            className="h-7 w-24 text-right tabular-nums"
            placeholder={meta.zeroLabel}
            {...input}
          />
          {scale.unit ? (
            <span className="text-muted-foreground w-12 text-xs">{scale.unit}</span>
          ) : null}
        </div>
      </div>
      {slider ? (
        <SingleSlider
          min={slider.min}
          max={slider.max}
          step={meta.step ?? slider.step}
          value={value}
          onChange={onChange}
        />
      ) : null}
      <Help meta={meta} />
    </Field>
  );
}
