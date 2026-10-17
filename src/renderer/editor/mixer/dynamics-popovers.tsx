import {
  COMP_RANGES,
  DUCK_RANGES,
  type Bus,
  type Comp,
  type DuckSettings,
} from "@/shared/mix/types";
import { Checkbox } from "@/renderer/components/ui/checkbox";
import { Label } from "@/renderer/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/renderer/components/ui/popover";
import { Switch } from "@/renderer/components/ui/switch";
import { cn } from "@/renderer/lib/utils";
import { db, ms } from "./audio-format";
import { commitOnce, type DragPhase } from "./gesture";
import { Knobs, type KnobSpec } from "./knob";
import { rangeKnob } from "./knob-math";
import { ReductionReadout, type Reduction } from "./meter";

type CompField = keyof typeof COMP_RANGES;

const COMP_KNOBS: Array<KnobSpec<CompField>> = [
  {
    field: "threshold",
    label: "Threshold",
    ...rangeKnob(COMP_RANGES.threshold),
    step: 0.5,
    format: db,
  },
  {
    field: "ratio",
    label: "Ratio",
    ...rangeKnob(COMP_RANGES.ratio),
    step: 0.1,
    log: true,
    format: (v) => `${v.toFixed(1)}:1`,
  },
  {
    field: "attack",
    label: "Attack",
    ...rangeKnob(COMP_RANGES.attack),
    step: 0.1,
    log: true,
    format: ms,
  },
  {
    field: "release",
    label: "Release",
    ...rangeKnob(COMP_RANGES.release),
    step: 1,
    log: true,
    format: ms,
  },
  { field: "makeup", label: "Makeup", ...rangeKnob(COMP_RANGES.makeup), step: 0.5, format: db },
];

type DuckField = keyof typeof DUCK_RANGES;

const DUCK_KNOBS: Array<KnobSpec<DuckField>> = [
  {
    field: "amount",
    label: "Amount",
    ...rangeKnob(DUCK_RANGES.amount),
    step: 0.5,
    format: (v) => `${v.toFixed(1)} dB`,
  },
  {
    field: "attack",
    label: "Attack",
    ...rangeKnob(DUCK_RANGES.attack),
    step: 1,
    log: true,
    format: ms,
  },
  {
    field: "release",
    label: "Release",
    ...rangeKnob(DUCK_RANGES.release),
    step: 1,
    log: true,
    format: ms,
  },
  {
    field: "ignoreShorterMs",
    label: "Ignore under",
    ...rangeKnob(DUCK_RANGES.ignoreShorterMs),
    step: 50,
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
  reduction: Reduction;
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
        {comp.on ? <ReductionReadout reduction={reduction} /> : null}
      </PopoverTrigger>
      <PopoverContent className="w-80">
        <SwitchHeader
          title="Compressor"
          label="Compressor on"
          checked={comp.on}
          onChange={(on) => commitOnce(onChange, { ...comp, on })}
        />
        <div className="grid grid-cols-5 gap-1">
          <Knobs
            specs={COMP_KNOBS}
            values={comp}
            disabled={!comp.on}
            onChange={(field, v, phase) => onChange({ ...comp, [field]: v }, phase)}
          />
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
          <Knobs
            specs={DUCK_KNOBS}
            values={duck}
            disabled={!duck.on}
            onChange={(field, v, phase) => onChange({ ...duck, [field]: v }, phase)}
          />
        </div>
        <p className="text-muted-foreground text-xs">
          Louder sounds lower it more. Attack and release are in milliseconds. Sounds shorter than
          Ignore under don't lower it, so quick clicks and pops don't make the music pump.
        </p>
      </PopoverContent>
    </Popover>
  );
}
