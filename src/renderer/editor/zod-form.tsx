import {
  Check,
  ChevronsUpDown,
  ChevronRight,
  Crosshair,
  FolderOpen,
  Play,
  Plus,
  Trash2,
  X,
} from "lucide-react";
import { useMemo, useRef, useState } from "react";
import type { z } from "zod";
import { fieldMeta, unwrap, type FieldMeta } from "@/shared/elements/introspect";
import { QUALITY_PRESETS } from "@/shared/nodes/params";
import { transitionTitle } from "@/shared/nodes/transitions";
import { BUILTIN_SOUNDS, builtinSound } from "@/shared/sounds";
import { BACKDROPS } from "@/video/camera/lib/backdrop";
import { Button } from "@/renderer/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/renderer/components/ui/collapsible";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/renderer/components/ui/command";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldSet,
  FieldLegend,
} from "@/renderer/components/ui/field";
import { Input } from "@/renderer/components/ui/input";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/renderer/components/ui/input-group";
import { Kbd } from "@/renderer/components/ui/kbd";
import { Popover, PopoverContent, PopoverTrigger } from "@/renderer/components/ui/popover";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/renderer/components/ui/select";
import { Slider } from "@/renderer/components/ui/slider";
import { Switch } from "@/renderer/components/ui/switch";
import { Textarea } from "@/renderer/components/ui/textarea";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { cn } from "@/renderer/lib/utils";
import { FontPicker } from "../fonts/font-picker";
import { fileUrl } from "./file-url";
import { chordFromEvent } from "./keys";
import { VarText, type VarInfo } from "./var-text";

// oxlint-disable-next-line typescript/no-explicit-any
type AnySchema = z.ZodType<any>;
type Def = { type: string; [k: string]: unknown };

export type SpotBackground = { src: string; timeMs: number } | null;

export type FormContext = {
  projectDir: string | null;
  fps: number;
  aspect: number;
  vars: VarInfo[];
  secrets: string[] | null;
  layers?: number;
  spotBackground?: SpotBackground;
  pickTarget?: (key: string) => void;
  chooseAsset?: (key: string, accept?: string) => void;
};

const defOf = (s: AnySchema) => (s as unknown as { _zod: { def: Def } })._zod.def;
const REMOTION_COLOR = "__remotion-color";

function isColor(schema: AnySchema): boolean {
  const inner = unwrap(schema);
  return (
    (inner as { description?: string }).description === REMOTION_COLOR ||
    fieldMeta(schema).widget === "color"
  );
}

export const humanize = (v: string) =>
  v
    .replace(/[_-]+/g, " ")
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/^./, (c) => c.toUpperCase());

function labelFor(key: string, schema: AnySchema): string {
  const d = fieldMeta(schema).description;
  if (d && d !== REMOTION_COLOR) return d;
  return humanize(key);
}

function visible(meta: FieldMeta, value: Record<string, unknown>): boolean {
  if (meta.hidden) return false;
  if (!meta.showIf) return true;
  return Object.entries(meta.showIf).every(([k, allowed]) => allowed.includes(String(value[k])));
}

