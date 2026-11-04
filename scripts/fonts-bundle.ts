import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { homedir } from "node:os";
import path from "node:path";
import { gunzipSync } from "node:zlib";
import type { FontCategory, FontFamilyDef } from "@opsis/video/generated/spec";
import { z } from "zod";
import { type Pinned, verifiedDownload } from "./lib/pins";
import { readJsonWith } from "./lib/read-json";
import { ROOT } from "./lib/tool-config";
import { unzipTo } from "./lib/unzip";
import { walkFiles } from "./lib/walk";

const OUT = path.join(ROOT, "public", "fonts", "bundled");
const RUST_META_OUT = path.join(
  ROOT,
  "crates",
  "compile",
  "src",
  "fonts",
  "bundled.generated.json",
);

type BundledFont = { pkg: string };

const FONTS: BundledFont[] = [
  { pkg: "@fontsource-variable/inter" },
  { pkg: "@fontsource-variable/geist" },
  { pkg: "@fontsource-variable/geist-mono" },
  { pkg: "@fontsource-variable/jetbrains-mono" },
  { pkg: "@fontsource-variable/manrope" },
  { pkg: "@fontsource-variable/dm-sans" },
  { pkg: "@fontsource-variable/space-grotesk" },
  { pkg: "@fontsource-variable/outfit" },
  { pkg: "@fontsource-variable/montserrat" },
  { pkg: "@fontsource-variable/plus-jakarta-sans" },
  { pkg: "@fontsource-variable/ibm-plex-sans" },
  { pkg: "@fontsource-variable/playfair-display" },
  { pkg: "@fontsource-variable/fraunces" },
  { pkg: "@fontsource-variable/caveat" },
  { pkg: "@fontsource-variable/google-sans-flex" },
  { pkg: "@fontsource/instrument-serif" },
];

const STYLE_ORDER = ["normal", "italic"] as const;

const FONT_CATEGORIES = [
  "sans-serif",
  "serif",
  "monospace",
  "display",
  "handwriting",
] as const satisfies FontCategory[];

const metadataSchema = z.object({
  id: z.string(),
  family: z.string(),
  category: z.enum(FONT_CATEGORIES),
  styles: z.array(z.enum(STYLE_ORDER)),
  weights: z.array(z.number()),
  license: z.object({ type: z.string() }),
});

type Face = FontFamilyDef["faces"][number];

function copyFontFiles({ pkg }: BundledFont): FontFamilyDef {
  const dir = path.join(ROOT, "node_modules", pkg);
  const meta = readJsonWith(metadataSchema, path.join(dir, "metadata.json"));
  const variable = pkg.startsWith("@fontsource-variable/");
  const dest = path.join(OUT, meta.id);
  mkdirSync(dest, { recursive: true });
  const faces: Face[] = [];
  const min = Math.min(...meta.weights);
  const max = Math.max(...meta.weights);
  for (const style of STYLE_ORDER.filter((s) => meta.styles.includes(s))) {
    const files = variable
      ? [{ name: `${meta.id}-latin-wght-${style}.woff2`, weight: `${min} ${max}` }]
      : meta.weights.map((w) => ({
          name: `${meta.id}-latin-${w}-${style}.woff2`,
          weight: String(w),
        }));
    for (const { name, weight } of files) {
      const src = path.join(dir, "files", name);
      if (!existsSync(src)) throw new Error(`${pkg}: missing ${name}`);
      copyFileSync(src, path.join(dest, name));
      faces.push({ file: `app:fonts/bundled/${meta.id}/${name}`, weight, style, format: "woff2" });
    }
  }
  copyFileSync(path.join(dir, "LICENSE"), path.join(dest, "LICENSE"));
  return {
    id: meta.id,
    family: meta.family,
    category: meta.category,
    source: "bundled",
    license: meta.license.type,
    licenseFile: `app:fonts/bundled/${meta.id}/LICENSE`,
    faces,
  };
}

const CACHE = path.join(homedir(), ".cache", "opsis-fonts");
const TERMINAL_OUT = path.join(OUT, "terminal");

const JETBRAINS_MONO: Pinned = {
  url: "https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip",
  sha256: "6f6376c6ed2960ea8a963cd7387ec9d76e3f629125bc33d1fdcd7eb7012f7bbf",
  size: 5622857,
};

