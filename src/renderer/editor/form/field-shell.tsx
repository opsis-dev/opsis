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

export type Choice = { value: string; label: string; disabled?: boolean };

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
