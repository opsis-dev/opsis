export const pick = <T, K extends keyof T>(source: T, keys: readonly K[]): Pick<T, K> =>
  Object.fromEntries(keys.map((k) => [k, source[k]])) as Pick<T, K>;
