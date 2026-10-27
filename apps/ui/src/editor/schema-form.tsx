import { humanize } from "@opsis/video/common/format";
import { isNumber } from "@opsis/video/common/guards";
import type { FieldWidget } from "@opsis/video/generated/spec";
import { cn } from "cn";
import { ChevronRight, Copy, Plus, Trash2 } from "lucide-react";
import { useState, type ComponentType, type ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { Field, FieldGroup, FieldLabel, FieldLegend, FieldSet } from "@/components/ui/field";
import { Switch } from "@/components/ui/switch";
import {
  ANY_VALUE,
  asSchema,
  fieldMeta,
  schemaDefaults,
  shapeEntries,
  withDefaults,
  type FieldMeta,
  type JsonSchema,
} from "@/editor/form/schema";
import { fileName } from "@/lib/file-name";
import { whenOneOf } from "@/lib/one-of";
import { useItemKeys } from "@/lib/use-item-keys";
import { loudnessPresets, qualityPresets, transitionTitle } from "@/store/catalog";
import { BoxesField } from "./form/boxes-field";
import { CameraField } from "./form/camera-field";
import { ColorField } from "./form/color-field";
import { EnumField } from "./form/enum-field";
import { EnvelopeField } from "./form/envelope-field";
import { ChoiceGroup, Help } from "./form/field-shell";
import { GradientField } from "./form/gradient-field";
import { JsonField } from "./form/json-field";
import { KeysField } from "./form/keys-field";
import {
  ColorListField,
  DragHandle,
  NumberListField,
  SortableRows,
  StringListField,
} from "./form/list-fields";
import type { LengthLimits } from "./form/list-values";
import { NumberField, PresetNumber } from "./form/number-field";
import { OffsetsField } from "./form/offsets-field";
import { Presets } from "./form/presets";
import { typedField, type TypedField } from "./form/schema-kind";
import { labelFor } from "./form/schema-meta";
import { MusicField, SoundField } from "./form/sound-field";
import { SpotField } from "./form/spot-field";
import {
  actionChoices,
  anyPortChoices,
  cutChoices,
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
import { ThemeField } from "./form/theme-field";
import { TimezoneField } from "./form/timezone-field";
import type { FieldProps, FormContext, ParamFieldProps, ParamValues } from "./form/types";

type Entry = { key: string; field: JsonSchema; meta: FieldMeta };

type FieldOf<K extends TypedField["kind"]> = Extract<TypedField, { kind: K }>;

type Render<K extends TypedField["kind"]> = (
  field: FieldOf<K>,
  props: ParamFieldProps<unknown>,
) => ReactNode;

type Widget = { string: Render<"string"> } | { number: Render<"number"> };

const stringWidget = (Control: ComponentType<ParamFieldProps<string>>): Widget => ({
  string: (field, props) => <Control {...props} value={field.value} />,
});

const selectWidget = (choices: (ctx: FormContext, value: string) => SelectChoices): Widget => ({
  string: (field, props) => (
    <EnumField {...props} {...choices(props.ctx, field.value)} value={field.value} forceSelect />
  ),
});

const presetsWidget = (presets: () => Array<[string, number]>): Widget => ({
  number: (field, props) => (
    <PresetNumber {...props} value={field.value} presets={Object.fromEntries(presets())} />
  ),
});

const WIDGETS: Partial<Record<FieldWidget, Widget>> = {
  action: selectWidget(actionChoices),
  cut: selectWidget(cutChoices),
  port: selectWidget(anyPortChoices),
  variable: selectWidget(variableChoices),
  backdrop: selectWidget(backdropChoices),
  code: stringWidget(CodeField),
  asset: stringWidget(AssetField),
  font: stringWidget(FontField),
  sound: stringWidget(SoundField),
  music: stringWidget(MusicField),
  target: stringWidget(TargetField),
  timezone: stringWidget(TimezoneField),
  keys: stringWidget(KeysField),
  datetime: stringWidget(DatetimeField),
  offsets: stringWidget(OffsetsField),
  spot: { number: (field, props) => <SpotField {...props} value={field.value} /> },
  quality: presetsWidget(qualityPresets),
  loudness: presetsWidget(loudnessPresets),
};

export const widgetTakes = (widget: FieldWidget, kind: TypedField["kind"]): boolean => {
  const entry = WIDGETS[widget];
  return !entry || Object.hasOwn(entry, kind);
};

const COLLAPSING =
  "h-(--collapsible-panel-height) overflow-hidden transition-[height,opacity] duration-200 ease-out data-ending-style:h-0 data-ending-style:opacity-0 data-starting-style:h-0 data-starting-style:opacity-0";

function fileKindsMatch(meta: FieldMeta, ctx: FormContext): boolean {
  return Object.entries(meta.showIfFile ?? {}).every(([k, kinds]) => {
    const kind = ctx.fileKinds[k];
    return kind !== undefined && kinds.includes(kind);
  });
}

function visible(meta: FieldMeta, values: ParamValues, ctx: FormContext): boolean {
  if (meta.hidden || !fileKindsMatch(meta, ctx)) return false;
  if (!meta.showIf) return true;
  return Object.entries(meta.showIf).every(([k, allowed]) => {
    const v = values[k] == null ? "" : String(values[k]);
    return allowed.includes(v) || (allowed.includes(ANY_VALUE) && v !== "");
  });
}

export function SchemaForm({
  schema: raw,
  value,
  onChange,
  ctx,
  idPrefix = "f",
}: {
  schema: unknown;
  value: ParamValues;
  onChange: (patch: ParamValues) => void;
  ctx: FormContext;
  idPrefix?: string;
}) {
  const schema = asSchema(raw);
  const values = withDefaults(schema, value);
  const entries = shapeEntries(schema)
    .map(([key, field]): Entry => ({ key, field, meta: fieldMeta(field) }))
    .filter(({ meta }) => visible(meta, values, ctx));
  if (entries.length === 0) return <p className="text-muted-foreground text-sm">No settings.</p>;
  const basic = entries.filter(({ meta }) => !meta.advanced);
  const advanced = entries.filter(({ meta }) => meta.advanced);
  const render = (list: Entry[]) =>
    list.map(({ key, field, meta }) => (
      <FieldControl
        key={key}
        schema={field}
        props={{
          id: `${idPrefix}-${key}`,
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

function FieldControl({ schema, props }: { schema: JsonSchema; props: ParamFieldProps<unknown> }) {
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

function FieldBody({ schema, props }: { schema: JsonSchema; props: ParamFieldProps<unknown> }) {
  const field = typedField(schema, props.value);
  if (!field) return <JsonField {...props} />;
  const widget = props.meta.widget ? WIDGETS[props.meta.widget] : undefined;
  if (widget && field.kind === "string" && "string" in widget) return widget.string(field, props);
  if (widget && field.kind === "number" && "number" in widget) return widget.number(field, props);
  return plainBody(field, props);
}

function plainBody(field: TypedField, props: ParamFieldProps<unknown>): ReactNode {
  switch (field.kind) {
    case "color":
      return <ColorField {...props} value={field.value} />;
    case "boxes":
      return <BoxesField {...props} value={field.value} />;
    case "camera":
      return <CameraField {...props} value={field.value} />;
    case "envelope":
      return <EnvelopeField {...props} value={field.value} tracks={field.tracks} />;
    case "boolean":
      return <BooleanField {...props} value={field.value} />;
    case "number":
      return <NumberField {...props} value={field.value} min={field.min} max={field.max} />;
    case "optionalNumber":
      return (
        <NumberField
          {...props}
          value={field.value}
          min={field.min}
          max={field.max}
          onClear={() => props.onChange(undefined)}
        />
      );
    case "string":
      return <TextField {...props} value={field.value} />;
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
      return <ObjectListField {...props} value={field.value} item={field.item} limits={field} />;
    case "stringList":
      return <StringListField {...props} value={field.value} limits={field} />;
    case "numberList":
      return <NumberListField {...props} value={field.value} limits={field} />;
    case "colorList":
      return <ColorListField {...props} value={field.value} limits={field} />;
    case "gradient":
      return <GradientField {...props} value={field.value} />;
    case "theme":
      return <ThemeField {...props} value={field.value} />;
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

function LiteralChoice({
  options,
  value,
  onChange,
  ...props
}: FieldProps<string | number> & { options: Array<string | number> }) {
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
}: FieldProps<string[]> & { options: string[] }) {
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

const COLLAPSE_FROM = 4;

const SUMMARY_SKIPS: ReadonlySet<string | undefined> = new Set([
  "asset",
  "color",
  "code",
  "sound",
  "music",
]);

const PORT_WIDGETS: ReadonlySet<string | undefined> = new Set(["cut", "port"]);

function rowSummary(
  item: JsonSchema,
  entry: ParamValues,
  index: number,
  ports: FormContext["ports"],
): string {
  const fields = shapeEntries(item);
  const portLabel = (port: string) => ports.find((p) => p.port === port)?.label ?? "Not connected";
  const textOf = (key: string) => {
    const v = entry[key];
    if (typeof v !== "string") return undefined;
    const field = fields.find(([k]) => k === key)?.[1];
    if (field && PORT_WIDGETS.has(fieldMeta(field).widget)) return portLabel(v);
    return v.split("\n")[0]?.trim();
  };
  const text = fields
    .filter(
      ([, f]) =>
        typedField(f, undefined)?.kind === "string" && !SUMMARY_SKIPS.has(fieldMeta(f).widget),
    )
    .map(([k]) => textOf(k))
    .find(Boolean);
  const file = fields
    .filter(([, f]) => fieldMeta(f).widget === "asset")
    .map(([k]) => textOf(k))
    .find(Boolean);
  const number = fields.map(([k]) => entry[k]).find(isNumber);
  const shown = text ?? (file ? fileName(file) : number === undefined ? undefined : String(number));
  return shown ? shown.slice(0, 60) : `Item ${index + 1}`;
}

function ObjectListField({
  id,
  label,
  meta,
  item,
  value,
  onChange,
  ctx,
  limits,
}: FieldProps<ParamValues[]> & { item: JsonSchema; limits: LengthLimits }) {
  const list = useItemKeys(value);
  const [closed, setClosed] = useState<ReadonlySet<string>>(() =>
    value.length < COLLAPSE_FROM ? new Set() : new Set(list.rows.map((r) => r.key)),
  );
  const toggle = (key: string) =>
    setClosed((prev) => {
      const next = new Set(prev);
      if (!next.delete(key)) next.add(key);
      return next;
    });
  return (
    <FieldSet>
      <FieldLegend variant="label">{label}</FieldLegend>
      <Help meta={meta} />
      <SortableRows
        rows={list.rows}
        className="flex flex-col gap-2"
        onReorder={(keys) => onChange(list.reorder(keys))}
      >
        {({ item: entry, key }, i) => {
          const expanded = !closed.has(key);
          const name = rowSummary(item, entry, i, ctx.ports);
          return (
            <div className="bg-muted/40 flex min-w-0 flex-1 flex-col rounded-lg border">
              <div className="flex items-center gap-1 px-1 py-1">
                <DragHandle label={name} />
                <button
                  type="button"
                  aria-expanded={expanded}
                  className="flex min-w-0 flex-1 cursor-pointer items-center gap-1 text-left text-sm"
                  onClick={() => toggle(key)}
                >
                  <ChevronRight
                    className={cn(
                      "size-3.5 shrink-0 transition-transform",
                      expanded && "rotate-90",
                    )}
                  />
                  <span className="truncate">{name}</span>
                </button>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  className="text-muted-foreground"
                  aria-label={`Duplicate ${name}`}
                  title="Duplicate"
                  disabled={value.length >= limits.max}
                  onClick={() => onChange(list.insert(i + 1, [structuredClone(entry)]))}
                >
                  <Copy />
                </Button>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  className="text-muted-foreground"
                  aria-label={`Remove ${name}`}
                  title="Remove"
                  disabled={value.length <= limits.min}
                  onClick={() => onChange(list.remove(i))}
                >
                  <Trash2 />
                </Button>
              </div>
              {expanded ? (
                <div className="border-t p-3">
                  <SchemaForm
                    idPrefix={`${id}-${i}`}
                    schema={item}
                    value={entry}
                    ctx={ctx}
                    onChange={(patch) => onChange(list.update(i, { ...entry, ...patch }))}
                  />
                </div>
              ) : null}
            </div>
          );
        }}
      </SortableRows>
      <Button
        variant="outline"
        size="sm"
        className="w-fit"
        disabled={value.length >= limits.max}
        onClick={() => onChange(list.add(schemaDefaults(item)))}
      >
        <Plus data-icon="inline-start" />
        Add
      </Button>
    </FieldSet>
  );
}
