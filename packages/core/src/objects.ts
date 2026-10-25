export const pick = <T, K extends keyof T>(source: T, keys: readonly K[]): Pick<T, K> =>
  Object.fromEntries(keys.map((k) => [k, source[k]])) as Pick<T, K>;

export const mapValues = <K extends string, V, W>(
  source: Readonly<Record<K, V>>,
  fn: (value: V) => W,
): Record<K, W> =>
  Object.fromEntries(Object.entries<V>(source).map(([k, v]) => [k, fn(v)])) as Record<K, W>;
