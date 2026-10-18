import { ChevronRight, Plus, Trash2 } from "lucide-react";
import type { ComponentType, ReactNode } from "react";
import type { z } from "zod";
import {
  ANY_VALUE,
  fieldMeta,
  schemaDefaults,
  shapeEntries,
  withDefaults,
  type FieldMeta,
  type FieldWidget,
} from "@/shared/elements/introspect";
import { humanize } from "@/shared/format";
import { LOUDNESS_PRESETS, QUALITY_PRESETS } from "@/shared/nodes/params";
import { transitionTitle } from "@/shared/nodes/transitions";
import { Button } from "@/renderer/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/renderer/components/ui/collapsible";
import {
  Field,
  FieldGroup,
  FieldLabel,
  FieldLegend,
  FieldSet,
} from "@/renderer/components/ui/field";
import { Switch } from "@/renderer/components/ui/switch";
import { whenOneOf } from "@/renderer/lib/one-of";
import { useItemKeys } from "@/renderer/lib/use-item-keys";
import { BoxesField } from "./form/boxes-field";
import { ColorField } from "./form/color-field";
import { EnumField } from "./form/enum-field";
import { EnvelopeField } from "./form/envelope-field";
import { ChoiceGroup, Help } from "./form/field-shell";
import { JsonField } from "./form/json-field";
import { KeysField } from "./form/keys-field";
import { NumberField, PresetNumber, type NumberProps } from "./form/number-field";
import { OffsetsField } from "./form/offsets-field";
import { Presets } from "./form/presets";
import { typedField } from "./form/schema-kind";
import { labelFor } from "./form/schema-meta";
import { SecretField } from "./form/secret-field";
import { SoundField } from "./form/sound-field";
import { SpotField } from "./form/spot-field";
import {
  actionChoices,
  AssetField,
  backdropChoices,
  CodeField,
  DatetimeField,
  FontField,
  TargetField,
  TextField,
  variableChoices,
  type SelectChoices,
} from "./form/string-field";
import { TimezoneField } from "./form/timezone-field";
import type { FieldProps, FormContext, ParamFieldProps, ParamValues } from "./form/types";

type Entry = { key: string; field: z.ZodType; meta: FieldMeta };

type Widget =
  | { kind: "string"; Field: ComponentType<ParamFieldProps<string>> }
  | { kind: "select"; choices: (ctx: FormContext, value: string) => SelectChoices }
  | { kind: "number"; Field: ComponentType<ParamFieldProps<number> & NumberProps> }
  | { kind: "presets"; presets: Readonly<Record<string, number>> };

const WIDGETS: Partial<Record<FieldWidget, Widget>> = {
  action: { kind: "select", choices: actionChoices },
  variable: { kind: "select", choices: variableChoices },
  backdrop: { kind: "select", choices: backdropChoices },
  code: { kind: "string", Field: CodeField },
  asset: { kind: "string", Field: AssetField },
  font: { kind: "string", Field: FontField },
  sound: { kind: "string", Field: SoundField },
  target: { kind: "string", Field: TargetField },
  secret: { kind: "string", Field: SecretField },
  timezone: { kind: "string", Field: TimezoneField },
  keys: { kind: "string", Field: KeysField },
  datetime: { kind: "string", Field: DatetimeField },
  offsets: { kind: "string", Field: OffsetsField },
  spot: { kind: "number", Field: SpotField },
  quality: { kind: "presets", presets: QUALITY_PRESETS },
  loudness: { kind: "presets", presets: LOUDNESS_PRESETS },
};

const COLLAPSING =
  "h-(--collapsible-panel-height) overflow-hidden transition-[height,opacity] duration-200 ease-out data-ending-style:h-0 data-ending-style:opacity-0 data-starting-style:h-0 data-starting-style:opacity-0";

function visible(meta: FieldMeta, values: ParamValues): boolean {
  if (meta.hidden) return false;
  if (!meta.showIf) return true;
  return Object.entries(meta.showIf).every(([k, allowed]) => {
    const v = values[k] == null ? "" : String(values[k]);
    return allowed.includes(v) || (allowed.includes(ANY_VALUE) && v !== "");
  });
}

export function ZodForm({
  schema,
  value,
  onChange,
  ctx,
}: {
  schema: z.ZodObject;
  value: ParamValues;
  onChange: (patch: ParamValues) => void;
  ctx: FormContext;
}) {
  const values = withDefaults(schema, value);
  const entries = shapeEntries(schema)
    .map(([key, field]): Entry => ({ key, field, meta: fieldMeta(field) }))
    .filter(({ meta }) => visible(meta, values));
  if (entries.length === 0) return <p className="text-muted-foreground text-sm">No settings.</p>;
  const basic = entries.filter(({ meta }) => !meta.advanced);
  const advanced = entries.filter(({ meta }) => meta.advanced);
  const render = (list: Entry[]) =>
    list.map(({ key, field, meta }) => (
      <FieldControl
        key={key}
        schema={field}
        props={{
          id: `f-${key}`,
          name: key,
          label: labelFor(key, field),
          meta,
          value: values[key],
          all: values,
          onChange: (v) => onChange({ [key]: v }),
          onPatch: onChange,
          ctx,
        }}
      />
    ));
  return (
    <FieldGroup className="gap-4">
      {render(basic)}
      {advanced.length ? <AdvancedSection>{render(advanced)}</AdvancedSection> : null}
    </FieldGroup>
  );
}

