export const keepLast = <T>(items: readonly T[], count: number): T[] =>
  count > 0 ? items.slice(-count) : [];

export const appendCapped = <T>(items: readonly T[], item: T, max: number): T[] => [
  ...keepLast(items, max - 1),
  item,
];

export const byCodeUnit = (a: string, b: string): number => (a < b ? -1 : a > b ? 1 : 0);

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
