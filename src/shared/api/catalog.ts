import { z } from "zod";
import { CATEGORY_LABELS, type NodeDef, type PortSpec } from "../nodes/define";
import { getNodeDef, NODE_DEFS } from "../nodes/registry";

type CatalogPort = {
  type: string;
  label: string;
  optional: boolean;
  many: boolean;
};

type CatalogNode = {
  type: string;
  title: string;
  category: string;
  description: string;
  runs: boolean;
  inputs: Record<string, CatalogPort>;
  outputs: Record<string, CatalogPort>;
};

type CatalogEntry = CatalogNode & { params: z.core.JSONSchema.JSONSchema };

const ports = (specs: Record<string, PortSpec>): Record<string, CatalogPort> =>
  Object.fromEntries(
    Object.entries(specs).map(([id, p]) => [
      id,
      { type: p.type, label: p.label ?? id, optional: !!p.optional, many: !!p.many },
    ]),
  );

const catalogNode = (def: NodeDef): CatalogNode => ({
  type: def.type,
  title: def.title,
  category: CATEGORY_LABELS[def.category],
  description: def.description ?? "",
  runs: !!(def.executes || def.virtual),
  inputs: ports(def.inputs),
  outputs: ports(def.outputs),
});

export const nodeCatalog = (): CatalogNode[] => NODE_DEFS.filter((d) => !d.hidden).map(catalogNode);

export function catalogEntry(type: string): CatalogEntry | null {
  const def = getNodeDef(type);
  if (!def) return null;
  return {
    ...catalogNode(def),
    params: z.toJSONSchema(def.params, { io: "input", unrepresentable: "any" }),
  };
}
