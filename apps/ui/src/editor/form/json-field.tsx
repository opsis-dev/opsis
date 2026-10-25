import { tryParseJson } from "@opsis/core/guards";
import { useState } from "react";
import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";
import { Textarea } from "@/components/ui/textarea";
import { Help } from "./field-shell";
import type { FieldProps } from "./types";

const INVALID = "This isn't valid. Check brackets and quotes.";

export function JsonField({ id, label, value, onChange, meta }: FieldProps<unknown>) {
  const [draft, setDraft] = useState<string | null>(null);
  const [invalid, setInvalid] = useState(false);
  return (
    <Field data-invalid={invalid || undefined}>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <Textarea
        id={id}
        aria-invalid={invalid || undefined}
        className="font-mono text-xs"
        rows={4}
        value={draft ?? JSON.stringify(value ?? null, null, 2)}
        onChange={(e) => {
          setDraft(e.target.value);
          const parsed = tryParseJson(e.target.value);
          setInvalid(!parsed.ok);
          if (parsed.ok) onChange(parsed.value);
        }}
        onBlur={() => {
          if (!invalid) setDraft(null);
        }}
      />
      {invalid ? <FieldDescription>{INVALID}</FieldDescription> : null}
      <Help meta={meta} />
    </Field>
  );
}
