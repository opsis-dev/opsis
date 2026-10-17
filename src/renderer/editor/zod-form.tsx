import { ChevronRight, Plus, Trash2 } from "lucide-react";
import type { ComponentType, ReactNode } from "react";
import type { z } from "zod";
import {
  fieldMeta,
  schemaDefaults,
  type FieldMeta,
  type FieldWidget,
} from "@/shared/elements/introspect";
import { humanize } from "@/shared/format";
import { isBoolean, isNumber, isRecord, isString, isStringOrNumber } from "@/shared/guards";
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
import { newKeyedItem, useItemKeys } from "@/renderer/lib/use-item-keys";
import { BoxesField, isBoxList } from "./form/boxes-field";
import { ColorField } from "./form/color-field";
import { EnumField } from "./form/enum-field";
import { EnvelopeField, isEnvelope } from "./form/envelope-field";
import { ChoiceGroup, Help } from "./form/field-shell";
import { JsonField } from "./form/json-field";
import { NumberControl } from "./form/number-field";
import { Presets } from "./form/presets";
import { schemaKind } from "./form/schema-kind";
import { isColor, labelFor } from "./form/schema-meta";
import { StringField } from "./form/string-field";
import type { FieldProps, FormContext, Patch } from "./form/types";

type Entry = { key: string; field: z.ZodType; meta: FieldMeta };

type AnyFieldProps = FieldProps<unknown>;

function checked<V, X extends object = object>(
  is: (v: unknown) => v is V,
  empty: (props: X) => V,
  Component: ComponentType<FieldProps<V> & NoInfer<X>>,
): ComponentType<AnyFieldProps & X> {
  return function CheckedField(props) {
    const value = props.value ?? empty(props);
    return is(value) ? <Component {...props} value={value} /> : <JsonField {...props} />;
  };
}

const BoxesKindField = checked(isBoxList, () => [], BoxesField);
const EnvelopeKindField = checked(isEnvelope, () => ({}), EnvelopeField);

const VALUE_WIDGETS: Partial<Record<FieldWidget, ComponentType<AnyFieldProps>>> = {
  boxes: BoxesKindField,
  envelope: EnvelopeKindField,
};

const COLLAPSING =
  "h-(--collapsible-panel-height) overflow-hidden transition-[height,opacity] duration-200 ease-out data-ending-style:h-0 data-ending-style:opacity-0 data-starting-style:h-0 data-starting-style:opacity-0";

const isStringList = (v: unknown): v is string[] => Array.isArray(v) && v.every(isString);
const isRecordList = (v: unknown): v is Patch[] => Array.isArray(v) && v.every(isRecord);

function visible(meta: FieldMeta, value: Patch): boolean {
  if (meta.hidden) return false;
  if (!meta.showIf) return true;
  return Object.entries(meta.showIf).every(([k, allowed]) => {
    const v = value[k] == null ? "" : String(value[k]);
    return allowed.includes(v) || (allowed.includes("*") && v !== "");
  });
}

