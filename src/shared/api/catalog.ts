import { z } from "zod";
import { CATEGORY_LABELS, type NodeDef, type PortSpec } from "../nodes/define";
import { getNodeDef, NODE_DEFS } from "../nodes/registry";

export type CatalogPort = {
  type: string;
  label: string;
  optional: boolean;
  many: boolean;
};

export type CatalogNode = {
  type: string;
  title: string;
  category: string;
  description: string;
  runs: boolean;
  inputs: Record<string, CatalogPort>;
  outputs: Record<string, CatalogPort>;
  params: unknown;
};

const ports = (specs: Record<string, PortSpec>): Record<string, CatalogPort> =>
  Object.fromEntries(
    Object.entries(specs).map(([id, p]) => [
      id,
      { type: p.type, label: p.label ?? id, optional: !!p.optional, many: !!p.many },
    ]),
  );

export function paramsJsonSchema(def: NodeDef): unknown {
  return z.toJSONSchema(def.params, { io: "input", unrepresentable: "any" });
}

export function catalogNode(def: NodeDef, withParams = true): CatalogNode {
  return {
    type: def.type,
    title: def.title,
    category: CATEGORY_LABELS[def.category],
    description: def.description ?? "",
    runs: !!(def.executes || def.virtual),
    inputs: ports(def.inputs),
    outputs: ports(def.outputs),
    params: withParams ? paramsJsonSchema(def) : undefined,
  };
}

export const visibleNodeDefs = (): NodeDef[] => NODE_DEFS.filter((d) => !d.hidden);

export function nodeCatalog(withParams = false): CatalogNode[] {
  return visibleNodeDefs().map((d) => catalogNode(d, withParams));
}

export function catalogEntry(type: string): CatalogNode | null {
  const def = getNodeDef(type);
  return def ? catalogNode(def) : null;
}
