import { randomUUID } from "node:crypto";
import { mkdir, readFile, rm, writeFile, copyFile } from "node:fs/promises";
import path from "node:path";
import { app, dialog, type BrowserWindow } from "electron";
import { unzipSync } from "fflate";
import { create, type Font } from "fontkit";
import { fontFamilySchema, fontSlug, type FontFaceDef, type FontFamilyDef } from "../shared/fonts";
import type {
  FontCatalogEntry,
  FontInstalledFace,
  InspectedFonts,
  InspectedFamily,
} from "../shared/ipc";
import { familyNameFrom, planFontFiles, type FontCandidate } from "../shared/fonts/plan";
import { readJson } from "../engine/json";

const FONT_EXT = /\.(woff2|woff|ttf|otf)$/i;
const LICENSE_NAME = /(^|\/)(licen[cs]e|ofl|ffl|copying)[^/]*$/i;
const FORMAT: Record<string, FontFaceDef["format"]> = {
  woff2: "woff2",
  woff: "woff",
  ttf: "truetype",
  otf: "opentype",
};

export const fontsDir = () => path.join(app.getPath("userData"), "fonts");
const manifestPath = () => path.join(fontsDir(), "library.json");

type Pending = {
  families: Array<InspectedFamily & { files: Array<{ name: string; bytes: Uint8Array }> }>;
  license?: { name: string; bytes: Uint8Array };
  source: FontFamilyDef["source"];
};
const pending = new Map<string, Pending>();

export async function listLibrary(): Promise<FontFamilyDef[]> {
  const families = (await readJson<{ families?: unknown }>(manifestPath()))?.families;
  return (Array.isArray(families) ? families : []).flatMap((f) => {
    const parsed = fontFamilySchema.safeParse(f);
    return parsed.success ? [parsed.data] : [];
  });
}

async function saveLibrary(families: FontFamilyDef[]): Promise<void> {
  await mkdir(fontsDir(), { recursive: true });
  await writeFile(manifestPath(), JSON.stringify({ families }, null, 2));
}

function describe(name: string, bytes: Uint8Array): FontCandidate | null {
  const ext = /\.([a-z0-9]+)$/i.exec(name)?.[1]?.toLowerCase() ?? "";
  let font: Font;
  try {
    const opened = create(Buffer.from(bytes));
    if (!("familyName" in opened)) return null;
    font = opened as Font;
  } catch {
    return null;
  }
  const axes = font.variationAxes as Record<string, { min: number; max: number }> | undefined;
  const wght = axes?.wght;
  const os2 = (
    font as unknown as { "OS/2"?: { usWeightClass?: number; fsSelection?: { italic?: boolean } } }
  )["OS/2"];
  const sub = (font.subfamilyName ?? "").toLowerCase();
  const italic =
    Boolean(os2?.fsSelection?.italic) || /italic|oblique/.test(sub) || Boolean(axes?.ital);
  return {
    name,
    family: familyNameFrom(font.getName("preferredFamily", "en"), font.familyName),
    weight: wght
      ? `${Math.round(wght.min)} ${Math.round(wght.max)}`
      : String(os2?.usWeightClass ?? 400),
    style: italic ? "italic" : "normal",
    variable: Boolean(wght),
    format: FORMAT[ext] ?? "truetype",
    size: bytes.byteLength,
  };
}

function collect(
  entries: Array<{ name: string; bytes: Uint8Array }>,
  source: FontFamilyDef["source"],
): InspectedFonts {
  const candidates: FontCandidate[] = [];
  const bytesByName = new Map<string, Uint8Array>();
  let license: Pending["license"];
  for (const entry of entries) {
    if (FONT_EXT.test(entry.name)) {
      const info = describe(entry.name, entry.bytes);
      if (info) {
        candidates.push(info);
        bytesByName.set(entry.name, entry.bytes);
      }
    } else if (!license && LICENSE_NAME.test(entry.name)) {
      license = entry;
    }
  }
  const plan = planFontFiles(candidates);
  const token = randomUUID();
  const families = plan.map((fam) => ({
    ...fam,
    files: fam.faces.map((f) => ({ name: f.name, bytes: bytesByName.get(f.name)! })),
  }));
  pending.set(token, { families, license, source });
  return {
    token,
    families: plan,
    license: license?.name.split("/").pop(),
    skipped: candidates.length - plan.reduce((n, f) => n + f.faces.length, 0),
  };
}

