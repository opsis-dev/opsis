import {
  copyFileSync,
  existsSync,
  mkdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import type { FontCategory, FontFamilyDef } from "@opsis/video/generated/spec";
import { z } from "zod";
import { readJsonWith } from "./lib/read-json";
import { ROOT } from "./lib/tool-config";
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

function writeBundle(): void {
  rmSync(OUT, { recursive: true, force: true });
  mkdirSync(OUT, { recursive: true });
  const defs = FONTS.map(copyFontFiles);
  mkdirSync(path.dirname(RUST_META_OUT), { recursive: true });
  writeFileSync(RUST_META_OUT, `${JSON.stringify(defs, null, 2)}\n`);
}

function main(): void {
  writeBundle();
  const bytes = walkFiles(OUT)
    .filter((f) => f.endsWith(".woff2"))
    .reduce((n, f) => n + statSync(f).size, 0);
  console.log(`${FONTS.length} families, ${bytes} bytes of font files`);
}

main();
