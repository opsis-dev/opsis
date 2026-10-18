export const keepLast = <T>(items: readonly T[], count: number): T[] =>
  count > 0 ? items.slice(-count) : [];

export const appendCapped = <T>(items: readonly T[], item: T, max: number): T[] => [
  ...keepLast(items, max - 1),
  item,
];

export const withCurrent = (options: readonly string[], current: string): string[] => [
  ...new Set(current ? [...options, current] : options),
];

export function uniqueKeys(keys: readonly string[]): string[] {
  const seen = new Map<string, number>();
  return keys.map((key) => {
    const count = (seen.get(key) ?? 0) + 1;
    seen.set(key, count);
    return count === 1 ? key : `${key}#${count}`;
  });
}

export function byCodeUnit(a: string, b: string): number {
  if (a < b) return -1;
  return a > b ? 1 : 0;
}

export function uniqueBy<T>(items: readonly T[], key: (item: T) => string): T[] {
  const seen = new Map<string, T>();
  for (const item of items) if (!seen.has(key(item))) seen.set(key(item), item);
  return [...seen.values()];
}

export function firstFree(
  candidate: (n: number) => string,
  taken: (name: string) => boolean,
  start = 1,
): string {
  for (let n = start; ; n++) {
    const name = candidate(n);
    if (!taken(name)) return name;
  }
}

export const findById = <T extends { id: string }>(
  items: readonly T[] | undefined,
  id: string | null | undefined,
): T | undefined => items?.find((item) => item.id === id);

export const isNonEmpty = <T>(list: readonly T[]): list is readonly [T, ...T[]] => list.length > 0;
