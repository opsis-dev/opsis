import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { icons } from "lucide";
import { FRONTEND_DIR, ROOT } from "./lib/tool-config";

type IconNode = ReadonlyArray<readonly [string, Record<string, string>]>;

const GLYPHS_FILE = path.join(FRONTEND_DIR, "editor", "node-glyphs.generated.ts");
const ICONS_FILE = path.join(FRONTEND_DIR, "editor", "icons.ts");
const OUT = path.join(ROOT, "crates", "ui", "glyphs.generated.json");

const UI_ICONS = [
  "blend",
  "box",
  "braces",
  "check-check",
  "chevron-down",
  "chevron-right",
  "circle-check",
  "circle-slash",
  "circle-x",
  "clapperboard",
  "clock",
  "copy",
  "download",
  "file-play",
  "file-plus",
  "folder-open",
  "globe",
  "image",
  "layers",
  "layout-grid",
  "map",
  "maximize-2",
  "menu",
  "minus",
  "mouse-pointer-2",
  "music",
  "palette",
  "panel-bottom",
  "panel-left",
  "panel-left-open",
  "panel-right",
  "pause",
  "play",
  "plus",
  "redo-2",
  "refresh-cw",
  "save",
  "scan-search",
  "search",
  "settings",
  "shapes",
  "sparkles",
  "square",
  "square-terminal",
  "sticky-note",
  "trash-2",
  "type",
  "undo-2",
  "video",
  "wallpaper",
  "x",
];

function nodeGlyphs(): Record<string, IconNode> {
  const entries = readFileSync(GLYPHS_FILE, "utf8")
    .split("\n")
    .flatMap((line) => {
      const found = /^ {2}("[^"]+"): (\[.*\]),$/.exec(line);
      return found
        ? [[JSON.parse(found[1]) as string, JSON.parse(found[2]) as IconNode] as const]
        : [];
    });
  return Object.fromEntries(entries);
}

function cardIconNames(): string[] {
  const text = readFileSync(ICONS_FILE, "utf8");
  const block = text.slice(text.indexOf("NODE_ICONS"));
  return [...block.matchAll(/^ {2}"?([a-z0-9-]+)"?: [A-Za-z0-9]+Icon,$/gm)].map((m) => m[1]);
}

function pascal(name: string): string {
  return name
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
}

function lucideIcons(names: readonly string[]): Record<string, IconNode> {
  const table = icons as unknown as Record<string, IconNode>;
  return Object.fromEntries(
    names.map((name) => {
      const node = table[pascal(name)];
      if (!node) throw new Error(`no lucide icon ${name}`);
      return [name, node];
    }),
  );
}

function main(): void {
  const glyphs = nodeGlyphs();
  const iconNames = [...new Set([...cardIconNames(), ...UI_ICONS])].sort();
  writeFileSync(OUT, `${JSON.stringify({ glyphs, icons: lucideIcons(iconNames) })}\n`);
  console.log(`${Object.keys(glyphs).length} glyphs, ${iconNames.length} icons`);
}

main();
