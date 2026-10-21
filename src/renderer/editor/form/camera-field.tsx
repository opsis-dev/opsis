import { Plus, X } from "lucide-react";
import { Button } from "@/renderer/components/ui/button";
import { FieldLegend, FieldSet } from "@/renderer/components/ui/field";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/renderer/components/ui/input-group";
import { useItemKeys } from "@/renderer/lib/use-item-keys";
import type { CameraKey, CameraPath } from "@/shared/schema/timeline";
import { cameraKeyAt } from "@/shared/timeline/camera-path";
import { SingleSlider } from "../../app/single-slider";
import { usePlayback } from "../../store/playback";
import { useWorkflowStore } from "../../store/workflow";
import { useFilm } from "../film";
import { framesScale, parseShown, shownText, useDecimalInput } from "./decimal";
import { EaseSelect } from "./envelope-field";
import { Help } from "./field-shell";
import type { FieldProps } from "./types";

type Dial = {
  key: "yaw" | "pitch" | "distance" | "height";
  label: string;
  min: number;
  max: number;
  step: number;
  unit: string;
};

const DIALS: Dial[] = [
  { key: "yaw", label: "Turn", min: -180, max: 180, step: 1, unit: "°" },
  { key: "pitch", label: "Tilt", min: -89, max: 89, step: 1, unit: "°" },
  { key: "distance", label: "Distance", min: 0.3, max: 4, step: 0.05, unit: "×" },
  { key: "height", label: "Height", min: -2, max: 2, step: 0.05, unit: "" },
];

const byFrame = (a: CameraKey, b: CameraKey) => a.frame - b.frame;

function usePlayheadInNode(nodeId: string): number | null {
  const { flat } = useFilm();
  const frame = usePlayback((s) => s.frame);
  const link = flat?.bars.flatMap((b) => b.chain).find((l) => l.nodeId === nodeId);
  if (!link) return null;
  return Math.max(0, Math.round((frame - link.from) * link.rate));
}

function DialSlider({
  dial,
  value,
  onChange,
}: {
  dial: Dial;
  value: number;
  onChange: (value: number) => void;
}) {
  const store = useWorkflowStore.getState;
  return (
    <div className="grid grid-cols-[4.5rem_1fr_3.5rem] items-center gap-2">
      <span className="text-muted-foreground text-xs">{dial.label}</span>
      <SingleSlider
        aria-label={dial.label}
        min={dial.min}
        max={dial.max}
        step={dial.step}
        value={value}
        onChange={onChange}
        onPointerDown={() => store().beginGesture()}
        onValueCommitted={() => store().endGesture()}
      />
      <span className="text-right text-xs tabular-nums">
        {Number(value.toFixed(2))}
        {dial.unit}
      </span>
    </div>
  );
}

function CameraKeyRow({
  cameraKey,
  fps,
  onChange,
  onRemove,
}: {
  cameraKey: CameraKey;
  fps: number;
  onChange: (patch: Partial<CameraKey>) => void;
  onRemove: () => void;
}) {
  const frames = framesScale(fps);
  const at = useDecimalInput(shownText(cameraKey.frame, frames), "blur", (text) => {
    const frame = parseShown(text, frames);
    if (frame !== null) onChange({ frame: Math.max(0, Math.round(frame)) });
  });
  return (
    <div className="flex flex-col gap-1.5 rounded-md border p-2">
      <div className="grid grid-cols-[1fr_1.3fr_auto] items-center gap-1">
        <InputGroup className="h-7">
          <InputGroupInput className="text-right tabular-nums" aria-label="At" {...at} />
          <InputGroupAddon align="inline-end">{frames.unit}</InputGroupAddon>
        </InputGroup>
        <EaseSelect ease={cameraKey.ease} onChange={(ease) => onChange({ ease })} />
        <Button variant="ghost" size="icon-xs" aria-label="Remove camera key" onClick={onRemove}>
          <X />
        </Button>
      </div>
      {DIALS.map((dial) => (
        <DialSlider
          key={dial.key}
          dial={dial}
          value={cameraKey[dial.key]}
          onChange={(v) => onChange({ [dial.key]: v })}
        />
      ))}
    </div>
  );
}

export function CameraField({ label, value, onChange, meta, ctx }: FieldProps<CameraPath>) {
  const list = useItemKeys(value.keys, byFrame);
  const playhead = usePlayheadInNode(ctx.nodeId);
  const write = (keys: CameraKey[]) => onChange({ keys });
  return (
    <FieldSet>
      <div className="flex items-center justify-between">
        <FieldLegend variant="label">{label}</FieldLegend>
        <Button
          variant="ghost"
          size="xs"
          disabled={playhead === null}
          title={
            playhead === null ? "Put this node in the film to add keys at the playhead." : undefined
          }
          onClick={() => playhead !== null && write(cameraKeyAt(value, playhead).keys)}
        >
          <Plus data-icon="inline-start" />
          Add key at playhead
        </Button>
      </div>
      <Help meta={meta} />
      {list.rows.map(({ item, key }, i) => (
        <CameraKeyRow
          key={key}
          cameraKey={item}
          fps={ctx.fps}
          onChange={(patch) => write(list.update(i, { ...item, ...patch }))}
          onRemove={() => write(list.remove(i))}
        />
      ))}
    </FieldSet>
  );
}
