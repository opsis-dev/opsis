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
