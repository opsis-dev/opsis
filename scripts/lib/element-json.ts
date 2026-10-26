import { rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { z } from "zod";
import type { ElementMeta } from "./element-meta";
import { ROOT } from "./tool-config";

export const ELEMENTS_JSON = path.join(
  ROOT,
  "crates",
  "compile",
  "src",
  "elements",
  "registry.generated.json",
);

const REMOTION_COLOR = "__remotion-color";

const META_KEYS: Record<string, string> = {
  help: "x-help",
  widget: "x-widget",
  step: "x-step",
  unit: "x-unit",
  accept: "x-accept",
  advanced: "x-advanced",
  hidden: "x-hidden",
  labels: "x-labels",
  zeroLabel: "x-zero-label",
  unconnectedZeroLabel: "x-unconnected-zero-label",
  presets: "x-presets",
  presetsLabel: "x-presets-label",
  pauses: "x-pauses",
  timeKey: "x-time-key",
  lengthKey: "x-length-key",
  showIf: "x-show-if",
  showIfFile: "x-show-if-file",
  themes: "x-themes",
  framesAt: "x-frames-at",
};

const COLOUR_CHECK = { rule: "colour", message: "Pick a colour" };

type Json = Record<string, unknown>;

const isJson = (value: unknown): value is Json =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const renamedEach = (properties: unknown): unknown =>
  isJson(properties)
    ? Object.fromEntries(Object.entries(properties).map(([key, value]) => [key, renamed(value)]))
    : properties;

function renamed(schema: unknown): unknown {
  if (Array.isArray(schema)) return schema.map(renamed);
  if (!isJson(schema)) return schema;
  const out: Json = {};
  for (const [key, value] of Object.entries(schema)) {
    if (key === "description" && value === REMOTION_COLOR) continue;
    const named = META_KEYS[key] ?? key;
    if (key === "properties") out[key] = renamedEach(value);
    else out[named] = named.startsWith("x-") || key === "default" ? value : renamed(value);
  }
  return out;
}

export function elementJsonSchema(schema: z.ZodObject): unknown {
  const json = z.toJSONSchema(schema, {
    target: "draft-2020-12",
    io: "input",
    unrepresentable: "any",
    override: (ctx) => {
      const { description } = z.globalRegistry.get(ctx.zodSchema) ?? {};
      if (ctx.zodSchema._zod.def.type === "string" && description === REMOTION_COLOR)
        ctx.jsonSchema["x-check"] = COLOUR_CHECK;
    },
  });
  const { $schema: _schema, ...rest } = json;
  return renamed(rest);
}

export const elementEntry = (meta: ElementMeta) => ({
  key: meta.key,
  title: meta.title,
  ...(meta.description ? { description: meta.description } : {}),
  category: meta.category,
  kind: meta.kind,
  source: meta.source,
  schema: elementJsonSchema(meta.schema),
  defaultProps: meta.defaultProps,
  fixedProps: meta.fixedProps,
  defaultDurationInFrames: meta.defaultDurationInFrames,
  ...(meta.design ? { design: meta.design } : {}),
  ...(meta.optionalClip ? { optionalClip: true } : {}),
  ...(meta.thumbnail ? { thumbnail: meta.thumbnail } : {}),
});

export const elementsJsonText = (elements: readonly ElementMeta[]): string =>
  `${JSON.stringify(elements.map(elementEntry), null, 1)}\n`;

export const INGEST_DIR = path.join(ROOT, "scripts", ".ingest");

export const INGEST_META = path.join(INGEST_DIR, "elements.generated.ts");

type GeneratedMeta = { generatedElementMeta: ElementMeta[] };

export async function writeElementsJson(): Promise<void> {
  const { generatedElementMeta } = (await import(INGEST_META)) as GeneratedMeta;
  writeFileSync(ELEMENTS_JSON, elementsJsonText(generatedElementMeta));
  rmSync(INGEST_DIR, { recursive: true, force: true });
  console.log(`wrote ${path.relative(ROOT, ELEMENTS_JSON)}`);
}
