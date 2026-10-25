import { ChoiceSelect } from "./choice-select";
import { ChoiceGroup, FieldShell } from "./field-shell";
import type { FieldProps } from "./types";

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
  noneLabel,
}: FieldProps<string> & {
  options: string[];
  format?: (v: string) => string;
  forceSelect?: boolean;
  noneLabel?: string;
}) {
  const choices = options.map((o) => ({ value: o, label: format(o) }));
  const short =
    !forceSelect &&
    !noneLabel &&
    options.length <= SHORT_OPTIONS &&
    choices.reduce((n, c) => n + c.label.length, 0) <= SHORT_TEXT;
  if (short)
    return (
      <ChoiceGroup
        label={label}
        meta={meta}
        choices={choices}
        value={[value]}
        onChange={(v) => {
          if (v[0]) onChange(v[0]);
        }}
      />
    );
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <ChoiceSelect
        id={id}
        groups={[choices]}
        value={value}
        onChange={onChange}
        noneLabel={noneLabel}
        placeholder="Choose…"
      />
    </FieldShell>
  );
}
