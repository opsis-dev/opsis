import { createHash } from "node:crypto";
import { writeFileSync } from "node:fs";
import path from "node:path";
import { z } from "zod";
import { fetchOk } from "./lib/download";
import { ROOT } from "./lib/tool-config";

const VHS_COMMIT = "24fa2254a9806091e6ee6a980e9f3bcfe0a9ba53";
const THEMES_SHA256 = "a876676821391669c3074c1c6b2513e43f85d8e353366c1fb52e42e1fbdf20d0";
const SOURCE = `https://raw.githubusercontent.com/charmbracelet/vhs/${VHS_COMMIT}/themes.json`;
const OUT = path.join(ROOT, "crates", "compile", "src", "terminal", "themes.generated.json");

const colour = z.string().regex(/^#[0-9a-fA-F]{6}$/);

const vhsTheme = z.object({
  name: z.string(),
  black: colour,
  red: colour,
  green: colour,
  yellow: colour,
  blue: colour,
  magenta: colour,
  cyan: colour,
  white: colour,
  brightBlack: colour,
  brightRed: colour,
  brightGreen: colour,
  brightYellow: colour,
  brightBlue: colour,
  brightMagenta: colour,
  brightCyan: colour,
  brightWhite: colour,
  background: colour,
  foreground: colour,
  cursor: colour.optional(),
  meta: z.object({ isDark: z.boolean() }).optional(),
});

const PALETTE_ORDER = [
  "black",
  "red",
  "green",
  "yellow",
  "blue",
  "magenta",
  "cyan",
  "white",
  "brightBlack",
  "brightRed",
  "brightGreen",
  "brightYellow",
  "brightBlue",
  "brightMagenta",
  "brightCyan",
  "brightWhite",
] as const;

const response = await fetchOk(SOURCE);
const text = await response.text();
const digest = createHash("sha256").update(text).digest("hex");
if (digest !== THEMES_SHA256)
  throw new Error(`The themes file at ${SOURCE} isn't the pinned one (sha256 ${digest}).`);

const themes = z
  .array(vhsTheme)
  .parse(JSON.parse(text))
  .map((theme) => ({
    name: theme.name,
    fg: theme.foreground.toLowerCase(),
    bg: theme.background.toLowerCase(),
    cursor: (theme.cursor ?? theme.foreground).toLowerCase(),
    palette: PALETTE_ORDER.map((key) => theme[key].toLowerCase()),
    dark: theme.meta?.isDark ?? true,
  }));

writeFileSync(OUT, `${JSON.stringify(themes, null, 2)}\n`);
console.log(`wrote ${themes.length} themes to ${path.relative(ROOT, OUT)}`);
