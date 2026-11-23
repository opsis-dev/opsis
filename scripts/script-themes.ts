import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { parse } from "smol-toml";
import { z } from "zod";
import { isString } from "./lib/common/guards";
import { fetchOk } from "./lib/download";
import { devCache, ROOT } from "./lib/tool-config";

const HELIX_TAG = "25.07.1";
const TARBALL_SHA256 = "27c8bc3eba46bc7bab1e3629c6b28ff94882eeff17366b3ea69cd8ceffba7541";
const SOURCE = `https://github.com/helix-editor/helix/archive/refs/tags/${HELIX_TAG}.tar.gz`;
const CACHE = devCache("helix-themes");
const OUT_THEMES = path.join(ROOT, "crates", "script", "src", "themes.generated.json");
const OUT_QUERY = path.join(ROOT, "crates", "script", "src", "highlights.generated.scm");
const DEFAULT_THEME = "default";
const LEFT_OUT = new Set([
  "modus_vivendi",
  "modus_vivendi_deuteranopia",
  "modus_vivendi_tinted",
  "modus_vivendi_tritanopia",
]);
const SYNTAX_SCOPES = new Set([
  "attribute",
  "comment",
  "constant",
  "constructor",
  "function",
  "keyword",
  "label",
  "namespace",
  "operator",
  "punctuation",
  "special",
  "string",
  "tag",
  "type",
  "variable",
]);
const NAMED = new Map([
  ["black", "#000000"],
  ["red", "#cd0000"],
  ["green", "#00cd00"],
  ["yellow", "#cdcd00"],
  ["blue", "#0000ee"],
  ["magenta", "#cd00cd"],
  ["cyan", "#00cdcd"],
  ["gray", "#7f7f7f"],
  ["light-red", "#ff0000"],
  ["light-green", "#00ff00"],
  ["light-yellow", "#ffff00"],
  ["light-blue", "#5c5cff"],
  ["light-magenta", "#ff00ff"],
  ["light-cyan", "#00ffff"],
  ["light-gray", "#e5e5e5"],
  ["white", "#ffffff"],
]);

const hex = z.string().regex(/^#[0-9a-fA-F]{6}$/);
type Raw = Record<string, unknown>;

const outStyle = z.object({
  fg: hex,
  bold: z.boolean().optional(),
  italic: z.boolean().optional(),
});
const outTheme = z.object({
  name: z.string(),
  label: z.string(),
  background: hex,
  text: hex,
  selection: hex,
  bracket: hex,
  error: hex,
  warning: hex,
  styles: z.record(z.string(), outStyle),
});
type Theme = z.infer<typeof outTheme>;

async function source(): Promise<string> {
  const dir = path.join(CACHE, `helix-${HELIX_TAG}`);
  if (existsSync(path.join(dir, "LICENSE"))) return dir;
  mkdirSync(CACHE, { recursive: true });
  const bytes = Buffer.from(await (await fetchOk(SOURCE)).arrayBuffer());
  const digest = createHash("sha256").update(bytes).digest("hex");
  if (digest !== TARBALL_SHA256)
    throw new Error(`The Helix ${HELIX_TAG} tarball isn't the pinned one (sha256 ${digest}).`);
  const tarball = path.join(CACHE, `helix-${HELIX_TAG}.tar.gz`);
  writeFileSync(tarball, bytes);
  rmSync(dir, { recursive: true, force: true });
  execFileSync("cmake", ["-E", "tar", "xf", tarball], { cwd: CACHE });
  rmSync(tarball);
  return dir;
}

function readTheme(dir: string, name: string): Raw {
  const file =
    name === DEFAULT_THEME
      ? path.join(dir, "theme.toml")
      : path.join(dir, "runtime", "themes", `${name}.toml`);
  return parse(readFileSync(file, "utf8")) as Raw;
}

function merged(dir: string, name: string, seen: Set<string> = new Set()): Raw {
  if (seen.has(name)) throw new Error(`The theme ${name} inherits from itself.`);
  seen.add(name);
  const own = readTheme(dir, name);
  const parent = typeof own.inherits === "string" ? merged(dir, own.inherits, seen) : {};
  const palette = {
    ...(parent.palette as Raw | undefined),
    ...(own.palette as Raw | undefined),
  };
  return { ...parent, ...own, palette };
}

function colour(value: unknown, palette: Raw): string | undefined {
  if (typeof value !== "string") return undefined;
  const named = palette[value];
  const resolved = typeof named === "string" ? named : value;
  const mapped = NAMED.get(resolved) ?? resolved;
  const result = /^#[0-9a-fA-F]{6}[fF]{2}$/.test(mapped) ? mapped.slice(0, 7) : mapped;
  return hex.safeParse(result).success ? result.toLowerCase() : undefined;
}

function part(theme: Raw, key: string, field: "fg" | "bg"): string | undefined {
  const value = theme[key];
  const palette = theme.palette as Raw;
  if (typeof value === "string") return field === "fg" ? colour(value, palette) : undefined;
  if (value !== null && typeof value === "object") {
    return colour((value as Raw)[field], palette);
  }
  return undefined;
}

function underline(theme: Raw, key: string): string | undefined {
  const value = theme[key];
  if (value === null || typeof value !== "object") return undefined;
  const line = (value as Raw).underline;
  if (line === null || typeof line !== "object") return undefined;
  return colour((line as Raw).color, theme.palette as Raw);
}

function modifiers(value: unknown): string[] {
  if (value === null || typeof value !== "object") return [];
  const list = (value as Raw).modifiers;
  return Array.isArray(list) ? list.filter(isString) : [];
}

function lightness(colourHex: string): number {
  function channel(offset: number) {
    const value = Number.parseInt(colourHex.slice(offset, offset + 2), 16) / 255;
    return value <= 0.039_28 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  }
  return 0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5);
}