export function ZodForm({
  schema,
  value,
  onChange,
  ctx,
  omit = [],
}: {
  schema: z.ZodObject;
  value: Record<string, unknown>;
  onChange: (patch: Record<string, unknown>) => void;
  ctx: FormContext;
  omit?: string[];
}) {
  const entries = Object.entries(schema.shape)
    .filter(([k]) => !omit.includes(k))
    .map(([k, s]) => [k, s as AnySchema, fieldMeta(s as AnySchema)] as const)
    .filter(([, , meta]) => visible(meta, value));
  const basic = entries.filter(([, , meta]) => !meta.advanced);
  const advanced = entries.filter(([, , meta]) => meta.advanced);
  if (entries.length === 0) return <p className="text-muted-foreground text-sm">No settings.</p>;
  const render = (list: typeof entries) =>
    list.map(([key, field]) => (
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

function Help({ meta }: { meta: FieldMeta }) {
  return meta.help ? <FieldDescription>{meta.help}</FieldDescription> : null;
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
  schema: AnySchema;
  value: unknown;
  all: Record<string, unknown>;
  onChange: (v: unknown) => void;
  onPatch: (patch: Record<string, unknown>) => void;
  ctx: FormContext;
}) {
  const meta = fieldMeta(schema);
  const inner = unwrap(schema);
  const def = defOf(inner);
  const label = labelFor(name, schema);
  const id = `f-${name}`;

  if (isColor(schema))
    return <ColorField id={id} label={label} value={String(value ?? "")} onChange={onChange} />;
  if (meta.widget === "boxes")
    return <BoxesField label={label} value={value} onChange={onChange} ctx={ctx} meta={meta} />;

  switch (def.type) {
    case "boolean":
      return (
        <Field orientation="horizontal" className="justify-between">
          <FieldLabel htmlFor={id} className="font-normal">
            {label}
          </FieldLabel>
          <Switch id={id} checked={Boolean(value)} onCheckedChange={(v) => onChange(v)} />
        </Field>
      );

    case "number": {
      const n = inner as unknown as { minValue: number | null; maxValue: number | null };
      const num = typeof value === "number" ? value : 0;
      if (meta.widget === "quality")
        return (
          <PresetNumber
            label={label}
            value={num}
            options={Object.entries(QUALITY_PRESETS).map(([l, v]) => [l, v])}
            onChange={onChange}
            meta={meta}
          />
        );
      if (meta.widget === "loudness")
        return (
          <PresetNumber
            label={label}
            value={num}
            options={[
              ["Online video", -14],
              ["Podcast", -16],
              ["Broadcast", -23],
            ]}
            onChange={onChange}
            meta={meta}
          />
        );
      if (meta.widget === "spot")
        return (
          <SpotField label={label} name={name} all={all} onPatch={onPatch} ctx={ctx} meta={meta} />
        );
      return (
        <>
          {meta.presets ? <Presets meta={meta} all={all} onPatch={onPatch} /> : null}
          <NumberField
            id={id}
            label={label}
            value={num}
            min={n.minValue ?? undefined}
            max={n.maxValue ?? undefined}
            meta={meta}
            fps={ctx.fps}
            onChange={onChange}
          />
        </>
      );
    }

    case "enum": {
      const options = Object.values(def.entries as Record<string, string>);
      const format =
        meta.widget === "transition"
          ? transitionTitle
          : (v: string) => meta.labels?.[v] ?? humanize(v);
      return (
        <EnumField
          id={id}
          label={label}
          options={options}
          value={String(value ?? options[0])}
          onChange={onChange}
          format={format}
          meta={meta}
        />
      );
    }

    case "union": {
      const literals = (def.options as AnySchema[])
        .map((o) => defOf(o))
        .filter((d) => d.type === "literal");
      const values = literals.flatMap((d) => d.values as Array<string | number>);
      if (values.length) {
        return (
          <EnumField
            id={id}
            label={label}
            options={values.map(String)}
            value={String(value)}
            onChange={(v) => onChange(typeof values[0] === "number" ? Number(v) : v)}
            format={(v) => (meta.unit ? `${v} ${meta.unit}` : v)}
            meta={meta}
          />
        );
      }
      return <JsonField id={id} label={label} value={value} onChange={onChange} />;
    }

    case "array": {
      const element = unwrap(def.element as AnySchema);
      const elDef = defOf(element);
      const list = Array.isArray(value) ? value : [];
      if (elDef.type === "enum") {
        const options = Object.values(elDef.entries as Record<string, string>);
        return (
          <Field>
            <FieldLabel>{label}</FieldLabel>
            <ToggleGroup
              multiple
              variant="outline"
              size="sm"
              value={list as string[]}
              onValueChange={(v) => onChange(v)}
              className="flex-wrap"
            >
              {options.map((o) => (
                <ToggleGroupItem key={o} value={o}>
                  {meta.labels?.[o] ?? o}
                </ToggleGroupItem>
              ))}
            </ToggleGroup>
            <Help meta={meta} />
          </Field>
        );
      }
      if (elDef.type === "object") {
        return (
          <ObjectListField
            label={label}
            schema={element as unknown as z.ZodObject}
            value={list as Record<string, unknown>[]}
            onChange={onChange}
            ctx={ctx}
          />
        );
      }
      return <JsonField id={id} label={label} value={value} onChange={onChange} />;
    }

    case "string":
      return (
        <StringField
          id={id}
          name={name}
          label={label}
          value={String(value ?? "")}
          meta={meta}
          onChange={onChange}
          onPatch={onPatch}
          ctx={ctx}
        />
      );

    default:
      return <JsonField id={id} label={label} value={value} onChange={onChange} />;
  }
}

function StringField({
  id,
  name,
  label,
  value,
  meta,
  onChange,
  onPatch,
  ctx,
}: {
  id: string;
  name: string;
  label: string;
  value: string;
  meta: FieldMeta;
  onChange: (v: string) => void;
  onPatch: (patch: Record<string, unknown>) => void;
  ctx: FormContext;
}) {
  switch (meta.widget) {
    case "code":
      return (
        <Field>
          <FieldLabel htmlFor={id}>{label}</FieldLabel>
          <Textarea
            id={id}
            value={value}
            onChange={(e) => onChange(e.target.value)}
            rows={6}
            className="font-mono text-xs"
            spellCheck={false}
          />
          <Help meta={meta} />
        </Field>
      );
    case "asset":
      return (
        <Field>
          <FieldLabel htmlFor={id}>{label}</FieldLabel>
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
                onClick={() => ctx.chooseAsset?.(name, meta.accept)}
                disabled={!ctx.projectDir}
              >
                <FolderOpen data-icon="inline-start" />
                Choose
              </InputGroupButton>
            </InputGroupAddon>
          </InputGroup>
          <Help meta={meta} />
        </Field>
      );
    case "font":
      return (
        <Field>
          <FieldLabel htmlFor={id}>{label}</FieldLabel>
          <FontPicker id={id} value={value} onChange={onChange} emptyLabel="Project font" />
          <Help meta={meta} />
        </Field>
      );
    case "sound":
      return (
        <SoundField
          id={id}
          name={name}
          label={label}
          value={value}
          onChange={onChange}
          meta={meta}
          ctx={ctx}
        />
      );
    case "target":
      return (
        <Field>
          <FieldLabel htmlFor={id}>{label}</FieldLabel>
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
          <Help meta={meta} />
        </Field>
      );
    case "secret":
      return (
        <SecretField
          id={id}
          label={label}
          value={value}
          onChange={onChange}
          meta={meta}
          ctx={ctx}
        />
      );
    case "timezone":
      return <TimezoneField label={label} value={value} onChange={onChange} />;
    case "keys":
      return <KeysField id={id} label={label} value={value} onChange={onChange} meta={meta} />;
    case "datetime":
      return (
        <Field>
          <FieldLabel htmlFor={id}>{label}</FieldLabel>
          <Input
            id={id}
            type="datetime-local"
            step={60}
            value={value.slice(0, 16)}
            onChange={(e) => e.target.value && onChange(`${e.target.value}:00`)}
          />
          <Help meta={meta} />
        </Field>
      );
    case "variable":
      return (
        <EnumField
          id={id}
          label={label}
          options={[...new Set([...ctx.vars.map((v) => v.name), ...(value ? [value] : [])])]}
          value={value}
          onChange={onChange}
          meta={meta}
          forceSelect
        />
      );
    case "backdrop":
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
    case "offsets":
      return (
        <OffsetsField label={label} value={value} onChange={onChange} layers={ctx.layers ?? 2} />
      );
    default:
      return (
        <Field>
          <FieldLabel htmlFor={id}>{label}</FieldLabel>
          <VarText
            id={id}
            value={value}
            onChange={onChange}
            vars={ctx.vars}
            multiline={meta.widget === "textarea"}
            pauses={meta.pauses}
          />
          <Help meta={meta} />
        </Field>
      );
  }
}

function trimNumber(n: number): string {
  return String(Math.round(n * 1000) / 1000);
}

function NumberField({
  id,
  label,
  value,
  min,
  max,
  meta,
  fps,
  onChange,
}: {
  id: string;
  label: string;
  value: number;
  min?: number;
  max?: number;
  meta: FieldMeta;
  fps: number;
  onChange: (v: number) => void;
}) {
  const factor = meta.unit === "ms" ? 1000 : meta.unit === "frames" ? fps : 1;
  const unit = meta.unit === "ms" || meta.unit === "frames" ? "s" : meta.unit;
  const shownOf = (v: number) => (meta.zeroLabel && v === 0 ? "" : trimNumber(v / factor));
  const [draft, setDraft] = useState(shownOf(value));
  const [shown, setShown] = useState(value);
  if (shown !== value) {
    setShown(value);
    setDraft(shownOf(value));
  }
  const toStored = (n: number) => {
    const raw = meta.unit === "frames" || meta.unit === "ms" ? Math.round(n * factor) : n * factor;
    return min != null && raw < min ? min : max != null && raw > max ? max : raw;
  };
  const slider =
    min != null && max != null && max - min <= 5000 && meta.unit !== "ms" && meta.unit !== "frames";
  const commit = (text: string) => {
    if (text.trim() === "" && meta.zeroLabel) {
      onChange(0);
      return;
    }
    const n = Number(text.replace(",", "."));
    if (text.trim() !== "" && Number.isFinite(n)) onChange(toStored(n));
  };
  return (
    <Field>
      <div className="flex items-center justify-between gap-2">
        <FieldLabel htmlFor={id}>{label}</FieldLabel>
        <div className="flex items-center gap-1">
          <Input
            id={id}
            className="h-7 w-24 text-right tabular-nums"
            inputMode="decimal"
            placeholder={meta.zeroLabel}
            value={draft}
            onChange={(e) => {
              setDraft(e.target.value);
              commit(e.target.value);
            }}
            onBlur={() => setDraft(shownOf(value))}
          />
          {unit ? <span className="text-muted-foreground w-12 text-xs">{unit}</span> : null}
        </div>
      </div>
      {slider ? (
        <Slider
          min={min}
          max={max}
          step={meta.step ?? (max! - min! <= 1 ? 0.01 : 1)}
          value={[value]}
          onValueChange={(v) => onChange(Array.isArray(v) ? v[0] : v)}
        />
      ) : null}
      <Help meta={meta} />
    </Field>
  );
}

function PresetNumber({
  label,
  value,
  options,
  onChange,
  meta,
}: {
  label: string;
  value: number;
  options: Array<[string, number]>;
  onChange: (v: number) => void;
  meta: FieldMeta;
}) {
  const nearest = options.reduce((best, cur) =>
    Math.abs(cur[1] - value) < Math.abs(best[1] - value) ? cur : best,
  )[0];
  return (
    <Field>
      <FieldLabel>{label}</FieldLabel>
      <ToggleGroup
        variant="outline"
        size="sm"
        value={[nearest]}
        onValueChange={(v) => {
          const hit = options.find(([l]) => l === v[0]);
          if (hit) onChange(hit[1]);
        }}
        className="flex-wrap"
      >
        {options.map(([l]) => (
          <ToggleGroupItem key={l} value={l}>
            {l}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
      <Help meta={meta} />
    </Field>
  );
}

function Presets({
  meta,
  all,
  onPatch,
}: {
  meta: FieldMeta;
  all: Record<string, unknown>;
  onPatch: (patch: Record<string, unknown>) => void;
}) {
  const presets = meta.presets ?? [];
  const current =
    presets.find((p) => Object.entries(p.values).every(([k, v]) => all[k] === v))?.label ??
    "Custom";
  return (
    <Field>
      <FieldLabel>Size</FieldLabel>
      <ToggleGroup
        variant="outline"
        size="sm"
        value={[current]}
        onValueChange={(v) => {
          const hit = presets.find((p) => p.label === v[0]);
          if (hit) onPatch(hit.values);
        }}
        className="flex-wrap"
      >
        {presets.map((p) => (
          <ToggleGroupItem key={p.label} value={p.label}>
            {p.label}
          </ToggleGroupItem>
        ))}
        <ToggleGroupItem value="Custom" disabled>
          Custom
        </ToggleGroupItem>
      </ToggleGroup>
    </Field>
  );
}

function EnumField({
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
  const short =
    !forceSelect && options.length <= 4 && options.reduce((n, o) => n + format(o).length, 0) <= 34;
  if (short) {
    return (
      <Field>
        <FieldLabel>{label}</FieldLabel>
        <ToggleGroup
          variant="outline"
          size="sm"
          value={[value]}
          onValueChange={(v) => v[0] && onChange(v[0])}
          className="flex-wrap"
        >
          {options.map((o) => (
            <ToggleGroupItem key={o} value={o}>
              {format(o)}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
        <Help meta={meta} />
      </Field>
    );
  }
  return (
    <Field>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <Select
        items={options.map((o) => ({ value: o, label: format(o) }))}
        value={value}
        onValueChange={(v) => v != null && onChange(String(v))}
      >
        <SelectTrigger id={id} className="w-full">
          <SelectValue placeholder="Choose…" />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            {options.map((o) => (
              <SelectItem key={o} value={o}>
                {format(o)}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>
      <Help meta={meta} />
    </Field>
  );
}

const OWN_FILE = "own-file";

function soundUrl(ref: string): string {
  if (ref.startsWith("app:")) return ref.slice("app:".length);
  return fileUrl(ref);
}

function SoundField({
  id,
  name,
  label,
  value,
  onChange,
  meta,
  ctx,
}: {
  id: string;
  name: string;
  label: string;
  value: string;
  onChange: (v: string) => void;
  meta: FieldMeta;
  ctx: FormContext;
}) {
  const builtin = builtinSound(value);
  const ownLabel = value && !builtin ? (value.split("/").pop() ?? value) : "Your own file…";
  const items = [
    ...BUILTIN_SOUNDS.map((s) => ({ value: s.ref as string, label: s.label })),
    { value: OWN_FILE, label: ownLabel },
  ];
  return (
    <Field>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <div className="flex gap-2">
        <Select
          items={items}
          value={builtin ? builtin.ref : value ? OWN_FILE : null}
          onValueChange={(v) => {
            if (v === OWN_FILE) ctx.chooseAsset?.(name, meta.accept);
            else if (v != null) onChange(String(v));
          }}
        >
          <SelectTrigger id={id} className="min-w-0 flex-1">
            <SelectValue placeholder="Choose a sound" />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              {BUILTIN_SOUNDS.map((s) => (
                <SelectItem key={s.id} value={s.ref}>
                  {s.label}
                </SelectItem>
              ))}
            </SelectGroup>
            <SelectGroup>
              <SelectItem value={OWN_FILE} disabled={!ctx.projectDir}>
                {ownLabel}
              </SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
        <Button
          variant="outline"
          size="icon"
          aria-label="Play sound"
          disabled={!value}
          onClick={() => void new Audio(soundUrl(value)).play().catch(() => undefined)}
        >
          <Play />
        </Button>
      </div>
      <Help meta={meta} />
    </Field>
  );
}

function SecretField({
  id,
  label,
  value,
  onChange,
  meta,
  ctx,
}: {
  id: string;
  label: string;
  value: string;
  onChange: (v: string) => void;
  meta: FieldMeta;
  ctx: FormContext;
}) {
  const NONE = "__none";
  const names = [...new Set([...(ctx.secrets ?? []), ...(value ? [value] : [])])];
  return (
    <Field>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <Select
        items={[
          { value: NONE, label: "Not signed in" },
          ...names.map((n) => ({ value: n, label: n })),
        ]}
        value={value || NONE}
        onValueChange={(v) => onChange(v == null || v === NONE ? "" : String(v))}
      >
        <SelectTrigger id={id} className="w-full">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            <SelectItem value={NONE}>Not signed in</SelectItem>
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
      <Help meta={meta} />
    </Field>
  );
}

const ZONES: string[] = (() => {
  try {
    return (Intl as unknown as { supportedValuesOf: (k: string) => string[] }).supportedValuesOf(
      "timeZone",
    );
  } catch {
    return ["UTC"];
  }
})();

function TimezoneField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
}) {
  const [open, setOpen] = useState(false);
  return (
    <Field>
      <FieldLabel>{label}</FieldLabel>
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger
          render={
            <Button variant="outline" className="w-full justify-between font-normal">
              <span className={cn("truncate", !value && "text-muted-foreground")}>
                {value ? value.replaceAll("_", " ") : "Same as this computer"}
              </span>
              <ChevronsUpDown data-icon="inline-end" />
            </Button>
          }
        />
        <PopoverContent className="w-72 p-0" align="start">
          <Command>
            <CommandInput placeholder="Search time zones…" />
            <CommandList>
              <CommandEmpty>No time zone found.</CommandEmpty>
              <CommandGroup>
                <CommandItem
                  value="same as this computer"
                  onSelect={() => {
                    onChange("");
                    setOpen(false);
                  }}
                >
                  Same as this computer
                  {!value ? <Check data-icon="inline-end" className="ml-auto" /> : null}
                </CommandItem>
                {ZONES.map((z) => (
                  <CommandItem
                    key={z}
                    value={z.replaceAll("_", " ")}
                    onSelect={() => {
                      onChange(z);
                      setOpen(false);
                    }}
                  >
                    {z.replaceAll("_", " ")}
                    {value === z ? <Check data-icon="inline-end" className="ml-auto" /> : null}
                  </CommandItem>
                ))}
              </CommandGroup>
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
    </Field>
  );
}

function KeysField({
  id,
  label,
  value,
  onChange,
  meta,
}: {
  id: string;
  label: string;
  value: string;
  onChange: (v: string) => void;
  meta: FieldMeta;
}) {
  const [listening, setListening] = useState(false);
  return (
    <Field>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <button
        id={id}
        type="button"
        className={cn(
          "border-input flex min-h-8 w-full items-center gap-1 rounded-lg border px-2.5 text-left text-sm outline-none dark:bg-input/30",
          "focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-3",
        )}
        onFocus={() => setListening(true)}
        onBlur={() => setListening(false)}
        onKeyDown={(e) => {
          if (e.key === "Tab") return;
          e.preventDefault();
          e.stopPropagation();
          const chord = chordFromEvent(e.nativeEvent);
          if (chord) onChange(chord);
        }}
      >
        {value ? (
          value.split("+").map((k, i) => <Kbd key={i}>{k}</Kbd>)
        ) : (
          <span className="text-muted-foreground">Press a key</span>
        )}
        <span className="text-muted-foreground ml-auto text-xs">
          {listening ? "Listening…" : ""}
        </span>
      </button>
      <Help meta={meta} />
    </Field>
  );
}

function SpotField({
  label,
  name,
  all,
  onPatch,
  ctx,
  meta,
}: {
  label: string;
  name: string;
  all: Record<string, unknown>;
  onPatch: (patch: Record<string, unknown>) => void;
  ctx: FormContext;
  meta: FieldMeta;
}) {
  const yKey = name === "x" ? "y" : name.replace(/X$/, "Y");
  const x = Number(all[name] ?? 0.5);
  const y = Number(all[yKey] ?? 0.5);
  const box = useRef<HTMLDivElement>(null);
  const time = Number(all.atMs ?? 0);
  const bg = ctx.spotBackground;
  const set = (e: React.PointerEvent) => {
    const r = box.current?.getBoundingClientRect();
    if (!r) return;
    const clamp = (v: number) => Math.min(1, Math.max(0, Math.round(v * 1000) / 1000));
    onPatch({
      [name]: clamp((e.clientX - r.left) / r.width),
      [yKey]: clamp((e.clientY - r.top) / r.height),
    });
  };
  const src = useMemo(() => (bg ? `${bg.src}#t=${(time || bg.timeMs) / 1000}` : null), [bg, time]);
  return (
    <Field>
      <FieldLabel>{label}</FieldLabel>
      <div
        ref={box}
        role="button"
        tabIndex={0}
        aria-label={`${label}: ${Math.round(x * 100)} %, ${Math.round(y * 100)} %`}
        className="bg-muted relative w-full cursor-crosshair overflow-hidden rounded-md border"
        style={{ aspectRatio: String(ctx.aspect) }}
        onPointerDown={(e) => {
          e.currentTarget.setPointerCapture(e.pointerId);
          set(e);
        }}
        onPointerMove={(e) => e.buttons === 1 && set(e)}
      >
        {src ? (
          <video
            src={src}
            muted
            preload="auto"
            className="pointer-events-none absolute inset-0 size-full object-cover"
          />
        ) : null}
        <span
          className="border-background bg-primary pointer-events-none absolute size-3 -translate-1/2 rounded-full border-2 shadow"
          style={{ left: `${x * 100}%`, top: `${y * 100}%` }}
        />
      </div>
      <Help meta={meta} />
    </Field>
  );
}

type Box = { x: number; y: number; width: number; height: number };
type BoxDrag = { mode: "new" | "move"; index: number; x: number; y: number; origin: Box };

const MAX_BOXES = 8;
const clamp01 = (v: number) => Math.min(1, Math.max(0, v));
const round3 = (v: number) => Math.round(v * 1000) / 1000;
const boxFrom = (x0: number, y0: number, x1: number, y1: number): Box => ({
  x: round3(Math.min(x0, x1)),
  y: round3(Math.min(y0, y1)),
  width: round3(Math.abs(x1 - x0)),
  height: round3(Math.abs(y1 - y0)),
});

function BoxesField({
  label,
  value,
  onChange,
  ctx,
  meta,
}: {
  label: string;
  value: unknown;
  onChange: (v: unknown) => void;
  ctx: FormContext;
  meta: FieldMeta;
}) {
  const boxes = Array.isArray(value) ? (value as Box[]) : [];
  const area = useRef<HTMLDivElement>(null);
  const drag = useRef<BoxDrag | null>(null);
  const [draft, setDraft] = useState<{ mode: BoxDrag["mode"]; index: number; box: Box } | null>(
    null,
  );
  const bg = ctx.spotBackground;
  const src = useMemo(() => (bg ? `${bg.src}#t=${bg.timeMs / 1000}` : null), [bg]);

  const point = (e: React.PointerEvent): [number, number] => {
    const r = area.current?.getBoundingClientRect();
    if (!r) return [0, 0];
    return [clamp01((e.clientX - r.left) / r.width), clamp01((e.clientY - r.top) / r.height)];
  };
  const start = (e: React.PointerEvent, mode: BoxDrag["mode"], index: number) => {
    const [x, y] = point(e);
    e.currentTarget.setPointerCapture(e.pointerId);
    const origin = mode === "move" ? boxes[index] : boxFrom(x, y, x, y);
    drag.current = { mode, index, x, y, origin };
    setDraft({ mode, index, box: origin });
  };
  const move = (e: React.PointerEvent) => {
    const d = drag.current;
    if (!d) return;
    const [x, y] = point(e);
    const box =
      d.mode === "new"
        ? boxFrom(d.x, d.y, x, y)
        : {
            ...d.origin,
            x: round3(Math.min(1 - d.origin.width, Math.max(0, d.origin.x + x - d.x))),
            y: round3(Math.min(1 - d.origin.height, Math.max(0, d.origin.y + y - d.y))),
          };
    setDraft({ mode: d.mode, index: d.index, box });
  };
  const end = () => {
    const d = drag.current;
    drag.current = null;
    setDraft(null);
    if (!d || !draft) return;
    const box = draft.box;
    if (d.mode === "move") onChange(boxes.map((b, i) => (i === d.index ? box : b)));
    else if (box.width > 0.01 && box.height > 0.01) onChange([...boxes, box]);
  };
  const shown = boxes.map((b, i) => (draft?.mode === "move" && draft.index === i ? draft.box : b));
  const full = boxes.length >= MAX_BOXES;

  return (
    <Field>
      <FieldLabel>{label}</FieldLabel>
      <div
        ref={area}
        className={cn(
          "bg-muted relative w-full touch-none overflow-hidden rounded-md border select-none",
          full ? "cursor-default" : "cursor-crosshair",
        )}
        style={{ aspectRatio: String(ctx.aspect) }}
        onPointerDown={(e) => !full && start(e, "new", boxes.length)}
        onPointerMove={move}
        onPointerUp={end}
        onPointerCancel={end}
      >
        {src ? (
          <video
            src={src}
            muted
            preload="auto"
            className="pointer-events-none absolute inset-0 size-full object-cover"
          />
        ) : null}
        {shown.map((b, i) => (
          <div
            key={i}
            className="border-primary bg-primary/25 absolute cursor-move border-2"
            style={{
              left: `${b.x * 100}%`,
              top: `${b.y * 100}%`,
              width: `${b.width * 100}%`,
              height: `${b.height * 100}%`,
            }}
            onPointerDown={(e) => {
              e.stopPropagation();
              start(e, "move", i);
            }}
          >
            <button
              type="button"
              aria-label={`Remove area ${i + 1}`}
              className="bg-background text-foreground absolute -top-2 -right-2 grid size-4 place-items-center rounded-full border"
              onPointerDown={(e) => e.stopPropagation()}
              onClick={() => onChange(boxes.filter((_, j) => j !== i))}
            >
              <X className="size-3" />
            </button>
          </div>
        ))}
        {draft?.mode === "new" ? (
          <div
            className="border-primary bg-primary/25 pointer-events-none absolute border-2 border-dashed"
            style={{
              left: `${draft.box.x * 100}%`,
              top: `${draft.box.y * 100}%`,
              width: `${draft.box.width * 100}%`,
              height: `${draft.box.height * 100}%`,
            }}
          />
        ) : null}
      </div>
      <FieldDescription>
        {full
          ? `Up to ${MAX_BOXES} areas. Remove one to add another.`
          : `${boxes.length} of ${MAX_BOXES} areas.`}{" "}
        {meta.help}
      </FieldDescription>
    </Field>
  );
}

function OffsetsField({
  label,
  value,
  onChange,
  layers,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  layers: number;
}) {
  const parts = value.split(",").map((s) => Number(s.trim()) || 0);
  const count = Math.max(layers, parts.length, 1);
  const list = Array.from({ length: count }, (_, i) => parts[i] ?? 0);
  return (
    <FieldSet>
      <FieldLegend variant="label">{label}</FieldLegend>
      <div className="grid grid-cols-2 gap-2">
        {list.map((ms, i) => (
          <label key={i} className="flex items-center gap-2 text-xs">
            <span className="text-muted-foreground w-14 shrink-0">Layer {i + 1}</span>
            <Input
              className="h-7 text-right tabular-nums"
              inputMode="decimal"
              defaultValue={trimNumber(ms / 1000)}
              onBlur={(e) => {
                const n = Number(e.target.value.replace(",", "."));
                const next = [...list];
                next[i] = Number.isFinite(n) ? Math.max(0, Math.round(n * 1000)) : 0;
                while (next.length > 1 && next.at(-1) === 0) next.pop();
                onChange(next.every((v) => v === 0) ? "" : next.join(","));
              }}
            />
            <span className="text-muted-foreground">s</span>
          </label>
        ))}
      </div>
    </FieldSet>
  );
}

function ColorField({
  id,
  label,
  value,
  onChange,
}: {
  id: string;
  label: string;
  value: string;
  onChange: (v: string) => void;
}) {
  const hex = /^#[0-9a-f]{6}$/i.test(value) ? value : "#000000";
  return (
    <Field>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <InputGroup>
        <InputGroupAddon>
          <input
            type="color"
            aria-label={`${label} swatch`}
            value={hex}
            onChange={(e) => onChange(e.target.value)}
            className="size-5 cursor-pointer rounded border-0 bg-transparent p-0"
          />
        </InputGroupAddon>
        <InputGroupInput
          id={id}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="font-mono text-xs"
        />
      </InputGroup>
    </Field>
  );
}

function ObjectListField({
  label,
  schema,
  value,
  onChange,
  ctx,
}: {
  label: string;
  schema: z.ZodObject;
  value: Record<string, unknown>[];
  onChange: (v: Record<string, unknown>[]) => void;
  ctx: FormContext;
}) {
  return (
    <FieldSet>
      <FieldLegend variant="label">{label}</FieldLegend>
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
        <Button
          variant="outline"
          size="sm"
          onClick={() => onChange([...value, schema.parse({}) as Record<string, unknown>])}
        >
          <Plus data-icon="inline-start" />
          Add
        </Button>
      </div>
    </FieldSet>
  );
}

function JsonField({
  id,
  label,
  value,
  onChange,
}: {
  id: string;
  label: string;
  value: unknown;
  onChange: (v: unknown) => void;
}) {
  const [text, setText] = useState(JSON.stringify(value ?? null, null, 2));
  const [error, setError] = useState<string | null>(null);
  return (
    <Field data-invalid={error ? true : undefined}>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <Textarea
        id={id}
        aria-invalid={error ? true : undefined}
        className="font-mono text-xs"
        rows={4}
        value={text}
        onChange={(e) => {
          setText(e.target.value);
          try {
            onChange(JSON.parse(e.target.value));
            setError(null);
          } catch {
            setError("This isn't valid. Check brackets and quotes.");
          }
        }}
      />
      {error ? <FieldDescription>{error}</FieldDescription> : null}
    </Field>
  );
}
