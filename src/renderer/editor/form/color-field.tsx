import { InputGroup, InputGroupAddon, InputGroupInput } from "@/renderer/components/ui/input-group";
import { FieldShell } from "./field-shell";
import type { FieldProps } from "./types";

export function ColorField({ id, label, value, onChange, meta }: FieldProps<string>) {
  const hex = /^#[0-9a-f]{6}$/i.test(value) ? value : "#000000";
  return (
    <FieldShell id={id} label={label} meta={meta}>
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
    </FieldShell>
  );
}
