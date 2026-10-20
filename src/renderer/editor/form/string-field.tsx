import { Crosshair, FolderOpen, type LucideIcon } from "lucide-react";
import { Input } from "@/renderer/components/ui/input";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/renderer/components/ui/input-group";
import { Textarea } from "@/renderer/components/ui/textarea";
import { BACKDROPS } from "@/shared/backdrops";
import { targetRoleKey } from "@/shared/elements/introspect";
import { humanize } from "@/shared/format";
import { withCurrent } from "@/shared/lists";
import { FontPicker } from "../../fonts/font-picker";
import { VarText } from "../var-text";
import { FieldShell } from "./field-shell";
import type { FieldProps, FormContext, ParamFieldProps } from "./types";

type StringProps = FieldProps<string>;

export type SelectChoices = {
  options: string[];
  format?: (value: string) => string;
  noneLabel?: string;
};

export function actionChoices(ctx: FormContext): SelectChoices {
  const names = new Map(ctx.actions.map((a) => [a.id, a.label]));
  return {
    options: ctx.actions.map((a) => a.id),
    format: (v) => names.get(v) ?? "Missing action",
    noneLabel: "None, use a time",
  };
}

export const variableChoices = (ctx: FormContext, value: string): SelectChoices => ({
  options: withCurrent(
    ctx.vars.map((v) => v.name),
    value,
  ),
});

export const backdropChoices = (): SelectChoices => ({ options: [...BACKDROPS], format: humanize });

export function CodeField({ id, label, value, onChange, meta }: StringProps) {
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <Textarea
        id={id}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        rows={6}
        className="font-mono text-xs"
        spellCheck={false}
      />
    </FieldShell>
  );
}

type InputAction = { icon: LucideIcon; label: string; onClick: () => void; disabled?: boolean };

function InputWithButton({
  id,
  value,
  placeholder,
  onChange,
  action: { icon: Icon, label, onClick, disabled },
}: {
  id: string;
  value: string;
  placeholder: string;
  onChange: (value: string) => void;
  action: InputAction;
}) {
  return (
    <InputGroup>
      <InputGroupInput
        id={id}
        value={value}
        placeholder={placeholder}
        onChange={(e) => onChange(e.target.value)}
      />
      <InputGroupAddon align="inline-end">
        <InputGroupButton size="xs" onClick={onClick} disabled={disabled}>
          <Icon data-icon="inline-start" />
          {label}
        </InputGroupButton>
      </InputGroupAddon>
    </InputGroup>
  );
}

export function AssetField({ id, label, value, onChange, meta, ctx }: StringProps) {
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <InputWithButton
        id={id}
        value={value}
        placeholder="Choose a file"
        onChange={onChange}
        action={{
          icon: FolderOpen,
          label: "Choose",
          onClick: () => ctx.chooseAsset(onChange),
          disabled: !ctx.projectDir,
        }}
      />
    </FieldShell>
  );
}

export function FontField({ id, label, value, onChange, meta }: StringProps) {
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <FontPicker id={id} value={value} onChange={onChange} emptyLabel="Project font" />
    </FieldShell>
  );
}

export function TargetField({
  id,
  name,
  label,
  value,
  onPatch,
  meta,
  ctx,
}: ParamFieldProps<string>) {
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <InputWithButton
        id={id}
        value={value}
        placeholder="Its visible text"
        onChange={(text) => onPatch({ [name]: text, [targetRoleKey(name)]: undefined })}
        action={{ icon: Crosshair, label: "Pick", onClick: () => ctx.pickTarget(name) }}
      />
    </FieldShell>
  );
}

export function DatetimeField({ id, label, value, onChange, meta }: StringProps) {
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <Input
        id={id}
        type="datetime-local"
        step={60}
        value={value.slice(0, 16)}
        onChange={(e) => e.target.value && onChange(`${e.target.value}:00`)}
      />
    </FieldShell>
  );
}

export function TextField({ id, label, value, onChange, meta, ctx }: StringProps) {
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <VarText
        id={id}
        value={value}
        onChange={onChange}
        vars={ctx.vars}
        multiline={meta.widget === "textarea"}
        pauses={meta.pauses}
        placeholder={meta.zeroLabel}
      />
    </FieldShell>
  );
}