const NERD_SYMBOLS: Pinned = {
  url: "https://deps.files.ghostty.org/NerdFontsSymbolsOnly-3.4.0.tar.gz",
  sha256: "1164d1b956d4bde248d7b2f0998c43cc94f5202431a1564a793895b1e73b0d04",
  size: 2932840,
};

const NOTO_EMOJI_COMMIT = "e20cbc2bbec1926686be9f9bee7d1d2cfa1fea0e";
const NOTO_EMOJI_FILES = `https://raw.githubusercontent.com/googlefonts/noto-emoji/${NOTO_EMOJI_COMMIT}/2D/fonts`;

const NOTO_EMOJI: Pinned = {
  url: `${NOTO_EMOJI_FILES}/NotoColorEmoji.ttf`,
  sha256: "15671215ab769fdc7162a045d56fd7d7e477c51b04e6b3c761d914d8fdd6cc44",
  size: 10730124,
};

const NOTO_EMOJI_LICENSE: Pinned = {
  url: `${NOTO_EMOJI_FILES}/LICENSE`,
  sha256: "6a73f9541c2de74158c0e7cf6b0a58ef774f5a780bf191f2d7ec9cc53efe2bf2",
  size: 4301,
};

const EMOJI_RANGES_PACKAGE = "@fontsource/noto-color-emoji";
const FIRST_EMOJI_CODE = 0x2000;

function emojiRanges(ranges: Record<string, string>): string {
  const parts = Object.values(ranges).flatMap((r) => r.split(","));
  const kept = parts.filter(
    (part) =>
      Number.parseInt(part.replace(/^U\+/i, "").split("-")[0] ?? "0", 16) >= FIRST_EMOJI_CODE,
  );
  return [...new Set(kept)].join(",");
}

const TAR_BLOCK = 512;
const TAR_NAME = 100;
const TAR_SIZE_AT = 124;
const TAR_SIZE_LENGTH = 12;
const TAR_TYPE_AT = 156;
const TAR_FILE_TYPES = new Set(["0", "\0"]);

function untarFile(archive: Buffer, wanted: string): Buffer {
  const tar = gunzipSync(archive);
  let at = 0;
  while (at + TAR_BLOCK <= tar.length) {
    const name = tar
      .subarray(at, at + TAR_NAME)
      .toString("utf8")
      .replace(/\0.*$/s, "");
    if (!name) break;
    const size = Number.parseInt(
      tar
        .subarray(at + TAR_SIZE_AT, at + TAR_SIZE_AT + TAR_SIZE_LENGTH)
        .toString("utf8")
        .trim(),
      8,
    );
    const start = at + TAR_BLOCK;
    const type = String.fromCharCode(tar[at + TAR_TYPE_AT] ?? 0);
    if (TAR_FILE_TYPES.has(type) && path.basename(name) === wanted)
      return tar.subarray(start, start + size);
    at = start + Math.ceil(size / TAR_BLOCK) * TAR_BLOCK;
  }
  throw new Error(`${wanted} isn't in the archive`);
}

function terminalFamily(id: string, family: string, faces: Face[], license: string): FontFamilyDef {
  return {
    id,
    family,
    category: "monospace",
    source: "bundled",
    license,
    licenseFile: `app:fonts/bundled/terminal/${id}/LICENSE`,
    faces,
    hidden: true,
  };
}

async function jetbrainsMono(): Promise<FontFamilyDef> {
  const zip = await verifiedDownload(JETBRAINS_MONO, CACHE);
  const unpacked = path.join(CACHE, "jetbrains-mono-2.304");
  if (!existsSync(unpacked)) await unzipTo(zip, unpacked);
  const id = "opsis-terminal";
  const dest = path.join(TERMINAL_OUT, id);
  mkdirSync(dest, { recursive: true });
  const faces: Face[] = [];
  for (const [file, style] of [
    ["JetBrainsMono[wght].ttf", "normal"],
    ["JetBrainsMono-Italic[wght].ttf", "italic"],
  ] as const) {
    const name = file.replace("[wght]", "-wght");
    copyFileSync(path.join(unpacked, "fonts", "variable", file), path.join(dest, name));
    faces.push({
      file: `app:fonts/bundled/terminal/${id}/${name}`,
      weight: "100 800",
      style,
      format: "truetype",
    });
  }
  copyFileSync(path.join(unpacked, "OFL.txt"), path.join(dest, "LICENSE"));
  return terminalFamily(id, "Opsis Terminal", faces, "OFL-1.1");
}

