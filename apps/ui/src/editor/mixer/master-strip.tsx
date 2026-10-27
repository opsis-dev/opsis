import { signedText } from "@opsis/video/common/format";
import { Ellipsis } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import type {
  Loudness,
  LoudnessTarget,
  Master,
  MasterPatch,
  MixResult,
} from "@/generated/bindings";
import { mixerBook } from "@/store/catalog";
import { usePlayback } from "../../store/playback";
import { workflowEdits } from "../../store/workflow";
import { OptionSelect } from "../option-select";
import { formatDb, formatReading } from "./audio-format";
import { bindChange, bindEq, bindGain, MASTER_TARGET } from "./bindings";
import { CompPopover } from "./dynamics-popovers";
import { EqPopover } from "./eq-popover";
import { Knob } from "./knob";
import { rangeProps } from "./knob-math";
import { sampleAt } from "./meter-levels";
import { FaderMeter } from "./strip-parts";

const loudnessChoice = (target: LoudnessTarget) =>
  mixerBook().loudness.find((c) => c.target === target);

const loudnessGoal = (target: LoudnessTarget) => loudnessChoice(target)?.goal ?? null;

function loudnessLabel(target: LoudnessTarget): string {
  const choice = loudnessChoice(target);
  if (!choice) return "";
  return choice.goal ? `${choice.label} (${signedText(choice.goal.lufs)})` : choice.label;
}

const QUIET_LUFS = -70;

const updateMaster = (patch: MasterPatch) => workflowEdits.mix({ op: "master", patch });

function Readout({
  label,
  value,
  unit,
  floor = -Infinity,
}: {
  label: string;
  value: number | null;
  unit: string;
  floor?: number;
}) {
  return (
    <div className="flex w-full items-baseline justify-between text-[10px]">
      <span className="text-muted-foreground">{label}</span>
      <span className="tabular-nums">
        {value === null ? "—" : formatReading(value, floor)}{" "}
        <span className="text-muted-foreground">{unit}</span>
      </span>
    </div>
  );
}

function LoudnessReadouts({
  loudness,
  target,
}: {
  loudness: Loudness | null;
  target: LoudnessTarget;
}) {
  const frame = usePlayback((s) => s.frame);
  const aim = loudnessGoal(target);
  return (
    <div className="flex w-full flex-col gap-0.5 border-t pt-1">
      <Readout
        label="Momentary"
        value={sampleAt(loudness?.momentary, frame)}
        unit="LUFS"
        floor={QUIET_LUFS}
      />
      <Readout
        label="Short-term"
        value={sampleAt(loudness?.shortTerm, frame)}
        unit="LUFS"
        floor={QUIET_LUFS}
      />
      <Readout
        label="Integrated"
        value={loudness?.integrated ?? null}
        unit="LUFS"
        floor={QUIET_LUFS}
      />
      <Readout label="True peak" value={loudness?.truePeak ?? null} unit="dBTP" />
      <Readout label="Range" value={loudness?.range ?? null} unit="LU" />
      {aim ? (
        <span className="text-muted-foreground text-[10px]">Aim: {signedText(aim.lufs)} LUFS</span>
      ) : null}
    </div>
  );
}

function LimiterRow({ limiter }: { limiter: Master["limiter"] }) {
  return (
    <div className="flex w-full items-center justify-between gap-1">
      <Label className="text-[10px] font-normal">Limiter</Label>
      <Switch
        size="sm"
        checked={limiter.on}
        aria-label="Limiter"
        onCheckedChange={(on) => updateMaster({ limiter: { ...limiter, on } })}
      />
      <Knob
        label="Ceiling"
        size={24}
        value={limiter.ceiling}
        {...rangeProps(mixerBook().limiterCeiling)}
        step={0.1}
        format={formatDb}
        unit="dB"
        disabled={!limiter.on}
        gesture={bindChange((ceiling) => updateMaster({ limiter: { ...limiter, ceiling } }))}
      />
    </div>
  );
}

function LoudnessSelect({ value }: { value: LoudnessTarget }) {
  return (
    <div className="flex w-full flex-col gap-0.5">
      <span className="text-muted-foreground text-[10px]">Loudness target</span>
      <OptionSelect
        label="Loudness target"
        value={value}
        options={mixerBook().loudness.map((c) => c.target)}
        optionLabel={loudnessLabel}
        className="h-6 w-full px-1.5 text-[11px]"
        onChange={(loudness) => updateMaster({ loudness })}
      />
    </div>
  );
}

export function MasterStrip({ master, result }: { master: Master; result: MixResult | null }) {
  const reduction = result?.reduction.master;
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
            onClick={() => updateMaster(mixerBook().defaultMaster)}
          >
            <Ellipsis />
          </Button>
        </div>
        <EqPopover
          eq={master.eq}
          highpass={null}
          gestures={bindEq(MASTER_TARGET, master.eq, (eq) => updateMaster({ eq }))}
        />
        <div className="flex w-full">
          <CompPopover
            comp={master.comp}
            reduction={reduction}
            gesture={bindChange((comp) => updateMaster({ comp }))}
          />
        </div>
        <LimiterRow limiter={master.limiter} />
        <LoudnessSelect value={master.loudness} />
        <LoudnessReadouts loudness={result?.loudness ?? null} target={master.loudness} />
      </div>
      <div className="flex w-12 shrink-0 flex-col items-center gap-1">
        <FaderMeter
          label="Master fader"
          value={master.fader}
          gesture={bindGain(MASTER_TARGET, "fader", master.fader, (fader) =>
            updateMaster({ fader }),
          )}
          levels={result?.levels.master}
          reduction={reduction}
          compOn={master.comp.on}
          className="w-full justify-center"
        />
      </div>
    </div>
  );
}
