import { HIGHPASS_RANGE, type BusSettings, type Strip } from "@/shared/mix/types";
import { Toggle } from "@/renderer/components/ui/toggle";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { cn } from "@/renderer/lib/utils";
import { db, hz } from "./audio-format";
import { bindChange, bindEq, bindGain, bindHighpass, type PreviewTarget } from "./bindings";
import { CompPopover } from "./dynamics-popovers";
import { EqPopover } from "./eq-popover";
import { Fader } from "./fader";
import type { Change } from "./gesture";
import { Knob } from "./knob";
import { rangeKnob } from "./knob-math";
import { LiveMeter } from "./meter";
import type { Levels, Reduction } from "./meter-levels";

type StripControls = BusSettings & Pick<Strip, "mute" | "solo">;

export type StripBinding = {
  controls: StripControls;
  target: PreviewTarget;
  update: (fn: (c: StripControls) => Partial<StripControls>) => void;
};

export function stripBinding<T extends StripControls>(
  controls: T,
  target: PreviewTarget,
  update: (fn: (c: T) => T) => void,
): StripBinding {
  return { controls, target, update: (fn) => update((c) => ({ ...c, ...fn(c) })) };
}

export function StripShell({
  children,
  className,
  dim,
}: {
  children: React.ReactNode;
  className?: string;
  dim?: boolean;
}) {
  return (
    <div
      className={cn(
        "bg-card/40 flex w-[104px] shrink-0 flex-col items-center gap-1 px-1.5 py-1.5 [&>*]:shrink-0",
        dim && "opacity-60",
        className,
      )}
    >
      {children}
    </div>
  );
}

function StripToggle({
  letter,
  label,
  tone,
  tip,
  pressed,
  onChange,
}: {
  letter: string;
  label: string;
  tone: string;
  tip: string;
  pressed: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <Toggle
            size="sm"
            variant="outline"
            data-custom-pressed
            className={cn("h-6 min-w-7 px-1 text-[11px]", tone)}
            pressed={pressed}
            aria-label={label}
            onPressedChange={onChange}
          />
        }
      >
        {letter}
      </TooltipTrigger>
      <TooltipContent>{tip}</TooltipContent>
    </Tooltip>
  );
}

function MuteSolo({
  mute,
  solo,
  onMute,
  onSolo,
}: {
  mute: boolean;
  solo: boolean;
  onMute: (v: boolean) => void;
  onSolo: (v: boolean) => void;
}) {
  return (
    <div className="flex gap-1">
      <StripToggle
        letter="M"
        label="Mute"
        tone="data-[pressed]:bg-destructive/30"
        tip="Mute. Saved with the project and left out of exports."
        pressed={mute}
        onChange={onMute}
      />
      <StripToggle
        letter="S"
        label="Solo"
        tone="data-[pressed]:bg-meter-mid/40"
        tip="Solo. While anything is soloed, only soloed sounds play, in exports too."
        pressed={solo}
        onChange={onSolo}
      />
    </div>
  );
}

export function HighpassKnob({ controls, target, update }: StripBinding) {
  return (
    <Knob
      label="Hz"
      size={24}
      value={controls.highpass.freq}
      {...rangeKnob(HIGHPASS_RANGE)}
      step={1}
      log
      format={hz}
      disabled={!controls.highpass.on}
      onChange={bindHighpass(target, (freq) =>
        update((c) => ({ highpass: { ...c.highpass, freq } })),
      )}
    />
  );
}

export function StripTone({
  controls,
  target,
  update,
  reduction,
}: StripBinding & { reduction: Reduction }) {
  return (
    <>
      <EqPopover
        eq={controls.eq}
        highpass={controls.highpass.on ? controls.highpass.freq : null}
        onChange={bindEq(target, controls.eq, (eq) => update(() => ({ eq })))}
      />
      <div className="flex w-full items-center gap-1">
        <Toggle
          size="sm"
          variant="outline"
          className="h-5 min-w-0 px-1 text-[10px]"
          pressed={controls.highpass.on}
          aria-label="Low cut"
          title="Cut rumble below the low-cut frequency"
          onPressedChange={(on) => update((c) => ({ highpass: { ...c.highpass, on } }))}
        >
          Low cut
        </Toggle>
        <CompPopover
          comp={controls.comp}
          reduction={reduction}
          onChange={bindChange((comp) => update(() => ({ comp })))}
        />
      </div>
    </>
  );
}

export function FaderMeter({
  label,
  value,
  onChange,
  levels,
  reduction,
  compOn,
  holdFrames,
  className,
}: {
  label: string;
  value: number;
  onChange: Change<number>;
  levels: Levels;
  reduction: Reduction;
  compOn: boolean;
  holdFrames: number;
  className?: string;
}) {
  return (
    <>
      <div className={cn("flex min-h-16 flex-1 items-stretch gap-2 py-0.5", className)}>
        <Fader label={label} value={value} onChange={onChange} />
        <LiveMeter levels={levels} reduction={reduction} compOn={compOn} holdFrames={holdFrames} />
      </div>
      <span className="text-[11px] whitespace-nowrap tabular-nums">{db(value)} dB</span>
    </>
  );
}

export function StripLevel({
  controls,
  target,
  update,
  label,
  levels,
  reduction,
  holdFrames,
}: StripBinding & { label: string; levels: Levels; reduction: Reduction; holdFrames: number }) {
  return (
    <>
      <FaderMeter
        label={label}
        value={controls.fader}
        onChange={bindGain(target, "fader", controls.fader, (fader) => update(() => ({ fader })))}
        levels={levels}
        reduction={reduction}
        compOn={controls.comp.on}
        holdFrames={holdFrames}
        className="shrink"
      />
      <MuteSolo
        mute={controls.mute}
        solo={controls.solo}
        onMute={(mute) => update(() => ({ mute }))}
        onSolo={(solo) => update(() => ({ solo }))}
      />
    </>
  );
}
