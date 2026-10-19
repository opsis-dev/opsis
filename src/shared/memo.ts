import type { z } from "zod";

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

export function lruCache<T extends NonNullable<unknown>>(limit: number) {
  const items = new Map<string, T>();
  return {
    peek: (key: string): T | undefined => items.get(key),
    delete: (key: string): void => void items.delete(key),
    use(key: string, make: () => T): T {
      const value = items.get(key) ?? make();
      items.delete(key);
      items.set(key, value);
      const [oldest] = items.keys();
      if (items.size > limit) items.delete(oldest);
      return value;
    },
  };
}

export function parseOncePer<T>(schema: z.ZodType<T>): (value: object) => T | null {
  const results = new WeakMap<object, T | null>();
  return (value) => {
    if (!results.has(value)) {
      const parsed = schema.safeParse(value);
      results.set(value, parsed.success ? parsed.data : null);
    }
    return results.get(value) ?? null;
  };
}