const WORDS = new Map([
  ["github", "GitHub"],
  ["onedark", "One Dark"],
  ["onedarker", "One Darker"],
  ["jetbrains", "JetBrains"],
  ["nvim", "Neovim"],
  ["tokyonight", "Tokyo Night"],
  ["papercolor", "PaperColor"],
]);

function readable(name: string): string {
  if (name === DEFAULT_THEME) return "Helix Default";
  return name
    .split(/[_-]+/)
    .filter((word) => word.length > 0)
    .map((word) => WORDS.get(word) ?? `${word.charAt(0).toUpperCase()}${word.slice(1)}`)
    .join(" ");
}

function isDark(background: string): boolean {
  return lightness(background) < 0.4;
}

function built(dir: string, name: string): Theme | undefined {
  const theme = merged(dir, name);
  const background = part(theme, "ui.background", "bg");
  const text = part(theme, "ui.text", "fg") ?? part(theme, "ui.background", "fg");
  if (background === undefined || text === undefined || !isDark(background)) return undefined;
  const selection = part(theme, "ui.selection", "bg") ?? part(theme, "ui.selection", "fg") ?? text;
  const bracket =
    part(theme, "ui.cursor.match", "bg") ?? part(theme, "ui.cursor.match", "fg") ?? selection;
  const error =
    underline(theme, "diagnostic.error") ??
    part(theme, "diagnostic.error", "fg") ??
    part(theme, "error", "fg") ??
    colour("red", theme.palette as Raw) ??
    "#ff5555";
  const warning =
    underline(theme, "diagnostic.warning") ??
    part(theme, "diagnostic.warning", "fg") ??
    part(theme, "warning", "fg") ??
    colour("yellow", theme.palette as Raw) ??
    "#e5c07b";
  const styles: Theme["styles"] = {};
  for (const [key, value] of Object.entries(theme)) {
    if (!SYNTAX_SCOPES.has(key.split(".")[0] ?? "")) continue;
    const fg = part(theme, key, "fg");
    if (fg === undefined) continue;
    const marks = modifiers(value);
    styles[key] = {
      fg,
      ...(marks.includes("bold") ? { bold: true } : {}),
      ...(marks.includes("italic") ? { italic: true } : {}),
    };
  }
  return outTheme.parse({
    name,
    label: readable(name),
    background,
    text,
    selection,
    bracket,
    error,
    warning,
    styles,
  });
}

const dir = await source();
const names = readdirSync(path.join(dir, "runtime", "themes"))
  .filter((file) => file.endsWith(".toml"))
  .map((file) => file.slice(0, -".toml".length))
  .filter((name) => !LEFT_OUT.has(name))
  .toSorted();
const themes: Theme[] = [];
const skipped: string[] = [];
for (const name of [DEFAULT_THEME, ...names]) {
  const theme = built(dir, name);
  if (theme === undefined) skipped.push(name);
  else themes.push(theme);
}
writeFileSync(OUT_THEMES, `${JSON.stringify(themes)}\n`);
writeFileSync(
  OUT_QUERY,
  readFileSync(path.join(dir, "runtime", "queries", "lua", "highlights.scm"), "utf8"),
);
console.log(
  `wrote ${themes.length} themes to ${path.relative(ROOT, OUT_THEMES)} and the Lua query to ${path.relative(ROOT, OUT_QUERY)}`,
);
if (skipped.length > 0)
  console.log(
    `left out ${skipped.length} light themes and ones with no fixed background or text colour: ${skipped.join(", ")}`,
  );
