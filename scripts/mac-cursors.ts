import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { parse } from "smol-toml";
import { z } from "zod";
import { VIDEO_DIR } from "./lib/tool-config";

const OUT = path.join(VIDEO_DIR, "camera", "lib", "mac-cursors.generated.ts");
const THEME = "macOS";
const BOX = 256;

const CSS_KEYWORDS = [
  "default",
  "context-menu",
  "help",
  "pointer",
  "progress",
  "wait",
  "cell",
  "crosshair",
  "text",
  "vertical-text",
  "alias",
  "copy",
  "no-drop",
  "not-allowed",
  "grab",
  "grabbing",
  "all-scroll",
  "col-resize",
  "row-resize",
  "n-resize",
  "e-resize",
  "s-resize",
  "w-resize",
  "ne-resize",
  "nw-resize",
  "se-resize",
  "sw-resize",
  "ew-resize",
  "ns-resize",
  "nesw-resize",
  "nwse-resize",
  "zoom-in",
  "zoom-out",
];

const SAME_AS: Record<string, string> = { auto: "default", move: "all-scroll" };

const cursorSchema = z.looseObject({
  png: z.string().optional(),
  x_hotspot: z.number().optional(),
  y_hotspot: z.number().optional(),
  x11_symlinks: z.array(z.string()).optional(),
});
const configSchema = z.object({ cursors: z.record(z.string(), cursorSchema) });
const renderSchema = z.record(
  z.string(),
  z.object({
    dir: z.string(),
    colors: z.array(z.object({ match: z.string(), replace: z.string() })),
  }),
);

type Cursor = z.infer<typeof cursorSchema>;

function cursorFor(keyword: string, cursors: Record<string, Cursor>): string {
  const found = Object.entries(cursors).find(
    ([name, cursor]) => name === keyword || (cursor.x11_symlinks ?? []).includes(keyword),
  );
  if (!found) throw new Error(`apple_cursor has no cursor for "${keyword}"`);
  return found[0];
}

function sourceOf(source: string, dir: string, name: string, cursor: Cursor): string {
  const animated = cursor.png?.includes("*");
  return animated
    ? path.join(source, dir, name, `${name}-01.svg`)
    : path.join(source, dir, `${name}.svg`);
}

function recoloured(svg: string, colors: Array<{ match: string; replace: string }>): string {
  return colors.reduce(
    (text, { match, replace }) => text.replaceAll(new RegExp(match, "gi"), replace),
    svg,
  );
}

function main(): void {
  const source = process.argv[2];
  if (!source || !existsSync(path.join(source, "configs", "x.build.toml")))
    throw new Error("Pass the path of a checkout of https://github.com/ful1e5/apple_cursor.");
  const commit = execFileSync("git", ["rev-parse", "--short", "HEAD"], {
    cwd: source,
    encoding: "utf8",
  }).trim();
  const config = configSchema.parse(
    parse(readFileSync(path.join(source, "configs", "x.build.toml"), "utf8")),
  );
  const theme = renderSchema.parse(
    JSON.parse(readFileSync(path.join(source, "render.json"), "utf8")),
  )[THEME];
  if (!theme) throw new Error(`render.json has no "${THEME}" theme`);
  const { fallback_settings: fallback = {}, ...cursors } = config.cursors;
  const keywords = Object.fromEntries(CSS_KEYWORDS.map((k) => [k, cursorFor(k, cursors)]));
  const used = [...new Set(Object.values(keywords))].toSorted();
  const art = used.map((name) => {
    const cursor = cursors[name] ?? {};
    const svg = recoloured(
      readFileSync(sourceOf(source, theme.dir, name, cursor), "utf8"),
      theme.colors,
    );
    const hotspot = [
      cursor.x_hotspot ?? fallback.x_hotspot ?? BOX / 2,
      cursor.y_hotspot ?? fallback.y_hotspot ?? BOX / 2,
    ];
    const src = `data:image/svg+xml;base64,${Buffer.from(svg.replaceAll(/\s*\n\s*/g, " ").trim()).toString("base64")}`;
    return `  ${JSON.stringify(name)}: { src: ${JSON.stringify(src)}, hotspot: [${hotspot.join(", ")}] },`;
  });
  const aliases = Object.entries({
    ...keywords,
    ...Object.fromEntries(Object.entries(SAME_AS).map(([k, v]) => [k, keywords[v]])),
  })
    .toSorted(([a], [b]) => a.localeCompare(b))
    .map(([k, v]) => `  ${JSON.stringify(k)}: ${JSON.stringify(v)},`);
  writeFileSync(
    OUT,
    [
      `export const MAC_CURSOR_SOURCE = ${JSON.stringify(`ful1e5/apple_cursor ${commit}, ${THEME} theme`)};`,
      "",
      `export const MAC_CURSOR_BOX = ${BOX};`,
      "",
      "export const MAC_CURSORS: Readonly<Record<string, { src: string; hotspot: readonly [number, number] }>> = {",
      ...art,
      "};",
      "",
      "export const MAC_CURSOR_KEYWORDS: Readonly<Record<string, string>> = {",
      ...aliases,
      "};",
      "",
    ].join("\n"),
  );
  console.log(
    `wrote ${used.length} cursors for ${aliases.length} keywords to ${path.relative(process.cwd(), OUT)}`,
  );
}

main();
