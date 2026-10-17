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
import { useItemKeys } from "@/renderer/lib/use-item-keys";
import { parseNumberText } from "@/shared/format";
import { round2 } from "@/shared/math";
import { ChoiceSelect } from "./choice-select";
import { trimNumber, useDecimalInput } from "./decimal";
import { Help } from "./field-shell";
import type { FieldProps } from "./types";

type EnvelopeKey = VisualEnvelopeKey | AudioEnvelopeKey;
type Format = {
  unit: string;
  toShown: (v: number) => number;
  fromShown: (v: number) => number;
  initial: number;
};

const envelopeSchema = z.record(z.string(), z.array(keyframeSchema).optional());
type Envelope = z.infer<typeof envelopeSchema>;

export const isEnvelope = (value: unknown): value is Envelope =>
  envelopeSchema.safeParse(value).success;

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

export function EnvelopeField({ label, value: env, onChange, meta, ctx }: FieldProps<Envelope>) {
  const tracks = Object.entries(meta.labels ?? {}).flatMap(([prop, text]) =>
    isEnvelopeKey(prop) && text ? [{ prop, text }] : [],
  );
  const write = (prop: EnvelopeKey, keys: Keyframe[]) => {
    const next: Envelope = { ...env };
    if (keys.length) next[prop] = keys;
    else delete next[prop];
    onChange(next);
  };
  return (
    <FieldSet>
      <FieldLegend variant="label">{label}</FieldLegend>
      <Help meta={meta} />
      {tracks.map(({ prop, text }) => (
        <EnvelopeTrack
          key={prop}
          label={text}
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
  const rowKeys = useItemKeys(keys.length);
  const indices = keys.map((_, i) => i);
  const commit = (next: Keyframe[], origin: number[]) => {
    const order = next.map((_, j) => j).sort((a, b) => next[a].frame - next[b].frame);
    rowKeys.reorder(order.map((j) => origin[j]));
    onChange(order.map((j) => next[j]));
  };
  const add = () => {
    const last = keys.at(-1);
    const added: Keyframe = {
      frame: last ? last.frame + fps : 0,
      value: last ? last.value : format.initial,
      ease: "linear",
    };
    commit([...keys, added], [...indices, -1]);
  };
  const updateKey = (i: number, patch: Partial<Keyframe>) =>
    commit(
      keys.map((k, j) => (j === i ? { ...k, ...patch } : k)),
      indices,
    );
  const removeKey = (i: number) =>
    commit(
      keys.filter((_, j) => j !== i),
      indices.filter((j) => j !== i),
    );
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
          key={rowKeys.keys[i] ?? i}
          keyframe={k}
          format={format}
          fps={fps}
          onChange={(patch) => updateKey(i, patch)}
          onRemove={() => removeKey(i)}
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
  const at = useDecimalInput(trimNumber(keyframe.frame / fps), "blur", (text) => {
    const n = parseNumberText(text);
    if (n !== null) onChange({ frame: Math.max(0, Math.round(n * fps)) });
  });
  const amount = useDecimalInput(
    trimNumber(round2(format.toShown(keyframe.value))),
    "blur",
    (text) => {
      const n = parseNumberText(text);
      if (n !== null) onChange({ value: format.fromShown(n) });
    },
  );
  return (
    <div className="grid grid-cols-[1fr_1fr_1.3fr_auto] items-center gap-1">
      <InputGroup className="h-7">
        <InputGroupInput className="text-right tabular-nums" aria-label="At" {...at} />
        <InputGroupAddon align="inline-end">s</InputGroupAddon>
      </InputGroup>
      <InputGroup className="h-7">
        <InputGroupInput className="text-right tabular-nums" aria-label="Value" {...amount} />
        {format.unit ? <InputGroupAddon align="inline-end">{format.unit}</InputGroupAddon> : null}
      </InputGroup>
      <EaseSelect ease={keyframe.ease} onChange={(ease) => onChange({ ease })} />
      <Button variant="ghost" size="icon-xs" aria-label="Remove keyframe" onClick={onRemove}>
        <X />
      </Button>
    </div>
  );
}

function EaseSelect({ ease, onChange }: { ease: Ease; onChange: (ease: Ease) => void }) {
  return (
    <ChoiceSelect
      groups={[EASE_ITEMS]}
      value={ease}
      onChange={(v) => {
        const picked = easeOf(v);
        if (picked) onChange(picked);
      }}
      size="sm"
      className="h-7 text-xs"
      aria-label="Easing"
    />
  );
}
