import { useRef, type PointerEvent } from "react";
import { capturePrimaryPointer } from "./dom";

type PointerSession<E extends Element, S> = {
  move: (e: PointerEvent<E>, drag: S) => void;
  end?: (drag: S) => void;
};

export function usePointerSession<E extends Element, S>({ move, end }: PointerSession<E, S>) {
  const drag = useRef<{ state: S } | null>(null);
  return {
    track: (state: S) => {
      drag.current = { state };
    },
    isActive: () => drag.current !== null,
    handlers: {
      onPointerMove: (e: PointerEvent<E>) => {
        if (drag.current && e.currentTarget.hasPointerCapture(e.pointerId))
          move(e, drag.current.state);
      },
      onLostPointerCapture: () => {
        const active = drag.current;
        drag.current = null;
        if (active) end?.(active.state);
      },
    },
  };
}

type PointerDrag<E extends Element, S> = PointerSession<E, S> & {
  disabled?: boolean;
  start: (e: PointerEvent<E>) => S;
};

export function usePointerDrag<E extends Element, S>({
  disabled = false,
  start,
  move,
  end,
}: PointerDrag<E, S>) {
  const session = usePointerSession<E, S>({ move, end });
  return {
    onPointerDown: (e: PointerEvent<E>) => {
      if (disabled || !capturePrimaryPointer(e)) return;
      session.track(start(e));
    },
    ...session.handlers,
  };
}
