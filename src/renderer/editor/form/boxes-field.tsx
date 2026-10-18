import { X } from "lucide-react";
import { FieldDescription } from "@/renderer/components/ui/field";
import { cn } from "@/renderer/lib/utils";
import { percent } from "@/shared/format";
import type { Bounds } from "@/shared/schema/actions";
import { FieldShell } from "./field-shell";
import { SpotFrame } from "./spot-frame";
import type { FieldProps } from "./types";
import { boxHandle, useBoxDrag } from "./use-box-drag";

const MAX_BOXES = 8;

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

export function BoxesField({ label, value: boxes, onChange, ctx, meta }: FieldProps<Bounds[]>) {
  const full = boxes.length >= MAX_BOXES;
  const { area, draft, shown, frame } = useBoxDrag(boxes, onChange, !full);

  return (
    <FieldShell label={label} meta={meta}>
      <SpotFrame
        ref={area}
        ctx={ctx}
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
      <FieldDescription>
        {full
          ? `Up to ${MAX_BOXES} areas. Remove one to add another.`
          : `${boxes.length} of ${MAX_BOXES} areas.`}
      </FieldDescription>
    </FieldShell>
  );
}
