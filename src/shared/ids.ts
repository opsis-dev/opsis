export function randomId(prefix?: string): string {
  const id = Math.random().toString(36).slice(2, 10).padEnd(8, "0");
  return prefix ? `${prefix}_${id}` : id;
}

export const nodeIdPrefix = (type: string): string => type.split(".")[0];

export const nodeIdFor = (type: string): string => randomId(nodeIdPrefix(type));
