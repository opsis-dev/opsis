import { Ellipsis } from "lucide-react";
import type { Loudness, MixResult } from "@/shared/mix/result";
import {
  DEFAULT_MASTER,
  LOUDNESS_CHOICES,
  LOUDNESS_TARGETS,
  type LoudnessTarget,
  type Master,
} from "@/shared/mix/types";
import { Button } from "@/renderer/components/ui/button";
import { Label } from "@/renderer/components/ui/label";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/renderer/components/ui/select";
import { Switch } from "@/renderer/components/ui/switch";
import { db } from "./audio-format";
import { bindChange, bindEq, bindGain, setMix } from "./bindings";
import { CompPopover } from "./dynamics-popovers";
import { EqPopover } from "./eq-popover";
import { Fader } from "./fader";
import { Knob } from "./knob";
import { levelAt, Meter, reductionAt, sampleAt } from "./meter";

const LOUDNESS_LABELS: Record<LoudnessTarget, string> = {
  off: "Off",
  web: "Online video (−14)",
  podcast: "Podcast (−16)",
  broadcast: "Broadcast (−23)",
};

const QUIET_LUFS = -70;

const MASTER = { key: "master", fraction: 1 };

type Update = (fn: (m: Master) => Master) => void;

const updateMaster: Update = (fn) => setMix((mix) => ({ ...mix, master: fn(mix.master) }));

function Readout({ label, value, unit }: { label: string; value: number | null; unit: string }) {
  return (
    <div className="flex w-full items-baseline justify-between text-[10px]">
      <span className="text-muted-foreground">{label}</span>
      <span className="tabular-nums">
        {value === null || value <= QUIET_LUFS ? "—" : value.toFixed(1)}{" "}
        <span className="text-muted-foreground">{unit}</span>
      </span>
    </div>
  );
}

function LoudnessReadouts({
  loudness,
  frame,
  target,
}: {
  loudness: Loudness | null;
  frame: number;
  target: LoudnessTarget;
}) {
  const aim = LOUDNESS_TARGETS[target];
  return (
    <div className="flex w-full flex-col gap-0.5 border-t pt-1">
      <Readout label="Momentary" value={sampleAt(loudness?.momentary, frame)} unit="LUFS" />
      <Readout label="Short-term" value={sampleAt(loudness?.shortTerm, frame)} unit="LUFS" />
      <Readout label="Integrated" value={loudness?.integrated ?? null} unit="LUFS" />
      <Readout label="True peak" value={loudness?.truePeak ?? null} unit="dBTP" />
      <Readout label="Range" value={loudness?.range ?? null} unit="LU" />
      {aim ? <span className="text-muted-foreground text-[10px]">Aim: {aim.lufs} LUFS</span> : null}
    </div>
  );
}

function LimiterRow({ limiter, update }: { limiter: Master["limiter"]; update: Update }) {
  return (
    <div className="flex w-full items-center justify-between gap-1">
      <Label className="text-[10px] font-normal">Limiter</Label>
      <Switch
        size="sm"
        checked={limiter.on}
        aria-label="Limiter"
        onCheckedChange={(on) => update((m) => ({ ...m, limiter: { ...m.limiter, on } }))}
      />
      <Knob
        label="Ceiling"
        size={24}
        value={limiter.ceiling}
        min={-12}
        max={0}
        step={0.1}
        reset={-1}
        format={db}
        disabled={!limiter.on}
        onChange={bindChange((ceiling) =>
          update((m) => ({ ...m, limiter: { ...m.limiter, ceiling } })),
        )}
      />
    </div>
  );
}

function LoudnessSelect({ value, update }: { value: LoudnessTarget; update: Update }) {
  return (
    <div className="flex w-full flex-col gap-0.5">
      <span className="text-muted-foreground text-[10px]">Loudness target</span>
      <Select
        value={value}
        onValueChange={(loudness) => {
          if (loudness) update((m) => ({ ...m, loudness }));
        }}
      >
        <SelectTrigger
          size="sm"
          className="h-6 w-full px-1.5 text-[11px]"
          aria-label="Loudness target"
        >
          <SelectValue>{(v: LoudnessTarget) => LOUDNESS_LABELS[v]}</SelectValue>
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            {LOUDNESS_CHOICES.map((t) => (
              <SelectItem key={t} value={t}>
                {LOUDNESS_LABELS[t]}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>
    </div>
  );
}

export function MasterStrip({
  master,
  result,
  frame,
  holdFrames,
}: {
  master: Master;
  result: MixResult | null;
  frame: number;
  holdFrames: number;
}) {
  const reduction = reductionAt(result?.reduction.master, frame);
  return (
    <div className="bg-sidebar sticky right-0 flex w-[250px] shrink-0 gap-2 border-l px-2 py-1.5">
      <div className="flex min-w-0 flex-1 flex-col gap-1 [&>*]:shrink-0">
        <div className="flex w-full items-center justify-between">
          <span className="text-xs font-medium">Master</span>
          <Button
            variant="ghost"
            size="icon-xs"
            aria-label="Reset master"
            title="Reset master"
            onClick={() => updateMaster(() => DEFAULT_MASTER)}
          >
            <Ellipsis />
          </Button>
        </div>
        <EqPopover
          eq={master.eq}
          highpass={null}
          onChange={bindEq(MASTER, master.eq, (eq) => updateMaster((m) => ({ ...m, eq })))}
        />
        <div className="flex w-full">
          <CompPopover
            comp={master.comp}
            reduction={reduction}
            onChange={bindChange((comp) => updateMaster((m) => ({ ...m, comp })))}
          />
        </div>
        <LimiterRow limiter={master.limiter} update={updateMaster} />
        <LoudnessSelect value={master.loudness} update={updateMaster} />
        <LoudnessReadouts
          loudness={result?.loudness ?? null}
          frame={frame}
          target={master.loudness}
        />
      </div>
      <div className="flex w-12 shrink-0 flex-col items-center gap-1">
        <div className="flex min-h-16 w-full flex-1 items-stretch justify-center gap-2 py-0.5">
          <Fader
            label="Master fader"
            value={master.fader}
            onChange={bindGain(MASTER, "fader", master.fader, (fader) =>
              updateMaster((m) => ({ ...m, fader })),
            )}
          />
          <Meter
            level={levelAt(result?.levels.master, frame, holdFrames)}
            reduction={master.comp.on && reduction !== null ? reduction : undefined}
          />
        </div>
        <span className="text-[11px] whitespace-nowrap tabular-nums">{db(master.fader)} dB</span>
      </div>
    </div>
  );
}
