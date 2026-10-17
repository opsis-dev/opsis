import { ChevronRight, Plus, Trash2 } from "lucide-react";
import type { ComponentType } from "react";
import type { z } from "zod";
import { fieldMeta, type FieldMeta } from "@/shared/elements/introspect";
import { humanize } from "@/shared/format";
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
import { BoxesField } from "./form/boxes-field";
import { ColorField } from "./form/color-field";
import { EnumField } from "./form/enum-field";
import { EnvelopeField } from "./form/envelope-field";
import { ChoiceGroup, Help } from "./form/field-shell";
import { JsonField } from "./form/json-field";
import { NumberControl } from "./form/number-field";
import { isColor, labelFor, schemaKind } from "./form/schema-kind";
import { StringField } from "./form/string-field";
import type { FieldProps, FormContext, Patch } from "./form/types";
import { isRecord } from "@/shared/guards";

type Entry = { key: string; field: z.ZodType; meta: FieldMeta };

const VALUE_WIDGETS: Partial<Record<string, ComponentType<FieldProps<unknown>>>> = {
  boxes: BoxesField,
  envelope: EnvelopeField,
};

const isString = (v: unknown): v is string => typeof v === "string";

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
      {advanced.length ? (
        <Collapsible>
          <CollapsibleTrigger
            render={
              <Button
                variant="ghost"
                size="sm"
                className="group/adv text-muted-foreground -ml-2 w-fit"
              >
                <ChevronRight
                  data-icon="inline-start"
                  className="transition-transform group-data-[panel-open]/adv:rotate-90"
                />
                Advanced
              </Button>
            }
          />
          <CollapsibleContent className="h-(--collapsible-panel-height) overflow-hidden transition-[height,opacity] duration-200 ease-out data-ending-style:h-0 data-ending-style:opacity-0 data-starting-style:h-0 data-starting-style:opacity-0">
            <FieldGroup className="gap-4 pt-3">{render(advanced)}</FieldGroup>
          </CollapsibleContent>
        </Collapsible>
      ) : null}
    </FieldGroup>
  );
}

function FieldControl({
  name,
  schema,
  value,
  all,
  onChange,
  onPatch,
  ctx,
}: {
  name: string;
  schema: z.ZodType;
  value: unknown;
  all: Patch;
  onChange: (v: unknown) => void;
  onPatch: (patch: Patch) => void;
  ctx: FormContext;
}) {
  const meta = fieldMeta(schema);
  const props = { id: `f-${name}`, name, label: labelFor(name, schema), meta, all, ctx };
  const handlers = { onChange, onPatch };
  if (isColor(schema)) return <ColorField {...props} {...handlers} value={String(value ?? "")} />;
  const Widget = meta.widget ? VALUE_WIDGETS[meta.widget] : undefined;
  if (Widget) return <Widget {...props} {...handlers} value={value} />;
  const kind = schemaKind(schema);
  switch (kind.kind) {
    case "boolean":
      return <BooleanField {...props} {...handlers} value={Boolean(value)} />;
    case "number":
      return (
        <NumberControl
          {...props}
          {...handlers}
          value={typeof value === "number" ? value : 0}
          min={kind.min}
          max={kind.max}
        />
      );
    case "string":
      return <StringField {...props} {...handlers} value={String(value ?? "")} />;
    case "enum":
      return (
        <EnumField
          {...props}
          options={kind.options}
          value={String(value ?? kind.options[0])}
          onChange={onChange}
          format={
            meta.widget === "transition" ? transitionTitle : (v) => meta.labels?.[v] ?? humanize(v)
          }
        />
      );
    case "literals":
      return (
        <EnumField
          {...props}
          options={kind.options.map(String)}
          value={String(value)}
          onChange={(v) => onChange(typeof kind.options[0] === "number" ? Number(v) : v)}
          format={(v) => (meta.unit ? `${v} ${meta.unit}` : v)}
        />
      );
    case "enumList":
      return (
        <ChoiceGroup
          multiple
          label={props.label}
          meta={meta}
          choices={kind.options.map((o) => ({ value: o, label: meta.labels?.[o] ?? o }))}
          value={Array.isArray(value) ? value.filter(isString) : []}
          onChange={onChange}
        />
      );
    case "objectList":
      return (
        <ObjectListField
          label={props.label}
          meta={meta}
          schema={kind.item}
          value={Array.isArray(value) ? value.filter(isRecord) : []}
          onChange={onChange}
          ctx={ctx}
        />
      );
    default:
      return <JsonField {...props} {...handlers} value={value} />;
  }
}

function BooleanField({ id, label, value, onChange }: FieldProps<boolean>) {
  return (
    <Field orientation="horizontal" className="justify-between">
      <FieldLabel htmlFor={id} className="font-normal">
        {label}
      </FieldLabel>
      <Switch id={id} checked={value} onCheckedChange={(v) => onChange(v)} />
    </Field>
  );
}

function ObjectListField({
  label,
  meta,
  schema,
  value,
  onChange,
  ctx,
}: {
  label: string;
  meta: FieldMeta;
  schema: z.ZodObject;
  value: Patch[];
  onChange: (v: Patch[]) => void;
  ctx: FormContext;
}) {
  return (
    <FieldSet>
      <FieldLegend variant="label">{label}</FieldLegend>
      <Help meta={meta} />
      <div className="flex flex-col gap-3">
        {value.map((item, i) => (
          <div key={i} className="bg-muted/40 flex flex-col gap-3 rounded-lg border p-3">
            <ZodForm
              schema={schema}
              value={item}
              ctx={ctx}
              onChange={(patch) =>
                onChange(value.map((v, j) => (j === i ? { ...v, ...patch } : v)))
              }
            />
            <Button
              variant="ghost"
              size="sm"
              onClick={() => onChange(value.filter((_, j) => j !== i))}
            >
              <Trash2 data-icon="inline-start" />
              Remove
            </Button>
          </div>
        ))}
        <Button variant="outline" size="sm" onClick={() => onChange([...value, schema.parse({})])}>
          <Plus data-icon="inline-start" />
          Add
        </Button>
      </div>
    </FieldSet>
  );
}
