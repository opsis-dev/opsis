import { useRef, useState } from "react";
import { z } from "zod";
import { clamp, clamp01, round3 } from "@/shared/math";

const boxSchema = z.object({ x: z.number(), y: z.number(), width: z.number(), height: z.number() });
export const boxListSchema = z.array(boxSchema).catch([]);
export type Box = z.infer<typeof boxSchema>;

type DragMode = "new" | "move";
type Drag = { mode: DragMode; index: number; x: number; y: number; origin: Box };
export type BoxDraft = { mode: DragMode; index: number; box: Box };

const MIN_SIDE = 0.01;

const boxFrom = (x0: number, y0: number, x1: number, y1: number): Box => ({
  x: round3(Math.min(x0, x1)),
  y: round3(Math.min(y0, y1)),
  width: round3(Math.abs(x1 - x0)),
  height: round3(Math.abs(y1 - y0)),
});

const moved = (d: Drag, x: number, y: number): Box => ({
  ...d.origin,
  x: round3(clamp(d.origin.x + x - d.x, 0, 1 - d.origin.width)),
  y: round3(clamp(d.origin.y + y - d.y, 0, 1 - d.origin.height)),
});

export function useBoxDrag(boxes: Box[], onChange: (boxes: Box[]) => void) {
  const area = useRef<HTMLDivElement>(null);
  const drag = useRef<Drag | null>(null);
  const [draft, setDraft] = useState<BoxDraft | null>(null);

  const point = (e: React.PointerEvent): [number, number] => {
    const r = area.current?.getBoundingClientRect();
    if (!r) return [0, 0];
    return [clamp01((e.clientX - r.left) / r.width), clamp01((e.clientY - r.top) / r.height)];
  };
  const start = (e: React.PointerEvent, mode: DragMode, index: number) => {
    const [x, y] = point(e);
    e.currentTarget.setPointerCapture(e.pointerId);
    const origin = mode === "move" ? boxes[index] : boxFrom(x, y, x, y);
    drag.current = { mode, index, x, y, origin };
    setDraft({ mode, index, box: origin });
  };
  const move = (e: React.PointerEvent) => {
    const d = drag.current;
    if (!d) return;
    const [x, y] = point(e);
    const box = d.mode === "new" ? boxFrom(d.x, d.y, x, y) : moved(d, x, y);
    setDraft({ mode: d.mode, index: d.index, box });
  };
  const end = () => {
    const d = drag.current;
    drag.current = null;
    setDraft(null);
    if (!d || !draft) return;
    const box = draft.box;
    if (d.mode === "move") onChange(boxes.map((b, i) => (i === d.index ? box : b)));
    else if (box.width > MIN_SIDE && box.height > MIN_SIDE) onChange([...boxes, box]);
  };
  const shown = boxes.map((b, i) => (draft?.mode === "move" && draft.index === i ? draft.box : b));
  return { area, draft, shown, start, move, end };
}
