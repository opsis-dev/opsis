import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { z } from "zod";
import { readJsonWith } from "./read-json";
import { ROOT } from "./tool-config";

export const NOTICES = path.join(ROOT, "THIRD_PARTY_NOTICES.md");
const PACKAGES = path.join(ROOT, "node_modules");
const LUCIDE_ICONS = path.join(PACKAGES, "lucide", "dist", "esm", "icons");
const TABLER_NODES = path.join(PACKAGES, "@tabler", "icons", "tabler-nodes-outline.json");
const LUCIDE_LAB_NODES = path.join(PACKAGES, "@lucide", "lab", "icon-nodes.json");
const HUGEICONS = path.join(PACKAGES, "@hugeicons", "core-free-icons", "dist", "esm");
const ICONOIR = path.join(PACKAGES, "iconoir", "icons", "regular");
const MYNAUI = path.join(PACKAGES, "@mynaui", "icons", "icons");
const SHAPES = new Set(["path", "line", "circle", "rect", "polyline", "polygon", "ellipse"]);
const GEOMETRY = new Set([
  "d",
  "cx",
  "cy",
  "r",
  "rx",
  "ry",
  "x",
  "y",
  "width",
  "height",
  "x1",
  "y1",
  "x2",
  "y2",
  "points",
]);
const PAINT = new Set([
  "key",
  "fill",
  "stroke",
  "stroke-width",
  "strokeWidth",
  "stroke-linecap",
  "strokeLinecap",
  "stroke-linejoin",
  "strokeLinejoin",
]);

const glyphNodeSchema = z.array(z.tuple([z.string(), z.record(z.string(), z.string())]));
export type GlyphNode = z.infer<typeof glyphNodeSchema>;
const nodesSchema = z.record(z.string(), glyphNodeSchema);

async function moduleGlyph(file: string): Promise<unknown> {
  const mod: { default: unknown } = await import(pathToFileURL(file).href);
  return mod.default;
}

async function lucideGlyph(key: string): Promise<GlyphNode> {
  const file = path.join(LUCIDE_ICONS, `${key}.mjs`);
  if (!existsSync(file)) throw new Error(`lucide:${key} is not an icon name (an alias or a typo)`);
  return glyphNodeSchema.parse(await moduleGlyph(file));
}

const tablerNodes = readJsonWith(nodesSchema, TABLER_NODES);

function tablerGlyph(key: string): GlyphNode {
  const node = tablerNodes[key];
  if (!node) throw new Error(`tabler:${key} is not an outline icon`);
  return node.filter(([, attrs]) => attrs.stroke !== "none");
}

function lucideLabGlyph(key: string): GlyphNode {
  const node = readJsonWith(nodesSchema, LUCIDE_LAB_NODES)[key];
  if (!node) throw new Error(`lucide-lab:${key} is not an icon name`);
  return node;
}

function pascal(key: string): string {
  return key
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
}

const looseNodeSchema = z.array(
  z.tuple([z.string(), z.record(z.string(), z.union([z.string(), z.number()]))]),
);

async function hugeiconsGlyph(key: string): Promise<GlyphNode> {
  const file = path.join(HUGEICONS, `${pascal(key)}Icon.js`);
  if (!existsSync(file)) {
    throw new Error(`hugeicons:${key} is not a free icon's own name (an alias or a typo)`);
  }
  return looseNodeSchema
    .parse(await moduleGlyph(file))
    .map(([tag, attrs]) => [
      tag,
      Object.fromEntries(Object.entries(attrs).map(([name, value]) => [name, String(value)])),
    ]);
}

function svgNodes(glyph: string, text: string): GlyphNode {
  const root = /<svg\b([^>]*)>/.exec(text);
  if (!root) throw new Error(`${glyph} is not an SVG file`);
  const rootFill = /\sfill="([^"]*)"/.exec(root[1])?.[1];
  if (rootFill && rootFill !== "none") throw new Error(`${glyph} is filled`);
  const body = text.slice(root.index + root[0].length).replace(/<\/svg>\s*$/, "");
  return [...body.matchAll(/<([a-zA-Z]+)\b([^>]*?)\/?>/g)].map(([, tag, attrs]) => [
    tag,
    Object.fromEntries([...attrs.matchAll(/([a-zA-Z:-]+)="([^"]*)"/g)].map(([, n, v]) => [n, v])),
  ]);
}

function svgFileGlyph(glyph: string, folder: string, key: string): GlyphNode {
  const file = path.join(folder, `${key}.svg`);
  if (!existsSync(file)) throw new Error(`${glyph} is not an outline icon name`);
  return svgNodes(glyph, readFileSync(file, "utf8"));
}

export const GLYPH_SETS: Record<
  string,
  { notice: string; read: (key: string) => Promise<GlyphNode> | GlyphNode }
> = {
  lucide: { notice: "Lucide icons", read: lucideGlyph },
  tabler: { notice: "Tabler Icons", read: tablerGlyph },
  "lucide-lab": { notice: "Lucide Lab", read: lucideLabGlyph },
  hugeicons: { notice: "Hugeicons", read: hugeiconsGlyph },
  iconoir: { notice: "Iconoir", read: (key) => svgFileGlyph(`iconoir:${key}`, ICONOIR, key) },
  mynaui: { notice: "MynaUI", read: (key) => svgFileGlyph(`mynaui:${key}`, MYNAUI, key) },
};

export function geometryOnly(glyph: string, node: GlyphNode): GlyphNode {
  return node.map(([tag, attrs]) => {
    if (!SHAPES.has(tag)) throw new Error(`${glyph} draws a <${tag}>`);
    if (attrs.fill && attrs.fill !== "none") throw new Error(`${glyph} has a filled part`);
    const kept: Record<string, string> = {};
    for (const [name, value] of Object.entries(attrs)) {
      if (GEOMETRY.has(name)) kept[name] = value;
      else if (!PAINT.has(name)) throw new Error(`${glyph} sets ${name}, which a glyph can't`);
    }
    return [tag, kept];
  });
}

function credited(notices: string, glyph: string, notice: string): void {
  if (!notices.includes(`## ${notice}`)) {
    throw new Error(`${glyph} needs the ${notice} licence in THIRD_PARTY_NOTICES.md first`);
  }
}

export async function loadGlyph(glyph: string, notices: string): Promise<GlyphNode> {
  const at = glyph.indexOf(":");
  const [set, key] = [glyph.slice(0, at), glyph.slice(at + 1)];
  const source = GLYPH_SETS[set];
  if (!source || !key)
    throw new Error(`${glyph} isn't from one of ${Object.keys(GLYPH_SETS).join(", ")}`);
  credited(notices, glyph, source.notice);
  return geometryOnly(glyph, await source.read(key));
}
