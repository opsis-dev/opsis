export type Gesture<T> = { begin: () => void; set: (value: T) => void; end: () => void };

export function commitOnce<T>(gesture: Gesture<T>, value: T): void {
  gesture.begin();
  gesture.set(value);
  gesture.end();
}

export function mapGesture<A, B>(gesture: Gesture<B>, map: (value: A) => B): Gesture<A> {
  return {
    begin: gesture.begin,
    set: (value) => gesture.set(map(value)),
    end: gesture.end,
  };
}
