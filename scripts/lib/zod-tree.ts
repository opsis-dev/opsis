import type { z } from "zod";

type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

type Def = Record<string, unknown> & { type: string };

type Check = { _zod: { def: Record<string, unknown> } };

const WRAPPERS = new Set(["optional", "nullable", "readonly", "nonoptional"]);

const defOf = (schema: z.ZodType): Def => schema.def as unknown as Def;

const jsonOf = (value: unknown): Json =>
  value === undefined ? null : (JSON.parse(JSON.stringify(value)) as Json);

function messageOf(check: Record<string, unknown>): string | null {
  const error = check.error;
  if (typeof error !== "function") return null;
  const said: unknown = error();
  if (typeof said === "string") return said;
  if (said && typeof said === "object" && "message" in said && typeof said.message === "string")
    return said.message;
  return null;
}

function checkTree(check: Check): Json {
  const def = check._zod.def;
  const out: Record<string, Json> = { check: String(def.check) };
  for (const key of ["value", "inclusive", "format", "minimum", "maximum", "length"])
    if (def[key] !== undefined) out[key] = jsonOf(def[key]);
  if (def.check === "custom") out.message = messageOf(def);
  return out;
}

function ownMeta(schema: z.ZodType): Json {
  const meta = schema.meta();
  return meta && Object.keys(meta).length ? jsonOf(meta) : null;
}

export function zodTree(schema: z.ZodType): Json {
  const def = defOf(schema);
  const out: Record<string, Json> = { type: def.type };
  const meta = ownMeta(schema);
  if (meta !== null) out.meta = meta;
  const checks = (def.checks as Check[] | undefined) ?? [];
  if (checks.length) out.checks = checks.map(checkTree);
  const inner = (key: string) => zodTree(def[key] as z.ZodType);
  if (WRAPPERS.has(def.type)) out.inner = inner("innerType");
  switch (def.type) {
    case "default":
    case "prefault":
      out.value = jsonOf(def.defaultValue);
      out.inner = inner("innerType");
      break;
    case "catch":
      out.inner = inner("innerType");
      break;
    case "enum":
      out.values = jsonOf(Object.values(def.entries as Record<string, unknown>));
      break;
    case "literal":
      out.values = jsonOf(def.values);
      break;
    case "object":
      out.shape = Object.entries(def.shape as Record<string, z.ZodType>).map(([key, field]) => [
        key,
        zodTree(field),
      ]);
      if (def.catchall) out.catchall = inner("catchall");
      break;
    case "array":
      out.element = inner("element");
      break;
    case "record":
      out.key = inner("keyType");
      out.value = inner("valueType");
      break;
    case "union":
      out.options = (def.options as z.ZodType[]).map(zodTree);
      if (def.discriminator !== undefined) out.discriminator = jsonOf(def.discriminator);
      break;
    case "tuple":
      out.items = (def.items as z.ZodType[]).map(zodTree);
      if (def.rest) out.rest = inner("rest");
      break;
    case "pipe":
      out.in = inner("in");
      out.out = inner("out");
      break;
    default:
      break;
  }
  return out;
}
