import type { FieldMeta } from "@/shared/elements/introspect";
import { LabelledChoices } from "./field-shell";
import type { ParamValues } from "./types";

type Preset = NonNullable<FieldMeta["presets"]>[number];

export function Presets({
  presets,
  label = "Presets",
  all,
  onPatch,
}: {
  presets: Preset[];
  label?: string;
  all: ParamValues;
  onPatch: (patch: ParamValues) => void;
}) {
  const current = presets.find((p) => Object.entries(p.values).every(([k, v]) => all[k] === v));
  return (
    <LabelledChoices
      label={label}
      options={presets.map((p) => ({ label: p.label, value: p.values }))}
      current={current?.label}
      unmatchedLabel="Custom"
      onPick={onPatch}
    />
  );
}
