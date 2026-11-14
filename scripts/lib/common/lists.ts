export function findById<T extends { id: string }>(
  items: readonly T[] | undefined,
  id: string | null | undefined,
): T | undefined {
  return items?.find((item) => item.id === id);
}
