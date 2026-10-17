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
import { googleFontModuleFile } from "../src/shared/fonts/google-font-imports";
import { FONT_CATEGORIES } from "../src/shared/fonts/types";
import { walkFiles } from "./lib/walk";

const ROOT = path.resolve(import.meta.dirname, "..");
const OUT = path.join(ROOT, "public", "fonts", "bundled");
const META_OUT = path.join(ROOT, "src", "shared", "fonts", "bundled.generated.ts");
const SHIM_DIR = path.join(ROOT, "src", "video", "fonts", "google");

const PACKAGES = [
  "@fontsource-variable/inter",
  "@fontsource-variable/geist",
  "@fontsource-variable/geist-mono",
  "@fontsource-variable/jetbrains-mono",
  "@fontsource-variable/manrope",
  "@fontsource-variable/dm-sans",
  "@fontsource-variable/space-grotesk",
  "@fontsource-variable/outfit",
  "@fontsource-variable/montserrat",
  "@fontsource-variable/plus-jakarta-sans",
  "@fontsource-variable/ibm-plex-sans",
  "@fontsource-variable/playfair-display",
  "@fontsource-variable/fraunces",
  "@fontsource-variable/caveat",
  "@fontsource-variable/google-sans-flex",
  "@fontsource/instrument-serif",
];

const GOOGLE_IMPORTS: Record<string, string> = {
  Inter: "Inter",
  JetBrainsMono: "JetBrains Mono",
  SpaceGrotesk: "Space Grotesk",
  Outfit: "Outfit",
  Montserrat: "Montserrat",
  InstrumentSerif: "Instrument Serif",
  GoogleSans: "Google Sans",
  Geist: "Geist",
  Caveat: "Caveat",
};

type Metadata = {
  id: string;
  family: string;
  category: string;
  styles: string[];
  weights: number[];
  license: { type: string };
};

type Face = { file: string; weight: string; style: string; format: string };

function bundle(pkg: string): string {
  const dir = path.join(ROOT, "node_modules", pkg);
  const meta = JSON.parse(readFileSync(path.join(dir, "metadata.json"), "utf8")) as Metadata;
  const variable = pkg.startsWith("@fontsource-variable/");
  const dest = path.join(OUT, meta.id);
  mkdirSync(dest, { recursive: true });
  const faces: Face[] = [];
  const min = Math.min(...meta.weights);
  const max = Math.max(...meta.weights);
  for (const style of meta.styles.toSorted().toReversed()) {
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
    category: FONT_CATEGORIES.find((c) => c === meta.category) ?? "sans-serif",
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
    if (!/\.(tsx?)$/.test(file)) continue;
    for (const m of readFileSync(file, "utf8").matchAll(/@remotion\/google-fonts\/([A-Za-z]+)/g))
      used.add(m[1]);
  }
  const missing = [...used].filter((name) => !(name in GOOGLE_IMPORTS));
  if (missing.length)
    throw new Error(
      `Elements load fonts that aren't bundled: ${missing.join(", ")}. Add them to PACKAGES and GOOGLE_IMPORTS.`,
    );
}

function writeBundle(): void {
  rmSync(OUT, { recursive: true, force: true });
  mkdirSync(OUT, { recursive: true });
  const defs = PACKAGES.map(bundle);
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
  for (const [name, family] of Object.entries(GOOGLE_IMPORTS))
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
  console.log(`${PACKAGES.length} families, ${bytes} bytes of font files`);
}

main();
