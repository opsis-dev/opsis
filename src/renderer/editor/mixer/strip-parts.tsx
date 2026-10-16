import type { BusSettings, Strip } from "@/shared/mix/types";
import { Toggle } from "@/renderer/components/ui/toggle";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { cn } from "@/renderer/lib/utils";
import { db, hz } from "./audio-format";
import { bindChange, bindEq, bindGain, bindHighpass, type PreviewTarget } from "./bindings";
import { CompPopover } from "./dynamics-popovers";
import { EqPopover } from "./eq-popover";
import { Fader } from "./fader";
import { Knob } from "./knob";
import { Meter, type Level } from "./meter";

export type StripControls = BusSettings & Pick<Strip, "mute" | "solo">;

export type StripBinding = {
  controls: StripControls;
  target: PreviewTarget;
  update: (fn: (c: StripControls) => Partial<StripControls>) => void;
};

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
      <Tooltip>
        <TooltipTrigger
          render={
            <Toggle
              size="sm"
              variant="outline"
              data-custom-pressed
              className="data-[pressed]:bg-destructive/30 h-6 min-w-7 px-1 text-[11px]"
              pressed={mute}
              aria-label="Mute"
              onPressedChange={onMute}
            />
          }
        >
          M
        </TooltipTrigger>
        <TooltipContent>Mute. Saved with the project and left out of exports.</TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger
          render={
            <Toggle
              size="sm"
              variant="outline"
              data-custom-pressed
              className="data-[pressed]:bg-meter-mid/40 h-6 min-w-7 px-1 text-[11px]"
              pressed={solo}
              aria-label="Solo"
              onPressedChange={onSolo}
            />
          }
        >
          S
        </TooltipTrigger>
        <TooltipContent>
          Solo. While anything is soloed, only soloed sounds play, in exports too.
        </TooltipContent>
      </Tooltip>
    </div>
  );
}

export function HighpassKnob({ controls, target, update }: StripBinding) {
  return (
    <Knob
      label="Hz"
      size={24}
      value={controls.highpass.freq}
      min={20}
      max={1000}
      step={1}
      reset={80}
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
}: StripBinding & { reduction: number | null }) {
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

export function StripLevel({
  controls,
  target,
  update,
  label,
  level,
  reduction,
}: StripBinding & { label: string; level: Level; reduction: number | null }) {
  return (
    <>
      <div className="flex min-h-16 flex-1 shrink items-stretch gap-2 py-0.5">
        <Fader
          label={label}
          value={controls.fader}
          onChange={bindGain(target, "fader", controls.fader, (fader) => update(() => ({ fader })))}
        />
        <Meter
          level={level}
          reduction={controls.comp.on && reduction !== null ? reduction : undefined}
        />
      </div>
      <span className="text-[11px] tabular-nums">{db(controls.fader)} dB</span>
      <MuteSolo
        mute={controls.mute}
        solo={controls.solo}
        onMute={(mute) => update(() => ({ mute }))}
        onSolo={(solo) => update(() => ({ solo }))}
      />
    </>
  );
}
