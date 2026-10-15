import type { z } from "zod";

// oxlint-disable-next-line typescript/no-explicit-any
type AnySchema = z.ZodType<any>;

export type FieldPreset = { label: string; values: Record<string, unknown> };

export type FieldMeta = {
  description?: string;
  help?: string;
  widget?: string;
  step?: number;
  unit?: string;
  accept?: string;
  advanced?: boolean;
  hidden?: boolean;
  labels?: Record<string, string>;
  zeroLabel?: string;
  presets?: FieldPreset[];
  pauses?: boolean;
  showIf?: Record<string, string[]>;
};

export function fieldMeta(schema: AnySchema): FieldMeta {
  const out: FieldMeta = {};
  let cur: AnySchema | undefined = schema;
  const chain: AnySchema[] = [];
  while (cur) {
    chain.push(cur);
    const def: { innerType?: AnySchema; type: string } = (
      cur as unknown as { _zod: { def: { innerType?: AnySchema; type: string } } }
    )._zod.def;
    cur = def.innerType;
  }
  for (const s of chain.toReversed())
    Object.assign(out, (s.meta?.() as FieldMeta | undefined) ?? {});
  return out;
}

export function unwrap(schema: AnySchema): AnySchema {
  let cur = schema;
  for (;;) {
    const def: { type: string; innerType?: AnySchema } = (
      cur as unknown as { _zod: { def: { type: string; innerType?: AnySchema } } }
    )._zod.def;
    if (
      def.innerType &&
      ["default", "optional", "nullable", "readonly", "catch", "prefault", "nonoptional"].includes(
        def.type,
      )
    ) {
      cur = def.innerType;
    } else return cur;
  }
}

export function assetKeys(schema: z.ZodObject): string[] {
  return Object.entries(schema.shape)
    .filter(([, s]) => fieldMeta(s as AnySchema).widget === "asset")
    .map(([k]) => k);
}

export function projectRef(path: string): string {
  if (!path) return path;
  if (/^(project:|app:|https?:|data:|blob:|demoflow-file:)/.test(path)) return path;
  return `project:${path.replace(/^\/+/, "")}`;
}