export function ZodForm({
  schema,
  value,
  onChange,
  ctx,
  omit = [],
}: {
  schema: z.ZodObject;
  value: Patch;
  onChange: (patch: Patch) => void;
  ctx: FormContext;
  omit?: string[];
}) {
  const entries = Object.entries(schema.shape)
    .filter(([key]) => !omit.includes(key))
    .map(([key, field]: [string, z.ZodType]): Entry => ({ key, field, meta: fieldMeta(field) }))
    .filter(({ meta }) => visible(meta, value));
  if (entries.length === 0) return <p className="text-muted-foreground text-sm">No settings.</p>;
  const basic = entries.filter(({ meta }) => !meta.advanced);
  const advanced = entries.filter(({ meta }) => meta.advanced);
  const render = (list: Entry[]) =>
    list.map(({ key, field }) => (
      <FieldControl
        key={key}
        name={key}
        schema={field}
        value={value[key]}
        all={value}
        onChange={(v) => onChange({ [key]: v })}
        onPatch={onChange}
        ctx={ctx}
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

function FieldControl({
  schema,
  ...rest
}: Omit<AnyFieldProps, "id" | "label" | "meta"> & { schema: z.ZodType }) {
  const meta = fieldMeta(schema);
  const props: AnyFieldProps = {
    ...rest,
    id: `f-${rest.name}`,
    label: labelFor(rest.name, schema),
    meta,
  };
  return (
    <>
      {meta.presets ? <Presets meta={meta} all={rest.all} onPatch={rest.onPatch} /> : null}
      <FieldBody schema={schema} props={props} />
    </>
  );
}

function FieldBody({ schema, props }: { schema: z.ZodType; props: AnyFieldProps }) {
  const { meta } = props;
  if (isColor(schema)) return <ColorKindField {...props} />;
  const Widget = meta.widget ? VALUE_WIDGETS[meta.widget] : undefined;
  if (Widget) return <Widget {...props} />;
  const kind = schemaKind(schema);
  switch (kind.kind) {
    case "boolean":
      return <BooleanKindField {...props} />;
    case "number":
      return <NumberKindField {...props} min={kind.min} max={kind.max} />;
    case "string":
      return <StringKindField {...props} />;
    case "enum":
      return <EnumKindField {...props} options={kind.options} />;
    case "literals":
      return <LiteralsField {...props} options={kind.options} />;
    case "enumList":
      return <EnumListField {...props} options={kind.options} />;
    case "objectList":
      return <ObjectListKindField {...props} item={kind.item} />;
    default:
      return <JsonField {...props} />;
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

function EnumChoice(props: FieldProps<string> & Options<string>) {
  return <EnumField {...props} format={enumFormat(props.meta)} />;
}

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
      onChange={(v) => {
        const picked = options.find((o) => String(o) === v);
        if (picked !== undefined) onChange(picked);
      }}
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

const ColorKindField = checked(isString, () => "", ColorField);
const StringKindField = checked(isString, () => "", StringField);
const BooleanKindField = checked(isBoolean, () => false, BooleanField);
const NumberKindField = checked<number, { min?: number; max?: number }>(
  isNumber,
  () => 0,
  NumberControl,
);
const EnumKindField = checked<string, Options<string>>(
  isString,
  ({ options }) => options[0] ?? "",
  EnumChoice,
);
const LiteralsField = checked<string | number, Options<string | number>>(
  isStringOrNumber,
  () => "",
  LiteralChoice,
);
const EnumListField = checked<string[], Options<string>>(isStringList, () => [], EnumListChoice);
const ObjectListKindField = checked<Patch[], { item: z.ZodObject }>(
  isRecordList,
  () => [],
  ObjectListField,
);

function ObjectListField({
  label,
  meta,
  item,
  value,
  onChange,
  ctx,
}: FieldProps<Patch[]> & { item: z.ZodObject }) {
  const { rows, commit } = useItemKeys(value);
  return (
    <FieldSet>
      <FieldLegend variant="label">{label}</FieldLegend>
      <Help meta={meta} />
      <div className="flex flex-col gap-3">
        {rows.map(({ item: entry, key }, i) => (
          <div key={key} className="bg-muted/40 flex flex-col gap-3 rounded-lg border p-3">
            <ZodForm
              schema={item}
              value={entry}
              ctx={ctx}
              onChange={(patch) =>
                onChange(value.map((v, j) => (j === i ? { ...v, ...patch } : v)))
              }
            />
            <Button
              variant="ghost"
              size="sm"
              onClick={() => onChange(commit(rows.toSpliced(i, 1)))}
            >
              <Trash2 data-icon="inline-start" />
              Remove
            </Button>
          </div>
        ))}
        <Button
          variant="outline"
          size="sm"
          onClick={() => onChange(commit([...rows, newKeyedItem(schemaDefaults(item))]))}
        >
          <Plus data-icon="inline-start" />
          Add
        </Button>
      </div>
    </FieldSet>
  );
}
