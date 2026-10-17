export function randomId(prefix?: string): string {
  const id = Math.random().toString(36).slice(2, 10).padEnd(8, "0");
  return prefix ? `${prefix}_${id}` : id;
}

export const nodeIdPrefix = (type: string): string => type.split(".")[0];

export const nodeIdFor = (type: string): string => randomId(nodeIdPrefix(type));

type EdgeEnds = { source: string; sourceHandle: string; target: string; targetHandle: string };

export const edgeId = (e: EdgeEnds): string =>
  `${e.source}.${e.sourceHandle}->${e.target}.${e.targetHandle}`;
