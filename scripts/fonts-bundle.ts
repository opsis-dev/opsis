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
import { z } from "zod";
import { googleFontModuleFile, googleImportName } from "../src/shared/fonts/google-font-imports";
import { FONT_CATEGORIES } from "../src/shared/fonts/types";
import { walkFiles } from "./lib/walk";
import { ROOT } from "./lib/tool-config";

const OUT = path.join(ROOT, "public", "fonts", "bundled");
const META_OUT = path.join(ROOT, "src", "shared", "fonts", "bundled.generated.ts");
const SHIM_DIR = path.join(ROOT, "src", "video", "fonts", "google");

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

type Face = { file: string; weight: string; style: string; format: string };

function bundle({ pkg }: BundledFont): string {
  const dir = path.join(ROOT, "node_modules", pkg);
  const meta = metadataSchema.parse(
    JSON.parse(readFileSync(path.join(dir, "metadata.json"), "utf8")),
  );
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
  const def = {
    id: meta.id,
    family: meta.family,
    category: meta.category,
    source: "bundled",
    license: meta.license.type,
    licenseFile: `app:fonts/bundled/${meta.id}/LICENSE`,
    faces,
  };
  return `  ${JSON.stringify(def)},`;
}

function checkImports(): void {
  const used = new Set<string>();
  for (const file of walkFiles(path.join(ROOT, "src", "video"))) {
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
  const defs = FONTS.map(bundle);
  writeFileSync(
    META_OUT,
    [
      'import type { FontFamilyDef } from "./types";',
      "",
      "export const BUNDLED_FONTS: FontFamilyDef[] = [",
      ...defs,
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
      `import { googleFont } from "../google-font";\n\nexport const { loadFont, getInfo } = googleFont(${JSON.stringify(family)});\n`,
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