export async function inspectFontFiles(
  win: BrowserWindow | null,
  paths?: string[],
): Promise<InspectedFonts | null> {
  let files = paths;
  if (!files?.length) {
    const options = {
      title: "Add fonts",
      properties: ["openFile", "multiSelections"] as Array<"openFile" | "multiSelections">,
      filters: [{ name: "Fonts", extensions: ["woff2", "woff", "ttf", "otf", "zip"] }],
    };
    const res = win
      ? await dialog.showOpenDialog(win, options)
      : await dialog.showOpenDialog(options);
    if (res.canceled || !res.filePaths.length) return null;
    files = res.filePaths;
  }
  const entries: Array<{ name: string; bytes: Uint8Array }> = [];
  for (const file of files) {
    const bytes = new Uint8Array(await readFile(file));
    if (/\.zip$/i.test(file)) {
      for (const [name, data] of Object.entries(unzipSync(bytes)))
        if (!name.endsWith("/") && !name.includes("__MACOSX")) entries.push({ name, bytes: data });
    } else {
      entries.push({ name: path.basename(file), bytes });
    }
  }
  return collect(entries, "file");
}

export function inspectInstalledFonts(faces: FontInstalledFace[]): InspectedFonts {
  return collect(
    faces.map((f) => ({
      name: `${f.postscriptName}.${f.format === "opentype" ? "otf" : "ttf"}`,
      bytes: new Uint8Array(f.data),
    })),
    "installed",
  );
}

async function addFamily(
  family: string,
  source: FontFamilyDef["source"],
  files: Array<{ face: Omit<FontFaceDef, "file">; name: string; bytes: Uint8Array }>,
  license?: { name: string; bytes: Uint8Array },
  licenseType?: string,
): Promise<FontFamilyDef> {
  const library = await listLibrary();
  const existing = library.find((f) => f.family.toLowerCase() === family.toLowerCase());
  const id = existing?.id ?? uniqueId(fontSlug(family), library);
  const dir = path.join(fontsDir(), id);
  await rm(dir, { recursive: true, force: true });
  await mkdir(dir, { recursive: true });
  const faces: FontFaceDef[] = [];
  for (const file of files) {
    const base = file.name
      .split("/")
      .pop()!
      .replace(/[^\w.-]+/g, "-");
    await writeFile(path.join(dir, base), file.bytes);
    faces.push({ ...file.face, file: `library:${id}/${base}` });
  }
  let licenseFile: string | undefined;
  if (license) {
    await writeFile(path.join(dir, "LICENSE.txt"), license.bytes);
    licenseFile = `library:${id}/LICENSE.txt`;
  }
  const def: FontFamilyDef = {
    id,
    family,
    category: guessCategory(family),
    source,
    ...(licenseType ? { license: licenseType } : {}),
    ...(licenseFile ? { licenseFile } : {}),
    faces,
  };
  await saveLibrary([...library.filter((f) => f.id !== id), def]);
  return def;
}

function uniqueId(base: string, library: FontFamilyDef[]): string {
  let id = base;
  for (let i = 2; library.some((f) => f.id === id); i++) id = `${base}-${i}`;
  return id;
}

function guessCategory(family: string): FontFamilyDef["category"] {
  if (/mono|code/i.test(family)) return "monospace";
  if (/serif/i.test(family) && !/sans/i.test(family)) return "serif";
  return "sans-serif";
}

export async function addInspectedFonts(
  token: string,
  families: string[],
): Promise<FontFamilyDef[]> {
  const data = pending.get(token);
  if (!data) throw new Error("Those fonts are no longer ready to add. Choose them again.");
  pending.delete(token);
  const added: FontFamilyDef[] = [];
  for (const fam of data.families.filter((f) => families.includes(f.family)))
    added.push(
      await addFamily(
        fam.family,
        data.source,
        fam.faces.map((face, i) => ({
          face: { weight: face.weight, style: face.style, format: face.format },
          name: fam.files[i].name,
          bytes: fam.files[i].bytes,
        })),
        data.license,
      ),
    );
  return added;
}

export async function removeLibraryFont(id: string): Promise<void> {
  const library = await listLibrary();
  await rm(path.join(fontsDir(), id), { recursive: true, force: true });
  await saveLibrary(library.filter((f) => f.id !== id));
}

let catalog: FontCatalogEntry[] | null = null;

