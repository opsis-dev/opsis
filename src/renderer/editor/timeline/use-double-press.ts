import { useRef } from "react";

const DOUBLE_PRESS_MS = 400;
const DOUBLE_PRESS_PX = 5;

const NO_PRESS = { at: 0, x: 0, y: 0 };

export function useDoublePress(): (e: React.PointerEvent) => boolean {
  const last = useRef(NO_PRESS);
  return (e) => {
    const now = performance.now();
    const prev = last.current;
    const hit =
      now - prev.at < DOUBLE_PRESS_MS &&
      Math.abs(e.clientX - prev.x) < DOUBLE_PRESS_PX &&
      Math.abs(e.clientY - prev.y) < DOUBLE_PRESS_PX;
    last.current = hit ? NO_PRESS : { at: now, x: e.clientX, y: e.clientY };
    return hit;
  };
}
