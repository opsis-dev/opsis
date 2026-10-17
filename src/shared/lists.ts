export const keepLast = <T>(items: readonly T[], count: number): T[] =>
  count > 0 ? items.slice(-count) : [];

export const appendCapped = <T>(items: readonly T[], item: T, max: number): T[] => [
  ...keepLast(items, max - 1),
  item,
];

export const byCodeUnit = (a: string, b: string): number => (a < b ? -1 : a > b ? 1 : 0);