export async function fontCatalog(): Promise<FontCatalogEntry[]> {
  if (catalog) return catalog;
  const res = await fetch("https://api.fontsource.org/v1/fonts");
  if (!res.ok) throw new Error("Couldn't load the font list. Check your internet connection.");
  const rows = (await res.json()) as Array<FontCatalogEntry & { subsets?: string[] }>;
  catalog = rows
    .filter((r) => r.subsets?.includes("latin") ?? true)
    .map(({ id, family, category, variable, styles, weights, license, type }) => ({
      id,
      family,
      category,
      variable,
      styles,
      weights,
      license,
      type,
    }));
  return catalog;
}

async function fetchBytes(url: string): Promise<Uint8Array> {
  const res = await fetch(url);
  if (!res.ok) throw new Error("A font file didn't download. Check your internet connection.");
  return new Uint8Array(await res.arrayBuffer());
}

type FontsourceDetail = {
  id: string;
  family: string;
  category: string;
  variable: boolean;
  styles: string[];
  weights: number[];
  license: string;
  variants: Record<string, Record<string, Record<string, { woff2?: string }>>>;
};

export async function downloadOnlineFont(id: string): Promise<FontFamilyDef> {
  const res = await fetch(`https://api.fontsource.org/v1/fonts/${encodeURIComponent(id)}`);
  if (!res.ok) throw new Error("Couldn't find that font online.");
  const detail = (await res.json()) as FontsourceDetail;
  const styles = detail.styles.filter(
    (s): s is "normal" | "italic" => s === "normal" || s === "italic",
  );
  const files: Array<{ face: Omit<FontFaceDef, "file">; name: string; bytes: Uint8Array }> = [];
  let axis: { min: string; max: string } | undefined;
  if (detail.variable) {
    const v = await fetch(`https://api.fontsource.org/v1/variable/${encodeURIComponent(id)}`);
    if (v.ok)
      axis = ((await v.json()) as { axes?: Record<string, { min: string; max: string }> }).axes
        ?.wght;
  }
  if (axis) {
    for (const style of styles) {
      const name = `${id}-latin-wght-${style}.woff2`;
      files.push({
        face: { weight: `${axis.min} ${axis.max}`, style, format: "woff2" },
        name,
        bytes: await fetchBytes(
          `https://cdn.jsdelivr.net/fontsource/fonts/${id}:vf@latest/latin-wght-${style}.woff2`,
        ),
      });
    }
  } else {
    for (const weight of detail.weights)
      for (const style of styles) {
        const url = detail.variants[String(weight)]?.[style]?.latin?.woff2;
        if (!url) continue;
        files.push({
          face: { weight: String(weight), style, format: "woff2" },
          name: `${id}-latin-${weight}-${style}.woff2`,
          bytes: await fetchBytes(url),
        });
      }
  }
  if (!files.length) throw new Error("That font has no Latin files to download.");
  let license: { name: string; bytes: Uint8Array } | undefined;
  try {
    license = {
      name: "LICENSE",
      bytes: await fetchBytes(`https://cdn.jsdelivr.net/npm/@fontsource/${id}/LICENSE`),
    };
  } catch {
    license = undefined;
  }
  const def = await addFamily(detail.family, "online", files, license, detail.license);
  const category = (["sans-serif", "serif", "monospace", "display", "handwriting"] as const).find(
    (c) => c === detail.category,
  );
  if (!category || category === def.category) return def;
  const library = await listLibrary();
  const fixed = { ...def, category };
  await saveLibrary(library.map((f) => (f.id === def.id ? fixed : f)));
  return fixed;
}

export async function copyFontToProject(
  projectDir: string,
  def: FontFamilyDef,
): Promise<FontFamilyDef> {
  const dest = path.join(projectDir, "assets", "fonts", def.id);
  await mkdir(dest, { recursive: true });
  const toProject = async (ref: string) => {
    const rel = ref.replace(/^library:/, "");
    const base = path.basename(rel);
    await copyFile(path.join(fontsDir(), rel), path.join(dest, base));
    return `project:assets/fonts/${def.id}/${base}`;
  };
  const faces: FontFaceDef[] = [];
  for (const face of def.faces) faces.push({ ...face, file: await toProject(face.file) });
  return {
    ...def,
    faces,
    ...(def.licenseFile ? { licenseFile: await toProject(def.licenseFile) } : {}),
  };
}
