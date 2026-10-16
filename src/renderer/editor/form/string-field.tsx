import { Crosshair, FolderOpen } from "lucide-react";
import type { ComponentType } from "react";
import { humanize } from "@/shared/format";
import { BACKDROPS } from "@/video/camera/lib/backdrop";
import { Input } from "@/renderer/components/ui/input";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/renderer/components/ui/input-group";
import { Textarea } from "@/renderer/components/ui/textarea";
import { FontPicker } from "../../fonts/font-picker";
import { VarText } from "../var-text";
import { EnumField } from "./enum-field";
import { FieldShell } from "./field-shell";
import { KeysField } from "./keys-field";
import { NONE, fromNone, orNone } from "./none";
import { OffsetsField } from "./offsets-field";
import { SecretField } from "./secret-field";
import { SoundField } from "./sound-field";
import { TimezoneField } from "./timezone-field";
import type { FieldProps } from "./types";

type StringProps = FieldProps<string>;

function ActionField({ id, label, value, onChange, meta, ctx }: StringProps) {
  const actions = ctx.actions ?? [];
  const names = new Map(actions.map((a) => [a.id, a.label]));
  return (
    <EnumField
      id={id}
      label={label}
      options={[NONE, ...actions.map((a) => a.id)]}
      value={orNone(value)}
      onChange={(v) => onChange(fromNone(v))}
      format={(v) => (v === NONE ? "None, use a time" : (names.get(v) ?? "Missing action"))}
      meta={meta}
      forceSelect
    />
  );
}

function CodeField({ id, label, value, onChange, meta }: StringProps) {
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

function AssetField({ id, name, label, value, onChange, meta, ctx }: StringProps) {
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <InputGroup>
        <InputGroupInput
          id={id}
          value={value}
          placeholder="Choose a file"
          onChange={(e) => onChange(e.target.value)}
        />
        <InputGroupAddon align="inline-end">
          <InputGroupButton
            size="xs"
            onClick={() => ctx.chooseAsset?.(name)}
            disabled={!ctx.projectDir}
          >
            <FolderOpen data-icon="inline-start" />
            Choose
          </InputGroupButton>
        </InputGroupAddon>
      </InputGroup>
    </FieldShell>
  );
}

function FontField({ id, label, value, onChange, meta }: StringProps) {
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <FontPicker id={id} value={value} onChange={onChange} emptyLabel="Project font" />
    </FieldShell>
  );
}

function TargetField({ id, name, label, value, onPatch, meta, ctx }: StringProps) {
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <InputGroup>
        <InputGroupInput
          id={id}
          value={value}
          placeholder="Its visible text"
          onChange={(e) => onPatch({ [name]: e.target.value, [`${name}Role`]: undefined })}
        />
        <InputGroupAddon align="inline-end">
          <InputGroupButton size="xs" onClick={() => ctx.pickTarget?.(name)}>
            <Crosshair data-icon="inline-start" />
            Pick
          </InputGroupButton>
        </InputGroupAddon>
      </InputGroup>
    </FieldShell>
  );
}

function DatetimeField({ id, label, value, onChange, meta }: StringProps) {
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

function VariableField({ id, label, value, onChange, meta, ctx }: StringProps) {
  const options = [...new Set([...ctx.vars.map((v) => v.name), ...(value ? [value] : [])])];
  return (
    <EnumField
      id={id}
      label={label}
      options={options}
      value={value}
      onChange={onChange}
      meta={meta}
      forceSelect
    />
  );
}

function BackdropField({ id, label, value, onChange, meta }: StringProps) {
  return (
    <EnumField
      id={id}
      label={label}
      options={[...BACKDROPS]}
      value={value}
      onChange={onChange}
      format={humanize}
      meta={meta}
      forceSelect
    />
  );
}

function TextField({ id, label, value, onChange, meta, ctx }: StringProps) {
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <VarText
        id={id}
        value={value}
        onChange={onChange}
        vars={ctx.vars}
        multiline={meta.widget === "textarea"}
        pauses={meta.pauses}
      />
    </FieldShell>
  );
}

const STRING_WIDGETS: Partial<Record<string, ComponentType<StringProps>>> = {
  action: ActionField,
  code: CodeField,
  asset: AssetField,
  font: FontField,
  sound: SoundField,
  target: TargetField,
  secret: SecretField,
  timezone: TimezoneField,
  keys: KeysField,
  datetime: DatetimeField,
  variable: VariableField,
  backdrop: BackdropField,
  offsets: OffsetsField,
};

export function StringField(props: StringProps) {
  const Widget = (props.meta.widget && STRING_WIDGETS[props.meta.widget]) || TextField;
  return <Widget {...props} />;
}
