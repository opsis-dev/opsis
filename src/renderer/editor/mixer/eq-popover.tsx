import { EQ_TYPES, type Eq, type EqBand, type EqType } from "@/shared/mix/types";
import { Label } from "@/renderer/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/renderer/components/ui/popover";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/renderer/components/ui/select";
import { Switch } from "@/renderer/components/ui/switch";
import { db, hz } from "./audio-format";
import { EqCurve } from "./eq-curve";
import { commitOnce, Knob, type DragPhase, type KnobSpec } from "./knob";

const BAND_LABELS: Record<EqType, string> = {
  lowshelf: "Low shelf",
  peak: "Bell",
  highshelf: "High shelf",
};

const BAND_KNOBS: Array<KnobSpec<"freq" | "gain" | "q">> = [
  { field: "freq", label: "Freq", min: 20, max: 20_000, step: 1, log: true, format: hz },
  { field: "gain", label: "Gain", min: -18, max: 18, step: 0.5, reset: 0, format: db },
  {
    field: "q",
    label: "Width",
    min: 0.1,
    max: 10,
    step: 0.05,
    reset: 1,
    log: true,
    format: (v) => v.toFixed(2),
  },
];

function BandControls({
  band,
  disabled,
  onChange,
}: {
  band: EqBand;
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
      {BAND_KNOBS.map(({ field, reset, ...spec }) => (
        <Knob
          key={field}
          {...spec}
          value={band[field]}
          reset={reset ?? band[field]}
          disabled={disabled}
          onChange={(v, phase) => onChange({ [field]: v }, phase)}
        />
      ))}
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
        <div className="flex items-center justify-between">
          <Label className="text-sm">Equaliser</Label>
          <Switch
            checked={eq.on}
            aria-label="Equaliser on"
            onCheckedChange={(on) => commitOnce(onChange, { ...eq, on })}
          />
        </div>
        <EqCurve bands={eq.bands} on={eq.on} width={388} height={90} highpass={highpass} />
        <div className="grid grid-cols-4 gap-2">
          {eq.bands.map((band, i) => (
            <BandControls
              key={i}
              band={band}
              disabled={!eq.on}
              onChange={(patch, phase) => setBand(i, patch, phase)}
            />
          ))}
        </div>
      </PopoverContent>
    </Popover>
  );
}
