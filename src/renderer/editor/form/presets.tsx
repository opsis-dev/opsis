import type { FieldMeta } from "@/shared/elements/introspect";
import { LabelledChoices } from "./field-shell";
import type { Patch } from "./types";

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
  const current = presets.findIndex((p) =>
    Object.entries(p.values).every(([k, v]) => all[k] === v),
  );
  return (
    <LabelledChoices
      label={meta.presetsLabel ?? "Presets"}
      options={presets.map((p) => ({ label: p.label, value: p.values }))}
      current={current < 0 ? undefined : current}
      unmatchedLabel="Custom"
      onPick={onPatch}
    />
  );
}
