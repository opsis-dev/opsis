import { formatDuration, percent } from "@opsis/core/format";
import { numberOr } from "@opsis/core/math";
import type { Bounds } from "@opsis/core/schema/actions";
import { X } from "lucide-react";
import { FieldDescription } from "@/components/ui/field";
import { cn } from "@/lib/utils";
import { SingleSlider } from "../../app/single-slider";
import { useWorkflowStore } from "../../store/workflow";
import { FieldShell } from "./field-shell";
import { SpotFrame } from "./spot-frame";
import type { FormContext, ParamFieldProps } from "./types";
import { boxHandle, useBoxDrag } from "./use-box-drag";

const MAX_BOXES = 8;
const FRAME_STEP_MS = 100;

const boxStyle = (b: Bounds) => ({
  left: percent(b.x),
  top: percent(b.y),
  width: percent(b.width),
  height: percent(b.height),
});

function BoxOverlay({
  box,
  number,
  onRemove,
}: {
  box: Bounds;
  number: number;
  onRemove: () => void;
}) {
  return (
    <div
      className="border-primary bg-primary/25 absolute cursor-move border-2"
      style={boxStyle(box)}
      {...boxHandle(number - 1)}
    >
      <button
        type="button"
        aria-label={`Remove area ${number}`}
        className="bg-background text-foreground absolute -top-2 -right-2 grid size-4 place-items-center rounded-full border"
        onPointerDown={(e) => e.stopPropagation()}
        onClick={onRemove}
      >
        <X className="size-3" />
      </button>
    </div>
  );
}

function FrameSlider({
  ctx,
  timeMs,
  onChange,
}: {
  ctx: FormContext;
  timeMs: number;
  onChange: (timeMs: number) => void;
}) {
  const store = useWorkflowStore.getState;
  const durationMs = ctx.spotBackground?.durationMs ?? 0;
  if (durationMs <= 0) return null;
  return (
    <div className="flex items-center gap-3">
      <SingleSlider
        aria-label="Frame to draw on"
        min={0}
        max={durationMs}
        step={FRAME_STEP_MS}
        value={Math.min(timeMs, durationMs)}
        onChange={onChange}
        onPointerDown={() => store().beginGesture()}
        onValueCommitted={() => store().endGesture()}
      />
      <span className="text-muted-foreground w-12 shrink-0 text-right text-xs tabular-nums">
        {formatDuration(Math.min(timeMs, durationMs))}
      </span>
    </div>
  );
}

export function BoxesField({
  label,
  value: boxes,
  onChange,
  ctx,
  meta,
  all,
  onPatch,
}: ParamFieldProps<Bounds[]>) {
  const full = boxes.length >= MAX_BOXES;
  const { area, draft, shown, frame } = useBoxDrag(boxes, onChange, !full);
  const { timeKey } = meta;
  const timeMs = timeKey ? numberOr(all[timeKey], 0) : undefined;

  return (
    <FieldShell label={label} meta={meta}>
      <SpotFrame
        ref={area}
        ctx={ctx}
        timeMs={timeMs}
        className={cn("touch-none select-none", full ? "cursor-default" : "cursor-crosshair")}
        {...frame}
      >
        {shown.map((b, i) => (
          <BoxOverlay
            key={i}
            box={b}
            number={i + 1}
            onRemove={() => onChange(boxes.filter((_, j) => j !== i))}
          />
        ))}
        {draft?.mode === "new" ? (
          <div
            className="border-primary bg-primary/25 pointer-events-none absolute border-2 border-dashed"
            style={boxStyle(draft.box)}
          />
        ) : null}
      </SpotFrame>
      {timeKey && timeMs !== undefined ? (
        <FrameSlider ctx={ctx} timeMs={timeMs} onChange={(next) => onPatch({ [timeKey]: next })} />
      ) : null}
      <FieldDescription>
        {full
          ? `Up to ${MAX_BOXES} areas. Remove one to add another.`
          : `${boxes.length} of ${MAX_BOXES} areas.`}
      </FieldDescription>
    </FieldShell>
  );
}
