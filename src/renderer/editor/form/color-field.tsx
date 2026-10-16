import { Field, FieldLabel } from "@/renderer/components/ui/field";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/renderer/components/ui/input-group";
import type { FieldProps } from "./types";

export function ColorField({ id, label, value, onChange }: FieldProps<string>) {
  const hex = /^#[0-9a-f]{6}$/i.test(value) ? value : "#000000";
  return (
    <Field>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <InputGroup>
        <InputGroupAddon>
          <input
            type="color"
            aria-label={`${label} swatch`}
            value={hex}
            onChange={(e) => onChange(e.target.value)}
            className="size-5 cursor-pointer rounded border-0 bg-transparent p-0"
          />
        </InputGroupAddon>
        <InputGroupInput
          id={id}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="font-mono text-xs"
        />
      </InputGroup>
    </Field>
  );
}
