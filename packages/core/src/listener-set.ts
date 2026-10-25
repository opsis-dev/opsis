export function listenerSet<A extends unknown[] = []>() {
  const listeners = new Set<(...args: A) => void>();
  return {
    add(listener: (...args: A) => void): () => void {
      listeners.add(listener);
      return () => void listeners.delete(listener);
    },
    emit(...args: A): void {
      for (const listener of listeners) listener(...args);
    },
  };
}
