import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import type { Eq, EqBand, EqType } from "@/generated/bindings";
import { mixerBook } from "@/store/catalog";
import { OptionSelect } from "../option-select";
import { formatDb, formatHz } from "./audio-format";
import type { EqGestures } from "./bindings";
import { EqCurve } from "./eq-curve";
import { commitOnce, mapGesture, type Gesture } from "./gesture";
import { Knobs, type KnobSpec } from "./knob";
import { SwitchHeader } from "./switch-header";

const BAND_LABELS: Record<EqType, string> = {
  lowshelf: "Low shelf",
  peak: "Bell",
  highshelf: "High shelf",
};

function bandKnobs(defaults: EqBand): Array<KnobSpec<"freq" | "gain" | "q">> {
  return [
    {
      field: "freq",
      label: "Freq",
      ...mixerBook().eqBand.freq,
      step: 1,
      reset: defaults.freq,
      log: true,
      format: formatHz,
      unit: "Hz",
    },
    {
      field: "gain",
      label: "Gain",
      ...mixerBook().eqBand.gain,
      step: 0.5,
      reset: defaults.gain,
      format: formatDb,
      unit: "dB",
    },
    {
      field: "q",
      label: "Width",
      ...mixerBook().eqBand.q,
      step: 0.05,
      reset: defaults.q,
      log: true,
      format: (v) => v.toFixed(2),
    },
  ];
}

function BandControls({
  band,
  defaults,
  disabled,
  gesture,
}: {
  band: EqBand;
  defaults: EqBand;
  disabled: boolean;
  gesture: Gesture<Partial<EqBand>>;
}) {
  return (
    <div className="flex flex-col items-center gap-1">
      <OptionSelect
        label="Band shape"
        value={band.type}
        options={mixerBook().eqTypes}
        optionLabel={(t) => BAND_LABELS[t]}
        className="h-6 w-full px-1.5 text-[11px]"
        onChange={(type) => commitOnce(gesture, { type })}
      />
      <Knobs
        specs={bandKnobs(defaults)}
        values={band}
        disabled={disabled}
        gestureFor={(field) => mapGesture(gesture, (v: number) => ({ [field]: v }))}
      />
    </div>
  );
}

export function EqPopover({
  eq,
  highpass,
  gestures,
}: {
  eq: Eq;
  highpass: number | null;
  gestures: EqGestures;
}) {
  function bandGesture(i: number): Gesture<Partial<EqBand>> {
    return mapGesture(gestures(i), (patch: Partial<EqBand>) => ({
      ...eq,
      bands: eq.bands.map((b, k) => (k === i ? { ...b, ...patch } : b)),
    }));
  }
  return (
    <Popover>
      <PopoverTrigger
        render={
          <button
            type="button"
            className="flex flex-col items-center gap-0.5"
            aria-label="Equaliser"
            title="Equaliser"
          />
        }
      >
        <EqCurve bands={eq.bands} on={eq.on} width={88} height={20} highpass={highpass} />
        <span className="text-muted-foreground text-[10px] leading-none">EQ</span>
      </PopoverTrigger>
      <PopoverContent className="w-[420px]">
        <SwitchHeader
          title="Equaliser"
          label="Equaliser on"
          checked={eq.on}
          onChange={(on) => commitOnce(gestures(), { ...eq, on })}
        />
        <EqCurve bands={eq.bands} on={eq.on} width={388} height={90} highpass={highpass} />
        <div className="grid grid-cols-4 gap-2">
          {eq.bands.map((band, i) => (
            <BandControls
              key={i}
              band={band}
              defaults={mixerBook().defaultStrip.eq.bands[i]}
              disabled={!eq.on}
              gesture={bandGesture(i)}
            />
          ))}
        </div>
      </PopoverContent>
    </Popover>
  );
}
