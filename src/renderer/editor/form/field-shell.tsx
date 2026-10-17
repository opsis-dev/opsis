import type { ReactNode } from "react";
import type { FieldMeta } from "@/shared/elements/introspect";
import { Field, FieldDescription, FieldLabel } from "@/renderer/components/ui/field";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";

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

type Choice = { value: string; label: string; disabled?: boolean };

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

type LabelledOption<T> = { label: string; value: T; disabled?: boolean };

export function LabelledChoices<T>({
  label,
  meta,
  options,
  current,
  onPick,
}: {
  label: string;
  meta?: FieldMeta;
  options: ReadonlyArray<LabelledOption<T>>;
  current: string;
  onPick: (value: T) => void;
}) {
  return (
    <ChoiceGroup
      label={label}
      meta={meta}
      choices={options.map((o) => ({ value: o.label, label: o.label, disabled: o.disabled }))}
      value={[current]}
      onChange={(v) => {
        const hit = options.find((o) => o.label === v[0]);
        if (hit && !hit.disabled) onPick(hit.value);
      }}
    />
  );
}
