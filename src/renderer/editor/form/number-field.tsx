import type { ComponentType } from "react";
import type { FieldWidget } from "@/shared/elements/introspect";
import { clamp } from "@/shared/math";
import { LOUDNESS_PRESETS, QUALITY_PRESETS } from "@/shared/nodes/params";
import { Field, FieldLabel } from "@/renderer/components/ui/field";
import { Input } from "@/renderer/components/ui/input";
import { SingleSlider } from "../../app/single-slider";
import { parseShown, shownText, unitScale, useDecimalInput } from "./decimal";
import { Help, LabelledChoices } from "./field-shell";
import { SpotField } from "./spot-field";
import type { FieldProps } from "./types";

const MAX_SLIDER_RANGE = 5000;

const FINE_SLIDER_RANGE = 1;

const sliderStep = (min: number, max: number) => (max - min <= FINE_SLIDER_RANGE ? 0.01 : 1);

type NumberProps = FieldProps<number> & { min?: number; max?: number };

const presetNumber =
  (presets: Readonly<Record<string, number>>) =>
  ({ label, meta, value, onChange }: NumberProps) => {
    const options = Object.entries(presets).map(([name, n]) => ({ label: name, value: n }));
    const gaps = options.map((o) => Math.abs(o.value - value));
    const nearest = gaps.indexOf(Math.min(...gaps));
    return (
      <LabelledChoices
        label={label}
        meta={meta}
        options={options}
        current={nearest < 0 ? undefined : nearest}
        onPick={onChange}
      />
    );
  };

const NUMBER_WIDGETS: Partial<Record<FieldWidget, ComponentType<NumberProps>>> = {
  quality: presetNumber(QUALITY_PRESETS),
  loudness: presetNumber(LOUDNESS_PRESETS),
  spot: SpotField,
};

export function NumberControl(props: NumberProps) {
  const { meta } = props;
  const Widget = (meta.widget && NUMBER_WIDGETS[meta.widget]) || NumberField;
  return <Widget {...props} />;
}

function NumberField({ id, label, value, min, max, meta, ctx, onChange }: NumberProps) {
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
  const slider = min != null && max != null && max - min <= MAX_SLIDER_RANGE && !scale.time;
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
          min={min}
          max={max}
          step={meta.step ?? sliderStep(min, max)}
          value={value}
          onChange={onChange}
        />
      ) : null}
      <Help meta={meta} />
    </Field>
  );
}
