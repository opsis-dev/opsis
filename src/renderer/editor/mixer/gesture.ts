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
