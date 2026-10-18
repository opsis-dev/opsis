import { useRef, useState, type PointerEvent } from "react";
import { capturePrimaryPointer, fractionIn } from "@/renderer/lib/dom";
import { clamp, round3 } from "@/shared/math";
import type { Bounds } from "@/shared/schema/actions";

type DragTarget = { mode: "new" } | { mode: "move"; index: number };
type Drag = DragTarget & { x: number; y: number; origin: Bounds; box: Bounds };

const MIN_SIDE = 0.01;

const boxFrom = (x0: number, y0: number, x1: number, y1: number): Bounds => ({
  x: round3(Math.min(x0, x1)),
  y: round3(Math.min(y0, y1)),
  width: round3(Math.abs(x1 - x0)),
  height: round3(Math.abs(y1 - y0)),
});

const moved = (d: Drag, x: number, y: number): Bounds => ({
  ...d.origin,
  x: round3(clamp(d.origin.x + x - d.x, 0, 1 - d.origin.width)),
  y: round3(clamp(d.origin.y + y - d.y, 0, 1 - d.origin.height)),
});

export function useBoxDrag(boxes: Bounds[], onChange: (boxes: Bounds[]) => void) {
  const area = useRef<HTMLDivElement>(null);
  const drag = useRef<Drag | null>(null);
  const [draft, setDraft] = useState<Drag | null>(null);

  const pointIn = (e: PointerEvent) => {
    const r = area.current?.getBoundingClientRect();
    return r ? fractionIn(r, e.clientX, e.clientY) : null;
  };
  const setDrag = (d: Drag | null) => {
    drag.current = d;
    setDraft(d);
  };
  const start = (e: PointerEvent, target: DragTarget) => {
    const at = pointIn(e);
    if (!at || !capturePrimaryPointer(e)) return;
    const origin = target.mode === "move" ? boxes[target.index] : boxFrom(at.x, at.y, at.x, at.y);
    setDrag({ ...target, x: at.x, y: at.y, origin, box: origin });
  };
  const move = (e: PointerEvent) => {
    const d = drag.current;
    const at = pointIn(e);
    if (!d || !at) return;
    setDrag({
      ...d,
      box: d.mode === "new" ? boxFrom(d.x, d.y, at.x, at.y) : moved(d, at.x, at.y),
    });
  };
  const end = () => {
    const d = drag.current;
    setDrag(null);
    if (!d) return;
    if (d.mode === "move") onChange(boxes.map((b, i) => (i === d.index ? d.box : b)));
    else if (d.box.width > MIN_SIDE && d.box.height > MIN_SIDE) onChange([...boxes, d.box]);
  };
  const shown = boxes.map((b, i) => (draft?.mode === "move" && draft.index === i ? draft.box : b));
  return { area, draft, shown, start, move, end };
}
