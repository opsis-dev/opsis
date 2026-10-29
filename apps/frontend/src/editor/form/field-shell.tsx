import type { ReactNode } from "react";
import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { FieldMeta } from "@/editor/form/schema";
import type { Choice } from "./choice-select";

export function Help({ meta }: { meta?: FieldMeta }) {
  return meta?.help ? <FieldDescription>{meta.help}</FieldDescription> : null;
}

export function FieldShell({
  id,
  label,
  meta,
  children,
}: {
  id?: string;
  label: string;
  meta?: FieldMeta;
  children: ReactNode;
}) {
  return (
    <Field>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      {children}
      <Help meta={meta} />
    </Field>
  );
}

export function ChoiceGroup({
  label,
  meta,
  choices,
  value,
  onChange,
  multiple,
}: {
  label: string;
  meta?: FieldMeta;
  choices: Choice[];
  value: string[];
  onChange: (value: string[]) => void;
  multiple?: boolean;
}) {
  return (
    <FieldShell label={label} meta={meta}>
      <ToggleGroup
        multiple={multiple}
        variant="outline"
        size="sm"
        value={value}
        onValueChange={onChange}
        className="flex-wrap"
      >
        {choices.map((c) => (
          <ToggleGroupItem key={c.value} value={c.value} disabled={c.disabled}>
            {c.label}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
    </FieldShell>
  );
}

export function LabelledChoices<T>({
  label,
  meta,
  options,
  current,
  unmatchedLabel,
  onPick,
}: {
  label: string;
  meta?: FieldMeta;
  options: ReadonlyArray<{ label: string; value: T }>;
  current: string | undefined;
  unmatchedLabel?: string;
  onPick: (value: T) => void;
}) {
  const choices: Choice[] = options.map((o) => ({ value: o.label, label: o.label }));
  if (unmatchedLabel)
    choices.push({ value: unmatchedLabel, label: unmatchedLabel, disabled: true });
  const pressed = current ?? unmatchedLabel;
  return (
    <ChoiceGroup
      label={label}
      meta={meta}
      choices={choices}
      value={pressed === undefined ? [] : [pressed]}
      onChange={(v) => {
        const hit = options.find((o) => o.label === v[0]);
        if (hit) onPick(hit.value);
      }}
    />
  );
}
