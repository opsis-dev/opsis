export type Gesture<T> = { begin: () => void; set: (value: T) => void; end: () => void };

export function commitOnce<T>(gesture: Gesture<T>, value: T): void {
  gesture.begin();
  gesture.set(value);
  gesture.end();
}

export const mapGesture = <A, B>(gesture: Gesture<B>, map: (value: A) => B): Gesture<A> => ({
  begin: gesture.begin,
  set: (value) => gesture.set(map(value)),
  end: gesture.end,
});

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
