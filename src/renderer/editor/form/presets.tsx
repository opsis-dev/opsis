import type { FieldMeta } from "@/shared/elements/introspect";
import { LabelledChoices } from "./field-shell";
import type { Patch } from "./types";

const CUSTOM = "Custom";

export function Presets({
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
    <LabelledChoices
      label={meta.presetsLabel ?? "Presets"}
      options={[
        ...presets.map((p) => ({ label: p.label, value: p.values })),
        { label: CUSTOM, value: {}, disabled: true },
      ]}
      current={current}
      onPick={onPatch}
    />
  );
}
