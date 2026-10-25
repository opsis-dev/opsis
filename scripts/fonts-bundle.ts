import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { FONT_CATEGORIES, type FontFamilyDef } from "@opsis/core/fonts/types";
import {
  googleFontModuleFile,
  googleImportName,
  GOOGLE_SHIM_SEGMENTS,
} from "@opsis/video/fonts/google-font-imports";
import { z } from "zod";
import { readJsonWith } from "./lib/read-json";
import { CORE_DIR, ROOT, VIDEO_DIR } from "./lib/tool-config";
import { walkFiles } from "./lib/walk";

const OUT = path.join(ROOT, "public", "fonts", "bundled");
const META_OUT = path.join(CORE_DIR, "fonts", "bundled.generated.ts");
const SHIM_DIR = path.join(VIDEO_DIR, ...GOOGLE_SHIM_SEGMENTS);

type BundledFont = { pkg: string; googleFamily?: string };

const FONTS: BundledFont[] = [
  { pkg: "@fontsource-variable/inter", googleFamily: "Inter" },
  { pkg: "@fontsource-variable/geist", googleFamily: "Geist" },
  { pkg: "@fontsource-variable/geist-mono" },
  { pkg: "@fontsource-variable/jetbrains-mono", googleFamily: "JetBrains Mono" },
  { pkg: "@fontsource-variable/manrope" },
  { pkg: "@fontsource-variable/dm-sans" },
  { pkg: "@fontsource-variable/space-grotesk", googleFamily: "Space Grotesk" },
  { pkg: "@fontsource-variable/outfit", googleFamily: "Outfit" },
  { pkg: "@fontsource-variable/montserrat", googleFamily: "Montserrat" },
  { pkg: "@fontsource-variable/plus-jakarta-sans" },
  { pkg: "@fontsource-variable/ibm-plex-sans" },
  { pkg: "@fontsource-variable/playfair-display" },
  { pkg: "@fontsource-variable/fraunces" },
  { pkg: "@fontsource-variable/caveat", googleFamily: "Caveat" },
  { pkg: "@fontsource-variable/google-sans-flex", googleFamily: "Google Sans" },
  { pkg: "@fontsource/instrument-serif", googleFamily: "Instrument Serif" },
];

const GOOGLE_IMPORTS = new Map(
  FONTS.flatMap(({ googleFamily }) =>
    googleFamily ? [[googleImportName(googleFamily), googleFamily] as const] : [],
  ),
);

const STYLE_ORDER = ["normal", "italic"] as const;

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

function checkImports(): void {
  const used = new Set<string>();
  for (const file of walkFiles(VIDEO_DIR)) {
    if (!/\.tsx?$/.test(file) || file.startsWith(SHIM_DIR)) continue;
    for (const m of readFileSync(file, "utf8").matchAll(/@remotion\/google-fonts\/([A-Za-z]+)/g))
      used.add(m[1]);
  }
  const missing = [...used].filter((name) => !GOOGLE_IMPORTS.has(name));
  if (missing.length)
    throw new Error(
      `Elements load fonts that aren't bundled: ${missing.join(", ")}. Add them to FONTS with their googleFamily.`,
    );
}

function writeBundle(): void {
  rmSync(OUT, { recursive: true, force: true });
  mkdirSync(OUT, { recursive: true });
  const defs = FONTS.map(copyFontFiles);
  writeFileSync(
    META_OUT,
    [
      'import type { FontFamilyDef } from "#core/fonts/types";',
      "",
      "export const BUNDLED_FONTS: FontFamilyDef[] = [",
      ...defs.map((def) => `  ${JSON.stringify(def)},`),
      "];",
      "",
    ].join("\n"),
  );
}

function writeShims(): void {
  rmSync(SHIM_DIR, { recursive: true, force: true });
  mkdirSync(SHIM_DIR, { recursive: true });
  for (const [name, family] of GOOGLE_IMPORTS)
    writeFileSync(
      path.join(SHIM_DIR, googleFontModuleFile(name)),
      `import { googleFont } from "#video/fonts/google-font.ts";\n\nexport const { loadFont, getInfo } = googleFont(${JSON.stringify(family)});\n`,
    );
}

function main(): void {
  checkImports();
  writeBundle();
  writeShims();
  const bytes = walkFiles(OUT)
    .filter((f) => f.endsWith(".woff2"))
    .reduce((n, f) => n + statSync(f).size, 0);
  console.log(`${FONTS.length} families, ${bytes} bytes of font files`);
}

main();
