export function keepLatest<T>(
  dispose?: (value: T) => unknown,
): (key: string, make: () => Promise<T>) => Promise<T> {
  let held: { key: string; value: Promise<T> } | null = null;
  return (key, make) => {
    if (held?.key === key) return held.value;
    const previous = held?.value;
    const value = make().catch((error: unknown) => {
      if (held?.value === value) held = null;
      throw error;
    });
    held = { key, value };
    if (dispose && previous) void previous.then(dispose, () => undefined);
    return value;
  };
}

export function shareInFlight<T>(): (key: string, make: () => Promise<T>) => Promise<T> {
  const running = new Map<string, Promise<T>>();
  return (key, make) => {
    const current = running.get(key);
    if (current) return current;
    const value = make().finally(() => running.delete(key));
    running.set(key, value);
    return value;
  };
}
