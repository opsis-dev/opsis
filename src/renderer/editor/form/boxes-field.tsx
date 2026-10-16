import { X } from "lucide-react";
import { useMemo } from "react";
import { Field, FieldDescription, FieldLabel } from "@/renderer/components/ui/field";
import { cn } from "@/renderer/lib/utils";
import type { FieldProps } from "./types";
import { boxListSchema, useBoxDrag, type Box } from "./use-box-drag";

const MAX_BOXES = 8;

const boxStyle = (b: Box) => ({
  left: `${b.x * 100}%`,
  top: `${b.y * 100}%`,
  width: `${b.width * 100}%`,
  height: `${b.height * 100}%`,
});

export function BoxesField({ label, value, onChange, ctx, meta }: FieldProps<unknown>) {
  const boxes = boxListSchema.parse(value);
  const { area, draft, shown, start, move, end } = useBoxDrag(boxes, onChange);
  const bg = ctx.spotBackground;
  const src = useMemo(() => (bg ? `${bg.src}#t=${bg.timeMs / 1000}` : null), [bg]);
  const full = boxes.length >= MAX_BOXES;

  return (
    <Field>
      <FieldLabel>{label}</FieldLabel>
      <div
        ref={area}
        className={cn(
          "bg-muted relative w-full touch-none overflow-hidden rounded-md border select-none",
          full ? "cursor-default" : "cursor-crosshair",
        )}
        style={{ aspectRatio: String(ctx.aspect) }}
        onPointerDown={(e) => !full && start(e, "new", boxes.length)}
        onPointerMove={move}
        onPointerUp={end}
        onPointerCancel={end}
      >
        {src ? (
          <video
            src={src}
            muted
            preload="auto"
            className="pointer-events-none absolute inset-0 size-full object-cover"
          />
        ) : null}
        {shown.map((b, i) => (
          <div
            key={i}
            className="border-primary bg-primary/25 absolute cursor-move border-2"
            style={boxStyle(b)}
            onPointerDown={(e) => {
              e.stopPropagation();
              start(e, "move", i);
            }}
          >
            <button
              type="button"
              aria-label={`Remove area ${i + 1}`}
              className="bg-background text-foreground absolute -top-2 -right-2 grid size-4 place-items-center rounded-full border"
              onPointerDown={(e) => e.stopPropagation()}
              onClick={() => onChange(boxes.filter((_, j) => j !== i))}
            >
              <X className="size-3" />
            </button>
          </div>
        ))}
        {draft?.mode === "new" ? (
          <div
            className="border-primary bg-primary/25 pointer-events-none absolute border-2 border-dashed"
            style={boxStyle(draft.box)}
          />
        ) : null}
      </div>
      <FieldDescription>
        {full
          ? `Up to ${MAX_BOXES} areas. Remove one to add another.`
          : `${boxes.length} of ${MAX_BOXES} areas.`}{" "}
        {meta.help}
      </FieldDescription>
    </Field>
  );
}
