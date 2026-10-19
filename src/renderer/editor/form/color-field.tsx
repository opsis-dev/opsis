import { X } from "lucide-react";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/renderer/components/ui/input-group";
import { ColorChip, ColorPopover } from "./color-picker";
import { FieldShell } from "./field-shell";
import type { FieldProps } from "./types";

export function ColorInput({
  id,
  label,
  value,
  onChange,
  emptyLabel,
  fallback,
  swatches,
}: {
  id?: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  emptyLabel?: string;
  fallback?: string;
  swatches: string[];
}) {
  const shown = value || fallback || "";
  return (
    <InputGroup>
      <InputGroupAddon>
        <ColorPopover
          label={label}
          value={value}
          placeholder={fallback}
          onChange={onChange}
          swatches={swatches}
          trigger={<ColorChip value={shown} />}
        />
      </InputGroupAddon>
      <InputGroupInput
        id={id}
        value={value}
        placeholder={emptyLabel}
        title={!value && fallback ? `${emptyLabel ?? ""} ${fallback}`.trim() : undefined}
        spellCheck={false}
        onChange={(e) => onChange(e.target.value)}
        className="font-mono text-xs"
      />
      {emptyLabel && value ? (
        <InputGroupAddon align="inline-end">
          <InputGroupButton
            size="icon-xs"
            aria-label={`Use the ${emptyLabel.toLowerCase()} colour`}
            title={`Use the ${emptyLabel.toLowerCase()} colour`}
            onClick={() => onChange("")}
          >
            <X />
          </InputGroupButton>
        </InputGroupAddon>
      ) : null}
    </InputGroup>
  );
}

export function ColorField({ id, label, value, onChange, meta, ctx }: FieldProps<string>) {
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <ColorInput
        id={id}
        label={label}
        value={value}
        onChange={onChange}
        emptyLabel={meta.zeroLabel}
        swatches={ctx.swatches}
      />
    </FieldShell>
  );
}
