import type { ComponentProps } from "react";
import { Checkbox } from "@/renderer/components/ui/checkbox";
import { Label } from "@/renderer/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/renderer/components/ui/popover";
import { cn } from "@/renderer/lib/utils";
import {
  COMP_RANGES,
  DUCK_RANGES,
  type Bus,
  type Comp,
  type DuckSettings,
} from "@/shared/mix/types";
import { toggleIds } from "../../store/ui";
import { formatDb, formatMs } from "./audio-format";
import { commitOnce, mapGesture, type Gesture } from "./gesture";
import { Knobs, type KnobSpec } from "./knob";
import { rangeProps } from "./knob-math";
import { ReductionReadout } from "./meter";
import type { Reduction } from "./meter-levels";
import { SwitchHeader } from "./switch-header";

type CompField = keyof typeof COMP_RANGES;

const COMP_KNOBS: Array<KnobSpec<CompField>> = [
  {
    field: "threshold",
    label: "Threshold",
    ...rangeProps(COMP_RANGES.threshold),
    step: 0.5,
    format: formatDb,
    unit: "dB",
  },
  {
    field: "ratio",
    label: "Ratio",
    ...rangeProps(COMP_RANGES.ratio),
    step: 0.1,
    log: true,
    format: (v) => `${v.toFixed(1)}:1`,
  },
  {
    field: "attack",
    label: "Attack",
    ...rangeProps(COMP_RANGES.attack),
    step: 0.1,
    log: true,
    format: formatMs,
    unit: "ms",
  },
  {
    field: "release",
    label: "Release",
    ...rangeProps(COMP_RANGES.release),
    step: 1,
    log: true,
    format: formatMs,
    unit: "ms",
  },
  {
    field: "makeup",
    label: "Makeup",
    ...rangeProps(COMP_RANGES.makeup),
    step: 0.5,
    format: formatDb,
    unit: "dB",
  },
];

type DuckField = keyof typeof DUCK_RANGES;

const DUCK_KNOBS: Array<KnobSpec<DuckField>> = [
  {
    field: "amount",
    label: "Amount",
    ...rangeProps(DUCK_RANGES.amount),
    step: 0.5,
    format: (v) => v.toFixed(1),
    unit: "dB",
  },
  {
    field: "attack",
    label: "Attack",
    ...rangeProps(DUCK_RANGES.attack),
    step: 1,
    log: true,
    format: formatMs,
    unit: "ms",
  },
  {
    field: "release",
    label: "Release",
    ...rangeProps(DUCK_RANGES.release),
    step: 1,
    log: true,
    format: formatMs,
    unit: "ms",
  },
  {
    field: "ignoreShorterMs",
    label: "Ignore under",
    ...rangeProps(DUCK_RANGES.ignoreShorterMs),
    step: 50,
    format: (v) => (v ? formatMs(v) : "Off"),
    unit: "ms",
  },
];

function PopoverButton({ on, className, ...props }: ComponentProps<"button"> & { on: boolean }) {
  return (
    <button
      type="button"
      {...props}
      className={cn(
        "rounded border px-1.5 text-[10px]",
        on ? "border-keyframe/60" : "text-muted-foreground",
        className,
      )}
    />
  );
}

export function CompPopover({
  comp,
  reduction,
  gesture,
}: {
  comp: Comp;
  reduction: Reduction;
  gesture: Gesture<Comp>;
}) {
  return (
    <Popover>
      <PopoverTrigger
        render={
          <PopoverButton
            on={comp.on}
            className="flex h-5 min-w-0 flex-1 items-center justify-between"
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
          onChange={(on) => commitOnce(gesture, { ...comp, on })}
        />
        <div className="grid grid-cols-5 gap-1">
          <Knobs
            specs={COMP_KNOBS}
            values={comp}
            disabled={!comp.on}
            gestureFor={(field) => mapGesture(gesture, (v: number) => ({ ...comp, [field]: v }))}
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
  gesture,
}: {
  bus: Bus;
  buses: Bus[];
  gesture: Gesture<DuckSettings>;
}) {
  const duck = bus.duck;
  return (
    <Popover>
      <PopoverTrigger render={<PopoverButton on={duck.on} className="w-full py-0.5" />}>
        {duck.on ? "Ducks" : "No ducking"}
      </PopoverTrigger>
      <PopoverContent className="w-80">
        <SwitchHeader
          title="Lower this bus while others play"
          label="Ducking on"
          checked={duck.on}
          onChange={(on) => commitOnce(gesture, { ...duck, on })}
        />
        <div className="flex flex-col gap-1.5">
          <span className="text-muted-foreground text-xs">Listen to</span>
          {buses
            .filter((b) => b.id !== bus.id)
            .map((b) => (
              <Label key={b.id} className="flex items-center gap-2 text-sm font-normal">
                <Checkbox
                  checked={duck.keys.includes(b.id)}
                  onCheckedChange={() =>
                    commitOnce(gesture, { ...duck, keys: toggleIds(duck.keys, [b.id]) })
                  }
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
            gestureFor={(field) => mapGesture(gesture, (v: number) => ({ ...duck, [field]: v }))}
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
