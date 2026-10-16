import type { FieldMeta } from "@/shared/elements/introspect";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/renderer/components/ui/select";
import { ChoiceGroup, FieldShell } from "./field-shell";

const SHORT_OPTIONS = 4;
const SHORT_TEXT = 34;

export function EnumField({
  id,
  label,
  options,
  value,
  onChange,
  format = (v: string) => v,
  meta,
  forceSelect,
}: {
  id: string;
  label: string;
  options: string[];
  value: string;
  onChange: (v: string) => void;
  format?: (v: string) => string;
  meta: FieldMeta;
  forceSelect?: boolean;
}) {
  const choices = options.map((o) => ({ value: o, label: format(o) }));
  const short =
    !forceSelect &&
    options.length <= SHORT_OPTIONS &&
    choices.reduce((n, c) => n + c.label.length, 0) <= SHORT_TEXT;
  if (short)
    return (
      <ChoiceGroup
        label={label}
        meta={meta}
        choices={choices}
        value={[value]}
        onChange={(v) => v[0] && onChange(v[0])}
      />
    );
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <Select items={choices} value={value} onValueChange={(v) => v != null && onChange(String(v))}>
        <SelectTrigger id={id} className="w-full">
          <SelectValue placeholder="Choose…" />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            {choices.map((c) => (
              <SelectItem key={c.value} value={c.value}>
                {c.label}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>
    </FieldShell>
  );
}
