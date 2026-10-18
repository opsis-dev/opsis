import { useRef, type PointerEvent } from "react";
import { capturePrimaryPointer } from "./dom";

type PointerDrag<E extends Element, S> = {
  disabled?: boolean;
  start: (e: PointerEvent<E>) => S;
  move: (e: PointerEvent<E>, drag: S) => void;
  end?: (drag: S) => void;
};

export function usePointerDrag<E extends Element, S>({
  disabled = false,
  start,
  move,
  end,
}: PointerDrag<E, S>) {
  const drag = useRef<{ state: S } | null>(null);
  return {
    onPointerDown: (e: PointerEvent<E>) => {
      if (disabled || !capturePrimaryPointer(e)) return;
      drag.current = { state: start(e) };
    },
    onPointerMove: (e: PointerEvent<E>) => {
      if (drag.current && e.currentTarget.hasPointerCapture(e.pointerId))
        move(e, drag.current.state);
    },
    onLostPointerCapture: () => {
      const active = drag.current;
      drag.current = null;
      if (active) end?.(active.state);
    },
  };
}
