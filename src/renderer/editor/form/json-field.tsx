import { useState } from "react";
import { Field, FieldDescription, FieldLabel } from "@/renderer/components/ui/field";
import { Textarea } from "@/renderer/components/ui/textarea";
import type { FieldProps } from "./types";

const INVALID = "This isn't valid. Check brackets and quotes.";

function parseJson(text: string): { ok: true; value: unknown } | { ok: false } {
  try {
    return { ok: true, value: JSON.parse(text) };
  } catch {
    return { ok: false };
  }
}

export function JsonField({ id, label, value, onChange }: FieldProps<unknown>) {
  const [draft, setDraft] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  return (
    <Field data-invalid={error ? true : undefined}>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <Textarea
        id={id}
        aria-invalid={error ? true : undefined}
        className="font-mono text-xs"
        rows={4}
        value={draft ?? JSON.stringify(value ?? null, null, 2)}
        onChange={(e) => {
          setDraft(e.target.value);
          const parsed = parseJson(e.target.value);
          setError(parsed.ok ? null : INVALID);
          if (parsed.ok) onChange(parsed.value);
        }}
        onBlur={() => {
          if (!error) setDraft(null);
        }}
      />
      {error ? <FieldDescription>{error}</FieldDescription> : null}
    </Field>
  );
}
