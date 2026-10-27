import { execFileSync } from "node:child_process";
import { existsSync, writeFileSync } from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { usedGlyphs } from "@opsis/ui/nodes/looks";
import { z } from "zod";
import { readJsonWith } from "./lib/read-json";
import { ROOT, UI_DIR } from "./lib/tool-config";

const OUT = path.join(UI_DIR, "editor", "node-glyphs.generated.ts");
const LUCIDE_ICONS = path.join(ROOT, "node_modules", "lucide", "dist", "esm", "icons");
const TABLER_NODES = path.join(
  ROOT,
  "node_modules",
  "@tabler",
  "icons",
  "tabler-nodes-outline.json",
);
const SHAPES = new Set(["path", "line", "circle", "rect", "polyline", "polygon", "ellipse"]);

const lookSchema = z.object({ glyph: z.string(), morph: z.string().optional() });
const catalogSchema = z.object({ nodes: z.array(z.object({ look: lookSchema.optional() })) });

function catalogLooks(): Array<z.infer<typeof lookSchema>> {
  const text = execFileSync(
    "cargo",
    ["run", "-q", "--release", "-p", "opsis-compile", "--bin", "editor-data", "catalog"],
    { cwd: ROOT, encoding: "utf8", maxBuffer: 1 << 26 },
  );
  return catalogSchema
    .parse(JSON.parse(text))
    .nodes.flatMap((node) => (node.look ? [node.look] : []));
}

const glyphNodeSchema = z.array(z.tuple([z.string(), z.record(z.string(), z.string())]));
type GlyphNode = z.infer<typeof glyphNodeSchema>;

async function lucideGlyph(key: string): Promise<GlyphNode> {
  const file = path.join(LUCIDE_ICONS, `${key}.mjs`);
  if (!existsSync(file)) throw new Error(`lucide:${key} is not an icon name (an alias or a typo)`);
  const mod: { default: unknown } = await import(pathToFileURL(file).href);
  return glyphNodeSchema.parse(mod.default);
}

const tablerNodes = readJsonWith(z.record(z.string(), glyphNodeSchema), TABLER_NODES);

function tablerGlyph(key: string): GlyphNode {
  const node = tablerNodes[key];
  if (!node) throw new Error(`tabler:${key} is not an outline icon`);
  return node.filter(([, attrs]) => attrs.stroke !== "none");
}

function checked(glyph: string, node: GlyphNode): GlyphNode {
  for (const [tag, attrs] of node) {
    if (!SHAPES.has(tag)) throw new Error(`${glyph} draws a <${tag}>`);
    if (attrs.fill && attrs.fill !== "none") throw new Error(`${glyph} has a filled part`);
  }
  return node.map(([tag, { key: _key, ...attrs }]) => [tag, attrs]);
}

async function load(glyph: string): Promise<GlyphNode> {
  const [set, key] = glyph.split(":");
  return checked(glyph, set === "lucide" ? await lucideGlyph(key) : tablerGlyph(key));
}

async function main(): Promise<void> {
  const glyphs = usedGlyphs(catalogLooks());
  const lines = await Promise.all(
    glyphs.map(
      async (glyph) => `  ${JSON.stringify(glyph)}: ${JSON.stringify(await load(glyph))},`,
    ),
  );
  writeFileSync(
    OUT,
    [
      "export const NODE_GLYPHS: Record<string, ReadonlyArray<readonly [string, Record<string, string>]>> = {",
      ...lines,
      "};",
      "",
    ].join("\n"),
  );
  console.log(`${glyphs.length} glyphs`);
}

await main();
