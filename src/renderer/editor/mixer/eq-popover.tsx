import { Popover, PopoverContent, PopoverTrigger } from "@/renderer/components/ui/popover";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/renderer/components/ui/select";
import {
  DEFAULT_EQ_BANDS,
  EQ_BAND_BOUNDS,
  EQ_TYPES,
  type Eq,
  type EqBand,
  type EqType,
} from "@/shared/mix/types";
import { db, hz } from "./audio-format";
import { SwitchHeader } from "./dynamics-popovers";
import { EqCurve } from "./eq-curve";
import { commitOnce, type DragPhase } from "./gesture";
import { Knobs, type KnobSpec } from "./knob";

const BAND_LABELS: Record<EqType, string> = {
  lowshelf: "Low shelf",
  peak: "Bell",
  highshelf: "High shelf",
};

const bandKnobs = (defaults: EqBand): Array<KnobSpec<"freq" | "gain" | "q">> => [
  {
    field: "freq",
    label: "Freq",
    ...EQ_BAND_BOUNDS.freq,
    step: 1,
    reset: defaults.freq,
    log: true,
    format: hz,
  },
  {
    field: "gain",
    label: "Gain",
    ...EQ_BAND_BOUNDS.gain,
    step: 0.5,
    reset: defaults.gain,
    format: db,
  },
  {
    field: "q",
    label: "Width",
    ...EQ_BAND_BOUNDS.q,
    step: 0.05,
    reset: defaults.q,
    log: true,
    format: (v) => v.toFixed(2),
  },
];

function BandControls({
  band,
  defaults,
  disabled,
  onChange,
}: {
  band: EqBand;
  defaults: EqBand;
  disabled: boolean;
  onChange: (patch: Partial<EqBand>, phase: DragPhase) => void;
}) {
  return (
    <div className="flex flex-col items-center gap-1">
      <Select
        value={band.type}
        onValueChange={(type) => {
          if (type) commitOnce(onChange, { type });
        }}
      >
        <SelectTrigger size="sm" className="h-6 w-full px-1.5 text-[11px]" aria-label="Band shape">
          <SelectValue>{(v: EqType) => BAND_LABELS[v]}</SelectValue>
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            {EQ_TYPES.map((t) => (
              <SelectItem key={t} value={t}>
                {BAND_LABELS[t]}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>
      <Knobs
        specs={bandKnobs(defaults)}
        values={band}
        disabled={disabled}
        onChange={(field, v, phase) => onChange({ [field]: v }, phase)}
      />
    </div>
  );
}

export function EqPopover({
  eq,
  highpass,
  onChange,
}: {
  eq: Eq;
  highpass: number | null;
  onChange: (eq: Eq, phase: DragPhase, band?: number) => void;
}) {
  const setBand = (i: number, patch: Partial<EqBand>, phase: DragPhase) =>
    onChange(
      { ...eq, bands: eq.bands.map((b, k) => (k === i ? { ...b, ...patch } : b)) },
      phase,
      i,
    );
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
          onChange={(on) => commitOnce(onChange, { ...eq, on })}
        />
        <EqCurve bands={eq.bands} on={eq.on} width={388} height={90} highpass={highpass} />
        <div className="grid grid-cols-4 gap-2">
          {eq.bands.map((band, i) => (
            <BandControls
              key={i}
              band={band}
              defaults={DEFAULT_EQ_BANDS[i]}
              disabled={!eq.on}
              onChange={(patch, phase) => setBand(i, patch, phase)}
            />
          ))}
        </div>
      </PopoverContent>
    </Popover>
  );
}
