import type { Bus, Comp, DuckSettings } from "@/shared/mix/types";
import { Checkbox } from "@/renderer/components/ui/checkbox";
import { Label } from "@/renderer/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/renderer/components/ui/popover";
import { Switch } from "@/renderer/components/ui/switch";
import { cn } from "@/renderer/lib/utils";
import { db, ms } from "./audio-format";
import { commitOnce, Knob, type DragPhase, type KnobSpec } from "./knob";

type CompField = "threshold" | "ratio" | "attack" | "release" | "makeup";

const COMP_KNOBS: Array<KnobSpec<CompField>> = [
  { field: "threshold", label: "Threshold", min: -60, max: 0, step: 0.5, reset: -18, format: db },
  {
    field: "ratio",
    label: "Ratio",
    min: 1,
    max: 20,
    step: 0.1,
    reset: 3,
    log: true,
    format: (v) => `${v.toFixed(1)}:1`,
  },
  {
    field: "attack",
    label: "Attack",
    min: 0.1,
    max: 500,
    step: 0.1,
    reset: 10,
    log: true,
    format: ms,
  },
  {
    field: "release",
    label: "Release",
    min: 5,
    max: 3000,
    step: 1,
    reset: 150,
    log: true,
    format: ms,
  },
  { field: "makeup", label: "Makeup", min: 0, max: 24, step: 0.5, reset: 0, format: db },
];

type DuckField = "amount" | "attack" | "release" | "ignoreShorterMs";

const DUCK_KNOBS: Array<KnobSpec<DuckField>> = [
  {
    field: "amount",
    label: "Amount",
    min: 1,
    max: 24,
    step: 0.5,
    reset: 10,
    format: (v) => `${v.toFixed(1)} dB`,
  },
  {
    field: "attack",
    label: "Attack",
    min: 5,
    max: 2000,
    step: 1,
    reset: 80,
    log: true,
    format: ms,
  },
  {
    field: "release",
    label: "Release",
    min: 20,
    max: 5000,
    step: 1,
    reset: 400,
    log: true,
    format: ms,
  },
  {
    field: "ignoreShorterMs",
    label: "Ignore under",
    min: 0,
    max: 5000,
    step: 50,
    reset: 0,
    format: (v) => (v ? ms(v) : "Off"),
  },
];

function SwitchHeader({
  title,
  checked,
  label,
  onChange,
}: {
  title: string;
  checked: boolean;
  label: string;
  onChange: (on: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between">
      <Label className="text-sm">{title}</Label>
      <Switch checked={checked} aria-label={label} onCheckedChange={onChange} />
    </div>
  );
}

export function CompPopover({
  comp,
  reduction,
  onChange,
}: {
  comp: Comp;
  reduction: number | null;
  onChange: (comp: Comp, phase: DragPhase) => void;
}) {
  return (
    <Popover>
      <PopoverTrigger
        render={
          <button
            type="button"
            className={cn(
              "flex h-5 min-w-0 flex-1 items-center justify-between rounded border px-1.5 text-[10px]",
              comp.on ? "border-keyframe/60" : "text-muted-foreground",
            )}
            aria-label="Compressor"
          />
        }
      >
        <span>Comp</span>
        {comp.on && reduction !== null ? (
          <span className="tabular-nums">−{reduction.toFixed(1)}</span>
        ) : null}
      </PopoverTrigger>
      <PopoverContent className="w-80">
        <SwitchHeader
          title="Compressor"
          label="Compressor on"
          checked={comp.on}
          onChange={(on) => commitOnce(onChange, { ...comp, on })}
        />
        <div className="grid grid-cols-5 gap-1">
          {COMP_KNOBS.map(({ field, reset, ...spec }) => (
            <Knob
              key={field}
              {...spec}
              value={comp[field]}
              reset={reset ?? comp[field]}
              disabled={!comp.on}
              onChange={(v, phase) => onChange({ ...comp, [field]: v }, phase)}
            />
          ))}
        </div>
        <p className="text-muted-foreground text-xs">Attack and release are in milliseconds.</p>
      </PopoverContent>
    </Popover>
  );
}

export function DuckPopover({
  bus,
  buses,
  onChange,
}: {
  bus: Bus;
  buses: Bus[];
  onChange: (duck: DuckSettings, phase: DragPhase) => void;
}) {
  const duck = bus.duck;
  const setKey = (id: string, on: boolean) =>
    commitOnce(onChange, {
      ...duck,
      keys: on ? [...duck.keys, id] : duck.keys.filter((k) => k !== id),
    });
  return (
    <Popover>
      <PopoverTrigger
        render={
          <button
            type="button"
            className={cn(
              "w-full rounded border px-1.5 py-0.5 text-[10px]",
              duck.on ? "border-keyframe/60" : "text-muted-foreground",
            )}
          />
        }
      >
        {duck.on ? "Ducks" : "No ducking"}
      </PopoverTrigger>
      <PopoverContent className="w-80">
        <SwitchHeader
          title="Lower this bus while others play"
          label="Ducking on"
          checked={duck.on}
          onChange={(on) => commitOnce(onChange, { ...duck, on })}
        />
        <div className="flex flex-col gap-1.5">
          <span className="text-muted-foreground text-xs">Listen to</span>
          {buses
            .filter((b) => b.id !== bus.id)
            .map((b) => (
              <Label key={b.id} className="flex items-center gap-2 text-sm font-normal">
                <Checkbox
                  checked={duck.keys.includes(b.id)}
                  onCheckedChange={(on) => setKey(b.id, on)}
                />
                {b.name}
              </Label>
            ))}
        </div>
        <div className="grid grid-cols-4 gap-1">
          {DUCK_KNOBS.map(({ field, reset, ...spec }) => (
            <Knob
              key={field}
              {...spec}
              value={duck[field]}
              reset={reset ?? duck[field]}
              disabled={!duck.on}
              onChange={(v, phase) => onChange({ ...duck, [field]: v }, phase)}
            />
          ))}
        </div>
        <p className="text-muted-foreground text-xs">
          Louder sounds lower it more. Attack and release are in milliseconds. Sounds shorter than
          Ignore under don't lower it, so quick clicks and pops don't make the music pump.
        </p>
      </PopoverContent>
    </Popover>
  );
}
