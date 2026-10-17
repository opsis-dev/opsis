import type { ComponentType } from "react";
import type { FieldWidget } from "@/shared/elements/introspect";
import { parseNumberText } from "@/shared/format";
import { clamp } from "@/shared/math";
import { LOUDNESS_PRESETS, QUALITY_PRESETS } from "@/shared/nodes/params";
import { Field, FieldLabel } from "@/renderer/components/ui/field";
import { Input } from "@/renderer/components/ui/input";
import { SingleSlider } from "../../app/single-slider";
import { trimNumber, unitScale, useDecimalInput } from "./decimal";
import { Help, LabelledChoices } from "./field-shell";
import { Presets } from "./presets";
import { SpotField } from "./spot-field";
import type { FieldProps } from "./types";

const MAX_SLIDER_RANGE = 5000;

type NumberProps = FieldProps<number> & { min?: number; max?: number };

const presetNumber =
  (presets: Readonly<Record<string, number>>) =>
  ({ label, meta, value, onChange }: NumberProps) => {
    const options = Object.entries(presets).map(([name, n]) => ({ label: name, value: n }));
    const nearest = options.reduce((best, cur) =>
      Math.abs(cur.value - value) < Math.abs(best.value - value) ? cur : best,
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
  };

const NUMBER_WIDGETS: Partial<Record<FieldWidget, ComponentType<NumberProps>>> = {
  quality: presetNumber(QUALITY_PRESETS),
  loudness: presetNumber(LOUDNESS_PRESETS),
  spot: SpotField,
};

export function NumberControl(props: NumberProps) {
  const { meta } = props;
  const Widget = meta.widget ? NUMBER_WIDGETS[meta.widget] : undefined;
  if (Widget) return <Widget {...props} />;
  return (
    <>
      {meta.presets ? <Presets meta={meta} all={props.all} onPatch={props.onPatch} /> : null}
      <NumberField {...props} />
    </>
  );
}

function NumberField({ id, label, value, min, max, meta, ctx, onChange }: NumberProps) {
  const scale = unitScale(meta.unit, ctx.fps);
  const shown = meta.zeroLabel && value === 0 ? "" : trimNumber(value / scale.factor);
  const toStored = (n: number) =>
    clamp(
      scale.time ? Math.round(n * scale.factor) : n * scale.factor,
      min ?? -Infinity,
      max ?? Infinity,
    );
  const input = useDecimalInput(shown, "edit", (text) => {
    if (text.trim() === "") {
      if (meta.zeroLabel) onChange(0);
      return;
    }
    const n = parseNumberText(text);
    if (n !== null) onChange(toStored(n));
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
          step={meta.step ?? (max - min <= 1 ? 0.01 : 1)}
          value={value}
          onChange={onChange}
        />
      ) : null}
      <Help meta={meta} />
    </Field>
  );
}
