export function randomId(prefix?: string): string {
  const id = Math.random().toString(36).slice(2, 10).padEnd(8, "0");
  return prefix ? `${prefix}_${id}` : id;
}
