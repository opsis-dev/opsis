export type DragPhase = "start" | "move" | "end";

export type Change<T> = (value: T, phase: DragPhase) => void;

export function commitOnce<T>(onChange: Change<T>, value: T): void {
  onChange(value, "start");
  onChange(value, "move");
  onChange(value, "end");
}

function arrowDirection(key: string): -1 | 0 | 1 {
  if (key === "ArrowUp" || key === "ArrowRight") return 1;
  if (key === "ArrowDown" || key === "ArrowLeft") return -1;
  return 0;
}

export function onArrowKey(e: React.KeyboardEvent, step: (dir: -1 | 1, big: boolean) => void) {
  const dir = arrowDirection(e.key);
  if (!dir) return;
  e.preventDefault();
  e.stopPropagation();
  step(dir, e.shiftKey);
}