function AdvancedSection({ children }: { children: ReactNode }) {
  return (
    <Collapsible>
      <CollapsibleTrigger
        render={
          <Button variant="ghost" size="sm" className="group/adv text-muted-foreground -ml-2 w-fit">
            <ChevronRight
              data-icon="inline-start"
              className="transition-transform group-data-[panel-open]/adv:rotate-90"
            />
            Advanced
          </Button>
        }
      />
      <CollapsibleContent className={COLLAPSING}>
        <FieldGroup className="gap-4 pt-3">{children}</FieldGroup>
      </CollapsibleContent>
    </Collapsible>
  );
}

function FieldControl({ schema, props }: { schema: z.ZodType; props: ParamFieldProps<unknown> }) {
  const { presets, presetsLabel } = props.meta;
  return (
    <>
      {presets ? (
        <Presets presets={presets} label={presetsLabel} all={props.all} onPatch={props.onPatch} />
      ) : null}
      <FieldBody schema={schema} props={props} />
    </>
  );
}

function FieldBody({ schema, props }: { schema: z.ZodType; props: ParamFieldProps<unknown> }) {
  const field = typedField(schema, props.value);
  if (!field) return <JsonField {...props} />;
  const widget = props.meta.widget ? WIDGETS[props.meta.widget] : undefined;
  switch (field.kind) {
    case "color":
      return <ColorField {...props} value={field.value} />;
    case "boxes":
      return <BoxesField {...props} value={field.value} />;
    case "envelope":
      return <EnvelopeField {...props} value={field.value} />;
    case "boolean":
      return <BooleanField {...props} value={field.value} />;
    case "number": {
      if (widget?.kind === "presets")
        return <PresetNumber {...props} value={field.value} presets={widget.presets} />;
      const NumberWidget = widget?.kind === "number" ? widget.Field : NumberField;
      return <NumberWidget {...props} value={field.value} min={field.min} max={field.max} />;
    }
    case "string": {
      if (widget?.kind === "select")
        return (
          <EnumField
            {...props}
            {...widget.choices(props.ctx, field.value)}
            value={field.value}
            forceSelect
          />
        );
      const TextWidget = widget?.kind === "string" ? widget.Field : TextField;
      return <TextWidget {...props} value={field.value} />;
    }
    case "enum":
      return (
        <EnumField
          {...props}
          value={field.value}
          options={field.options}
          format={enumFormat(props.meta)}
        />
      );
    case "literals":
      return <LiteralChoice {...props} value={field.value} options={field.options} />;
    case "enumList":
      return <EnumListChoice {...props} value={field.value} options={field.options} />;
    case "objectList":
      return <ObjectListField {...props} value={field.value} item={field.item} />;
  }
}

function BooleanField({ id, label, value, onChange }: FieldProps<boolean>) {
  return (
    <Field orientation="horizontal" className="justify-between">
      <FieldLabel htmlFor={id} className="font-normal">
        {label}
      </FieldLabel>
      <Switch id={id} checked={value} onCheckedChange={onChange} />
    </Field>
  );
}

function enumFormat(meta: FieldMeta): (v: string) => string {
  if (meta.widget === "transition") return transitionTitle;
  return (v) => meta.labels?.[v] ?? humanize(v);
}

type Options<O> = { options: O[] };

function LiteralChoice({
  options,
  value,
  onChange,
  ...props
}: FieldProps<string | number> & Options<string | number>) {
  const { unit } = props.meta;
  return (
    <EnumField
      {...props}
      options={options.map(String)}
      value={String(value)}
      onChange={whenOneOf(options, onChange)}
      format={(v) => (unit ? `${v} ${unit}` : v)}
    />
  );
}

function EnumListChoice({
  label,
  meta,
  value,
  onChange,
  options,
}: FieldProps<string[]> & Options<string>) {
  return (
    <ChoiceGroup
      multiple
      label={label}
      meta={meta}
      choices={options.map((o) => ({ value: o, label: meta.labels?.[o] ?? o }))}
      value={value}
      onChange={onChange}
    />
  );
}

function ObjectListField({
  label,
  meta,
  item,
  value,
  onChange,
  ctx,
}: FieldProps<ParamValues[]> & { item: z.ZodObject }) {
  const list = useItemKeys(value);
  return (
    <FieldSet>
      <FieldLegend variant="label">{label}</FieldLegend>
      <Help meta={meta} />
      <div className="flex flex-col gap-3">
        {list.rows.map(({ item: entry, key }, i) => (
          <div key={key} className="bg-muted/40 flex flex-col gap-3 rounded-lg border p-3">
            <ZodForm
              schema={item}
              value={entry}
              ctx={ctx}
              onChange={(patch) => onChange(list.update(i, { ...entry, ...patch }))}
            />
            <Button variant="ghost" size="sm" onClick={() => onChange(list.remove(i))}>
              <Trash2 data-icon="inline-start" />
              Remove
            </Button>
          </div>
        ))}
        <Button
          variant="outline"
          size="sm"
          onClick={() => onChange(list.add(schemaDefaults(item)))}
        >
          <Plus data-icon="inline-start" />
          Add
        </Button>
      </div>
    </FieldSet>
  );
}
