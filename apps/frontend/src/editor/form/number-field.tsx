import { clamp, type MinMax, type StepRange } from "@opsis/video/common/math";
import { SingleSlider } from "@/app/single-slider";
import { Field, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { parseShown, shownText, unitScale, useDecimalInput } from "./decimal";
import { Help, LabelledChoices } from "./field-shell";
import type { FieldProps } from "./types";

const MAX_SLIDER_RANGE = 5000;
const FINE_RANGE = 1;
const FINE_STEP = 0.01;
const WHOLE_STEP = 1;

function sliderRange(min: number | undefined, max: number | undefined): StepRange | null {
  if (min == null || max == null || max - min > MAX_SLIDER_RANGE) return null;
  return { min, max, step: max - min <= FINE_RANGE ? FINE_STEP : WHOLE_STEP };
}

type NumberProps = Omit<FieldProps<number>, "value"> &
  Partial<MinMax> & { value: number | undefined; onClear?: () => void };

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

export function NumberField({
  id,
  label,
  value,
  min,
  max,
  meta,
  ctx,
  onChange,
  onClear,
}: NumberProps) {
  const scale = unitScale(meta.unit, ctx.fps);
  const zeroLabel =
    meta.unconnectedZeroLabel && ctx.connectedInputs.length === 0
      ? meta.unconnectedZeroLabel
      : meta.zeroLabel;
  const zeroIsEmpty = !onClear && zeroLabel && value === 0;
  const shown = value === undefined || zeroIsEmpty ? "" : shownText(value, scale);
  const input = useDecimalInput(shown, "edit", (text) => {
    if (text.trim() === "") {
      if (onClear) onClear();
      else if (zeroLabel) onChange(0);
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
            placeholder={onClear ? (zeroLabel ?? "Auto") : zeroLabel}
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
          value={value ?? min ?? 0}
          onChange={onChange}
        />
      ) : null}
      <Help meta={meta} />
    </Field>
  );
}
