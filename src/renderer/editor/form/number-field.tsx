import type { FieldMeta } from "@/shared/elements/introspect";
import { clamp } from "@/shared/math";
import { LOUDNESS_PRESETS, QUALITY_PRESETS } from "@/shared/nodes/params";
import { Field, FieldLabel } from "@/renderer/components/ui/field";
import { Input } from "@/renderer/components/ui/input";
import { Slider } from "@/renderer/components/ui/slider";
import { parseNumberText } from "@/shared/format";
import { trimNumber, unitScale, useDecimalInput } from "./decimal";
import { ChoiceGroup, Help } from "./field-shell";
import { SpotField } from "./spot-field";
import type { FieldProps, Patch } from "./types";

const NUMBER_PRESETS: Record<string, Readonly<Record<string, number>>> = {
  quality: QUALITY_PRESETS,
  loudness: LOUDNESS_PRESETS,
};
const MAX_SLIDER_RANGE = 5000;
const CUSTOM = "Custom";

type NumberProps = FieldProps<number> & { min?: number; max?: number };

export function NumberControl(props: NumberProps) {
  const { meta } = props;
  const presets = meta.widget ? NUMBER_PRESETS[meta.widget] : undefined;
  if (presets) return <PresetNumber {...props} presets={presets} />;
  if (meta.widget === "spot") return <SpotField {...props} />;
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
  const input = useDecimalInput(shown, {
    onEdit: (text) => {
      if (text.trim() === "") {
        if (meta.zeroLabel) onChange(0);
        return;
      }
      const n = parseNumberText(text);
      if (n !== null) onChange(toStored(n));
    },
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
        <Slider
          min={min}
          max={max}
          step={meta.step ?? (max - min <= 1 ? 0.01 : 1)}
          value={[value]}
          onValueChange={(v) => onChange(Array.isArray(v) ? v[0] : v)}
        />
      ) : null}
      <Help meta={meta} />
    </Field>
  );
}

function PresetNumber({
  label,
  meta,
  value,
  presets,
  onChange,
}: NumberProps & { presets: Readonly<Record<string, number>> }) {
  const entries = Object.entries(presets);
  const nearest = entries.reduce((best, cur) =>
    Math.abs(cur[1] - value) < Math.abs(best[1] - value) ? cur : best,
  )[0];
  return (
    <ChoiceGroup
      label={label}
      meta={meta}
      choices={entries.map(([l]) => ({ value: l, label: l }))}
      value={[nearest]}
      onChange={(v) => {
        const hit = entries.find(([l]) => l === v[0]);
        if (hit) onChange(hit[1]);
      }}
    />
  );
}

function Presets({
  meta,
  all,
  onPatch,
}: {
  meta: FieldMeta;
  all: Patch;
  onPatch: (patch: Patch) => void;
}) {
  const presets = meta.presets ?? [];
  const current =
    presets.find((p) => Object.entries(p.values).every(([k, v]) => all[k] === v))?.label ?? CUSTOM;
  return (
    <ChoiceGroup
      label="Size"
      choices={[
        ...presets.map((p) => ({ value: p.label, label: p.label })),
        { value: CUSTOM, label: CUSTOM, disabled: true },
      ]}
      value={[current]}
      onChange={(v) => {
        const hit = presets.find((p) => p.label === v[0]);
        if (hit) onPatch(hit.values);
      }}
    />
  );
}
