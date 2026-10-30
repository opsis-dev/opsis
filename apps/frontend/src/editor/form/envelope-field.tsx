import { Plus, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { FieldLegend, FieldSet } from "@/components/ui/field";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group";
import type { Ease, EnvelopeKey, Keyframe } from "@/generated/bindings";
import { parseNumberText } from "@/lib/number-text";
import { whenOneOf } from "@/lib/one-of";
import { useItemKeys } from "@/lib/use-item-keys";
import { timelineBook } from "@/store/catalog";
import { workflowEdits } from "@/store/workflow";
import { envelopeTrack, type Envelope } from "@/timeline/envelopes";
import { ChoiceSelect } from "./choice-select";
import { framesScale, parseShown, shownText, trimNumber, useDecimalInput } from "./decimal";
import { Help } from "./field-shell";
import type { FieldProps } from "./types";

type Format = {
  unit: string;
  toShown: (v: number) => number;
  fromShown: (v: number) => number;
};

function fromFraction(v: number) {
  return v * 100;
}
function toFraction(v: number) {
  return v / 100;
}
const FRACTION_AS_PERCENT = { unit: "%", toShown: fromFraction, fromShown: toFraction };

const PLAIN: Format = { unit: "×", toShown: (v) => v, fromShown: (v) => v };

const ENVELOPE_FORMAT: Record<EnvelopeKey, Format> = {
  opacity: FRACTION_AS_PERCENT,
  x: FRACTION_AS_PERCENT,
  y: FRACTION_AS_PERCENT,
  scale: PLAIN,
  volume: FRACTION_AS_PERCENT,
  pan: { ...FRACTION_AS_PERCENT, unit: "" },
};

type KeyPatch = Pick<Partial<Keyframe>, "frame" | "value" | "ease">;

export function EnvelopeField({
  label,
  value: env,
  meta,
  ctx,
  tracks,
}: FieldProps<Envelope> & { tracks: EnvelopeKey[] }) {
  return (
    <FieldSet>
      <FieldLegend variant="label">{label}</FieldLegend>
      <Help meta={meta} />
      {tracks.map((track) => (
        <EnvelopeTrack
          key={track}
          label={envelopeTrack(track).label}
          keys={env[track] ?? []}
          format={ENVELOPE_FORMAT[track]}
          fps={ctx.fps}
          onAdd={() => workflowEdits.keyframes({ op: "appendKey", nodeId: ctx.nodeId, track })}
          onSet={(index, patch) =>
            workflowEdits.keyframes({ op: "setKey", nodeId: ctx.nodeId, track, index, ...patch })
          }
          onRemove={(index) =>
            workflowEdits.keyframes({ op: "removeKey", nodeId: ctx.nodeId, track, index })
          }
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
  onAdd,
  onSet,
  onRemove,
}: {
  label: string;
  keys: Keyframe[];
  format: Format;
  fps: number;
  onAdd: () => void;
  onSet: (index: number, patch: KeyPatch) => void;
  onRemove: (index: number) => void;
}) {
  const list = useItemKeys(keys);
  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium">{label}</span>
        <Button variant="ghost" size="xs" onClick={onAdd}>
          <Plus data-icon="inline-start" />
          Add
        </Button>
      </div>
      {list.rows.map(({ item: k, key }, i) => (
        <KeyframeRow
          key={key}
          keyframe={k}
          format={format}
          fps={fps}
          onChange={(patch) => onSet(i, patch)}
          onRemove={() => onRemove(i)}
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
  onChange: (patch: KeyPatch) => void;
  onRemove: () => void;
}) {
  const frames = framesScale(fps);
  const at = useDecimalInput(shownText(keyframe.frame, frames), "blur", (text) => {
    const frame = parseShown(text, frames);
    if (frame !== null) onChange({ frame });
  });
  const amount = useDecimalInput(trimNumber(format.toShown(keyframe.value)), "blur", (text) => {
    const n = parseNumberText(text);
    if (n !== null) onChange({ value: format.fromShown(n) });
  });
  return (
    <div className="grid grid-cols-[1fr_1fr_1.3fr_auto] items-center gap-1">
      <InputGroup className="h-7">
        <InputGroupInput className="text-right tabular-nums" aria-label="At" {...at} />
        <InputGroupAddon align="inline-end">{frames.unit}</InputGroupAddon>
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

export function EaseSelect({ ease, onChange }: { ease: Ease; onChange: (ease: Ease) => void }) {
  const { eases } = timelineBook();
  return (
    <ChoiceSelect
      groups={[eases.map(([value, label]) => ({ value, label }))]}
      value={ease}
      onChange={whenOneOf(
        eases.map(([value]) => value),
        onChange,
      )}
      size="sm"
      className="h-7 text-xs"
      aria-label="Easing"
    />
  );
}
