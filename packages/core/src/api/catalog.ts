import { z } from "zod";
import { jsonSchemaOf } from "#core/api/bodies";
import { CATEGORY_LABELS, type NodeDef, type PortSpec } from "#core/nodes/define";
import { portLabel } from "#core/nodes/ports";
import { getNodeDef, NODE_DEFS } from "#core/nodes/registry";

export const catalogPortSchema = z.object({
  type: z.string(),
  label: z.string(),
  optional: z.boolean(),
  many: z.boolean().meta({ description: "Takes any number of connections" }),
});
type CatalogPort = z.infer<typeof catalogPortSchema>;

const portMap = z.record(z.string(), catalogPortSchema);

export const catalogNodeSchema = z.object({
  type: z.string(),
  title: z.string(),
  category: z.string(),
  description: z.string(),
  runs: z.boolean().meta({ description: "Does work in a run (records, exports)" }),
  inputs: portMap,
  outputs: portMap,
  params: z
    .record(z.string(), z.unknown())
    .optional()
    .meta({ description: "JSON Schema of the node's settings" }),
});
type CatalogNode = Omit<z.infer<typeof catalogNodeSchema>, "params">;

type CatalogEntry = CatalogNode & { params: z.core.JSONSchema.JSONSchema };

const ports = (specs: Record<string, PortSpec>): Record<string, CatalogPort> =>
  Object.fromEntries(
    Object.entries(specs).map(([id, p]) => [
      id,
      { type: p.type, label: portLabel(p), optional: !!p.optional, many: !!p.many },
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
    params: jsonSchemaOf(def.params),
  };
}
