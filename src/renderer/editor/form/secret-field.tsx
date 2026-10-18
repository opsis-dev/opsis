import { withCurrent } from "@/shared/lists";
import { ChoiceSelect } from "./choice-select";
import { FieldShell } from "./field-shell";
import type { FieldProps } from "./types";

const NOT_SIGNED_IN = "Not signed in";

export function SecretField({ id, label, value, onChange, meta, ctx }: FieldProps<string>) {
  const names = withCurrent(ctx.secrets ?? [], value);
  const unsaved = (name: string) => !!ctx.secrets && !ctx.secrets.includes(name);
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <ChoiceSelect
        id={id}
        groups={[
          names.map((n) => ({
            value: n,
            label: n,
            hint: unsaved(n) ? "(not saved yet)" : undefined,
          })),
        ]}
        value={value}
        onChange={onChange}
        noneLabel={NOT_SIGNED_IN}
      />
    </FieldShell>
  );
}
