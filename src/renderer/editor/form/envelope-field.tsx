import { Plus, X } from "lucide-react";
import { z } from "zod";
import {
  AUDIO_ENVELOPES,
  EASES,
  VISUAL_ENVELOPES,
  keyframeSchema,
  type AudioEnvelopeKey,
  type Ease,
  type Keyframe,
  type VisualEnvelopeKey,
} from "@/shared/schema/timeline";
import { Button } from "@/renderer/components/ui/button";
import { FieldLegend, FieldSet } from "@/renderer/components/ui/field";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/renderer/components/ui/input-group";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/renderer/components/ui/select";
import { parseDecimal, trimNumber, unitScale, useDecimalInput } from "./decimal";
import { Help } from "./field-shell";
import type { FieldProps } from "./types";

type EnvelopeKey = VisualEnvelopeKey | AudioEnvelopeKey;
type Format = {
  unit: string;
  toShown: (v: number) => number;
  fromShown: (v: number) => number;
  initial: number;
};

const envelopeSchema = z.record(z.string(), z.array(keyframeSchema).optional()).catch({});
type Envelope = z.infer<typeof envelopeSchema>;

const ENVELOPE_KEYS: readonly string[] = [...VISUAL_ENVELOPES, ...AUDIO_ENVELOPES];
const isEnvelopeKey = (key: string): key is EnvelopeKey => ENVELOPE_KEYS.includes(key);

const percent = { unit: "%", toShown: (v: number) => v * 100, fromShown: (v: number) => v / 100 };

const ENVELOPE_FORMAT: Record<EnvelopeKey, Format> = {
  opacity: { ...percent, initial: 1 },
  x: { ...percent, initial: 0 },
  y: { ...percent, initial: 0 },
  scale: { unit: "×", toShown: (v) => v, fromShown: (v) => v, initial: 1 },
  volume: { ...percent, initial: 1 },
  pan: { ...percent, unit: "", initial: 0 },
};

const EASE_LABELS: Record<Ease, string> = {
  linear: "Steady",
  in: "Speed up",
  out: "Slow down",
  "in-out": "Smooth",
  hold: "Jump",
};
const EASE_ITEMS = EASES.map((e) => ({ value: e, label: EASE_LABELS[e] }));
const easeOf = (v: unknown): Ease | undefined => EASES.find((e) => e === v);

export function EnvelopeField({ label, value, onChange, meta, ctx }: FieldProps<unknown>) {
  const env = envelopeSchema.parse(value);
  const props = Object.keys(meta.labels ?? {}).filter(isEnvelopeKey);
  const write = (prop: EnvelopeKey, keys: Keyframe[]) => {
    const next: Envelope = { ...env };
    if (keys.length) next[prop] = keys.toSorted((a, b) => a.frame - b.frame);
    else delete next[prop];
    onChange(next);
  };
  return (
    <FieldSet>
      <FieldLegend variant="label">{label}</FieldLegend>
      <Help meta={meta} />
      {props.map((prop) => (
        <EnvelopeTrack
          key={prop}
          label={meta.labels?.[prop] ?? prop}
          keys={env[prop] ?? []}
          format={ENVELOPE_FORMAT[prop]}
          fps={ctx.fps}
          onChange={(keys) => write(prop, keys)}
        />
      ))}
    </FieldSet>
  );
}

function EnvelopeTrack({
  label,
  keys,
  format,
  fps,
  onChange,
}: {
  label: string;
  keys: Keyframe[];
  format: Format;
  fps: number;
  onChange: (keys: Keyframe[]) => void;
}) {
  const add = () => {
    const last = keys.at(-1);
    onChange([
      ...keys,
      {
        frame: last ? last.frame + fps : 0,
        value: last ? last.value : format.initial,
        ease: "linear",
      },
    ]);
  };
  const updateKey = (i: number, patch: Partial<Keyframe>) =>
    onChange(keys.map((k, j) => (j === i ? { ...k, ...patch } : k)));
  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium">{label}</span>
        <Button variant="ghost" size="xs" onClick={add}>
          <Plus data-icon="inline-start" />
          Add
        </Button>
      </div>
      {keys.map((k, i) => (
        <KeyframeRow
          key={i}
          keyframe={k}
          format={format}
          fps={fps}
          onChange={(patch) => updateKey(i, patch)}
          onRemove={() => onChange(keys.filter((_, j) => j !== i))}
        />
      ))}
    </div>
  );
}

function KeyframeRow({
  keyframe,
  format,
  fps,
  onChange,
  onRemove,
}: {
  keyframe: Keyframe;
  format: Format;
  fps: number;
  onChange: (patch: Partial<Keyframe>) => void;
  onRemove: () => void;
}) {
  const time = unitScale("frames", fps);
  const at = useDecimalInput(trimNumber(keyframe.frame / time.factor), {
    onCommit: (text) => {
      const n = parseDecimal(text);
      if (n !== null) onChange({ frame: Math.max(0, Math.round(n * time.factor)) });
    },
  });
  const amount = useDecimalInput(
    trimNumber(Math.round(format.toShown(keyframe.value) * 100) / 100),
    {
      onCommit: (text) => {
        const n = parseDecimal(text);
        if (n !== null) onChange({ value: format.fromShown(n) });
      },
    },
  );
  return (
    <div className="grid grid-cols-[1fr_1fr_1.3fr_auto] items-center gap-1">
      <InputGroup className="h-7">
        <InputGroupInput className="text-right tabular-nums" aria-label="At" {...at} />
        <InputGroupAddon align="inline-end">{time.unit}</InputGroupAddon>
      </InputGroup>
      <InputGroup className="h-7">
        <InputGroupInput className="text-right tabular-nums" aria-label="Value" {...amount} />
        {format.unit ? <InputGroupAddon align="inline-end">{format.unit}</InputGroupAddon> : null}
      </InputGroup>
      <Select
        items={EASE_ITEMS}
        value={keyframe.ease}
        onValueChange={(v) => {
          const ease = easeOf(v);
          if (ease) onChange({ ease });
        }}
      >
        <SelectTrigger size="sm" className="h-7 text-xs" aria-label="Easing">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            {EASE_ITEMS.map((e) => (
              <SelectItem key={e.value} value={e.value}>
                {e.label}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>
      <Button variant="ghost" size="icon-xs" aria-label="Remove keyframe" onClick={onRemove}>
        <X />
      </Button>
    </div>
  );
}
