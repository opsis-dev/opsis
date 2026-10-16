import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/renderer/components/ui/select";
import { FieldShell } from "./field-shell";
import { NONE, fromNone, orNone } from "./none";
import type { FieldProps } from "./types";

const NOT_SIGNED_IN = "Not signed in";

export function SecretField({ id, label, value, onChange, meta, ctx }: FieldProps<string>) {
  const names = [...new Set([...(ctx.secrets ?? []), ...(value ? [value] : [])])];
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <Select
        items={[
          { value: NONE, label: NOT_SIGNED_IN },
          ...names.map((n) => ({ value: n, label: n })),
        ]}
        value={orNone(value)}
        onValueChange={(v) => onChange(fromNone(v))}
      >
        <SelectTrigger id={id} className="w-full">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            <SelectItem value={NONE}>{NOT_SIGNED_IN}</SelectItem>
            {names.map((n) => (
              <SelectItem key={n} value={n}>
                {n}
                {ctx.secrets && !ctx.secrets.includes(n) ? (
                  <span className="text-muted-foreground ml-1 text-xs">(not saved yet)</span>
                ) : null}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>
    </FieldShell>
  );
}
