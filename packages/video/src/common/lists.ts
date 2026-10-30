export function keepLast<T>(items: readonly T[], count: number): T[] {
  return count > 0 ? items.slice(-count) : [];
}

export function appendCapped<T>(items: readonly T[], item: T, max: number): T[] {
  return [...keepLast(items, max - 1), item];
}

export function withCurrent(options: readonly string[], current: string): string[] {
  return [...new Set(current ? [...options, current] : options)];
}

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
  const seen = new Set<string>();
  return items.filter((item) => {
    const k = key(item);
    if (seen.has(k)) return false;
    seen.add(k);
    return true;
  });
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

export function findById<T extends { id: string }>(
  items: readonly T[] | undefined,
  id: string | null | undefined,
): T | undefined {
  return items?.find((item) => item.id === id);
}

export function isNonEmpty<T>(list: readonly T[]): list is readonly [T, ...T[]] {
  return list.length > 0;
}
