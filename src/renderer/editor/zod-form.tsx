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
import { isRecord } from "@/shared/guards";
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
import { useItemKeys } from "@/renderer/lib/use-item-keys";
import { BoxesField } from "./form/boxes-field";
import { ColorField } from "./form/color-field";
import { EnumField } from "./form/enum-field";
import { EnvelopeField, isEnvelope } from "./form/envelope-field";
import { ChoiceGroup, Help } from "./form/field-shell";
import { JsonField } from "./form/json-field";
import { NumberControl } from "./form/number-field";
import { schemaKind } from "./form/schema-kind";
import { isColor, labelFor } from "./form/schema-meta";
import { StringField } from "./form/string-field";
import type { FieldProps, FormContext, Patch } from "./form/types";
import { isBoxList } from "./form/use-box-drag";

type Entry = { key: string; field: z.ZodType; meta: FieldMeta };

type AnyFieldProps = FieldProps<unknown>;

function BoxesKindField(props: AnyFieldProps) {
  const field = typed(props, isBoxList, []);
  return field ? <BoxesField {...field} /> : <JsonField {...props} />;
}

function EnvelopeKindField(props: AnyFieldProps) {
  const field = typed(props, isEnvelope, {});
  return field ? <EnvelopeField {...field} /> : <JsonField {...props} />;
}

const VALUE_WIDGETS: Partial<Record<FieldWidget, ComponentType<AnyFieldProps>>> = {
  boxes: BoxesKindField,
  envelope: EnvelopeKindField,
};

const COLLAPSING =
  "h-(--collapsible-panel-height) overflow-hidden transition-[height,opacity] duration-200 ease-out data-ending-style:h-0 data-ending-style:opacity-0 data-starting-style:h-0 data-starting-style:opacity-0";

const isString = (v: unknown): v is string => typeof v === "string";
const isNumber = (v: unknown): v is number => typeof v === "number";
const isBoolean = (v: unknown): v is boolean => typeof v === "boolean";
const isLiteral = (v: unknown): v is string | number => isString(v) || isNumber(v);
const isStringList = (v: unknown): v is string[] => Array.isArray(v) && v.every(isString);
const isRecordList = (v: unknown): v is Patch[] => Array.isArray(v) && v.every(isRecord);

function typed<V>(
  props: AnyFieldProps,
  is: (v: unknown) => v is V,
  empty: V,
): FieldProps<V> | null {
  if (props.value == null) return { ...props, value: empty };
  return is(props.value) ? { ...props, value: props.value } : null;
}

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

function ColorKindField(props: AnyFieldProps) {
  const field = typed(props, isString, "");
  return field ? <ColorField {...field} /> : <JsonField {...props} />;
}

function StringKindField(props: AnyFieldProps) {
  const field = typed(props, isString, "");
  return field ? <StringField {...field} /> : <JsonField {...props} />;
}

function BooleanKindField(props: AnyFieldProps) {
  const field = typed(props, isBoolean, false);
  return field ? <BooleanField {...field} /> : <JsonField {...props} />;
}

function NumberKindField({ min, max, ...props }: AnyFieldProps & { min?: number; max?: number }) {
  const field = typed(props, isNumber, 0);
  return field ? <NumberControl {...field} min={min} max={max} /> : <JsonField {...props} />;
}

function EnumKindField({ options, ...props }: AnyFieldProps & { options: string[] }) {
  const field = typed(props, isString, options[0] ?? "");
  if (!field) return <JsonField {...props} />;
  const { meta } = field;
  return (
    <EnumField
      {...field}
      options={options}
      format={
        meta.widget === "transition" ? transitionTitle : (v) => meta.labels?.[v] ?? humanize(v)
      }
    />
  );
}

function LiteralsField({ options, ...props }: AnyFieldProps & { options: Array<string | number> }) {
  const field = typed(props, isLiteral, "");
  if (!field) return <JsonField {...props} />;
  const { meta, onChange } = field;
  return (
    <EnumField
      {...field}
      options={options.map(String)}
      value={String(field.value)}
      onChange={(v) => onChange(options.find((o) => String(o) === v) ?? v)}
      format={(v) => (meta.unit ? `${v} ${meta.unit}` : v)}
    />
  );
}

function EnumListField({ options, ...props }: AnyFieldProps & { options: string[] }) {
  const field = typed(props, isStringList, []);
  if (!field) return <JsonField {...props} />;
  const { meta } = field;
  return (
    <ChoiceGroup
      multiple
      label={field.label}
      meta={meta}
      choices={options.map((o) => ({ value: o, label: meta.labels?.[o] ?? o }))}
      value={field.value}
      onChange={field.onChange}
    />
  );
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

function ObjectListKindField({ item, ...props }: AnyFieldProps & { item: z.ZodObject }) {
  const field = typed(props, isRecordList, []);
  return field ? <ObjectListField {...field} item={item} /> : <JsonField {...props} />;
}

function ObjectListField({
  label,
  meta,
  item,
  value,
  onChange,
  ctx,
}: FieldProps<Patch[]> & { item: z.ZodObject }) {
  const keys = useItemKeys(value.length);
  return (
    <FieldSet>
      <FieldLegend variant="label">{label}</FieldLegend>
      <Help meta={meta} />
      <div className="flex flex-col gap-3">
        {value.map((entry, i) => (
          <div
            key={keys.keys[i] ?? i}
            className="bg-muted/40 flex flex-col gap-3 rounded-lg border p-3"
          >
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
              onClick={() => {
                keys.remove(i);
                onChange(value.toSpliced(i, 1));
              }}
            >
              <Trash2 data-icon="inline-start" />
              Remove
            </Button>
          </div>
        ))}
        <Button
          variant="outline"
          size="sm"
          onClick={() => {
            keys.add();
            onChange([...value, schemaDefaults(item)]);
          }}
        >
          <Plus data-icon="inline-start" />
          Add
        </Button>
      </div>
    </FieldSet>
  );
}
