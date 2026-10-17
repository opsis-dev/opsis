import { X } from "lucide-react";
import type { PointerEvent } from "react";
import { z } from "zod";
import { percent } from "@/shared/format";
import type { Bounds } from "@/shared/schema/actions";
import { Field, FieldDescription, FieldLabel } from "@/renderer/components/ui/field";
import { cn } from "@/renderer/lib/utils";
import { SpotFrame } from "./spot-frame";
import type { FieldProps } from "./types";
import { useBoxDrag } from "./use-box-drag";

const boxSchema = z.object({ x: z.number(), y: z.number(), width: z.number(), height: z.number() });

export const isBoxList = (value: unknown): value is Bounds[] =>
  z.array(boxSchema).safeParse(value).success;

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
  onGrab,
  onRemove,
}: {
  box: Bounds;
  number: number;
  onGrab: (e: PointerEvent) => void;
  onRemove: () => void;
}) {
  return (
    <div
      className="border-primary bg-primary/25 absolute cursor-move border-2"
      style={boxStyle(box)}
      onPointerDown={(e) => {
        e.stopPropagation();
        onGrab(e);
      }}
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
  const { area, draft, shown, start, move, end } = useBoxDrag(boxes, onChange);
  const full = boxes.length >= MAX_BOXES;

  return (
    <Field>
      <FieldLabel>{label}</FieldLabel>
      <SpotFrame
        ref={area}
        ctx={ctx}
        className={cn("touch-none select-none", full ? "cursor-default" : "cursor-crosshair")}
        onPointerDown={(e) => !full && start(e, { mode: "new" })}
        onPointerMove={move}
        onPointerUp={end}
        onPointerCancel={end}
      >
        {shown.map((b, i) => (
          <BoxOverlay
            key={i}
            box={b}
            number={i + 1}
            onGrab={(e) => start(e, { mode: "move", index: i })}
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
          : `${boxes.length} of ${MAX_BOXES} areas.`}{" "}
        {meta.help}
      </FieldDescription>
    </Field>
  );
}