async function nerdSymbols(): Promise<FontFamilyDef> {
  const archive = readFileSync(await verifiedDownload(NERD_SYMBOLS, CACHE));
  const id = "opsis-terminal-symbols";
  const dest = path.join(TERMINAL_OUT, id);
  mkdirSync(dest, { recursive: true });
  const name = "SymbolsNerdFontMono-Regular.ttf";
  writeFileSync(path.join(dest, name), untarFile(archive, name));
  writeFileSync(path.join(dest, "LICENSE"), untarFile(archive, "LICENSE"));
  return terminalFamily(
    id,
    "Opsis Terminal Symbols",
    [
      {
        file: `app:fonts/bundled/terminal/${id}/${name}`,
        weight: "400",
        style: "normal",
        format: "truetype",
      },
    ],
    "MIT",
  );
}

const unicodeSchema = z.record(z.string(), z.string());

async function notoEmoji(): Promise<FontFamilyDef> {
  const id = "opsis-terminal-emoji";
  const dest = path.join(TERMINAL_OUT, id);
  mkdirSync(dest, { recursive: true });
  const name = "NotoColorEmoji.ttf";
  copyFileSync(await verifiedDownload(NOTO_EMOJI, CACHE), path.join(dest, name));
  copyFileSync(await verifiedDownload(NOTO_EMOJI_LICENSE, CACHE), path.join(dest, "LICENSE"));
  const ranges = readJsonWith(
    unicodeSchema,
    path.join(ROOT, "node_modules", EMOJI_RANGES_PACKAGE, "unicode.json"),
  );
  return terminalFamily(
    id,
    "Opsis Terminal Emoji",
    [
      {
        file: `app:fonts/bundled/terminal/${id}/${name}`,
        weight: "400",
        style: "normal",
        format: "truetype",
        unicodeRange: emojiRanges(ranges),
      },
    ],
    "OFL-1.1",
  );
}

function fontsourceSubsets(
  pkg: string,
  id: string,
  family: string,
  weights: number[],
): FontFamilyDef {
  const dir = path.join(ROOT, "node_modules", pkg);
  const meta = readJsonWith(metadataSchema, path.join(dir, "metadata.json"));
  const ranges = readJsonWith(unicodeSchema, path.join(dir, "unicode.json"));
  const dest = path.join(TERMINAL_OUT, id);
  mkdirSync(dest, { recursive: true });
  const faces: Face[] = [];
  for (const weight of weights)
    for (const [key, range] of Object.entries(ranges)) {
      const subset = key.replace(/^\[(.*)\]$/, "$1");
      const name = `${meta.id}-${subset}-${weight}-normal.woff2`;
      const src = path.join(dir, "files", name);
      if (!existsSync(src)) throw new Error(`${pkg}: missing ${name}`);
      copyFileSync(src, path.join(dest, name));
      faces.push({
        file: `app:fonts/bundled/terminal/${id}/${name}`,
        weight: String(weight),
        style: "normal",
        format: "woff2",
        unicodeRange: range,
      });
    }
  copyFileSync(path.join(dir, "LICENSE"), path.join(dest, "LICENSE"));
  return terminalFamily(id, family, faces, meta.license.type);
}

async function terminalFonts(): Promise<FontFamilyDef[]> {
  return [
    await jetbrainsMono(),
    await nerdSymbols(),
    fontsourceSubsets(
      "@fontsource/noto-sans-symbols-2",
      "opsis-terminal-symbols-2",
      "Opsis Terminal Symbols 2",
      [400],
    ),
    fontsourceSubsets(
      "@fontsource/noto-sans-sc",
      "opsis-terminal-cjk",
      "Opsis Terminal CJK",
      [400, 700],
    ),
    await notoEmoji(),
  ];
}

async function writeBundle(): Promise<void> {
  rmSync(OUT, { recursive: true, force: true });
  mkdirSync(OUT, { recursive: true });
  const defs = [...FONTS.map(copyFontFiles), ...(await terminalFonts())];
  mkdirSync(path.dirname(RUST_META_OUT), { recursive: true });
  writeFileSync(RUST_META_OUT, `${JSON.stringify(defs, null, 2)}\n`);
}

async function main(): Promise<void> {
  await writeBundle();
  const bytes = walkFiles(OUT)
    .filter((f) => f.endsWith(".woff2") || f.endsWith(".ttf"))
    .reduce((n, f) => n + statSync(f).size, 0);
  console.log(`${FONTS.length} families and the terminal stack, ${bytes} bytes of font files`);
}

await main();
