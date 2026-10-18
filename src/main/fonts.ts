import { randomUUID } from "node:crypto";
import { copyFile, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { app, type BrowserWindow } from "electron";
import { unzipSync } from "fflate";
import { create, type Font } from "fontkit";
import { z } from "zod";
import { readJsonKeepingBroken, writeJson } from "../engine/json";
import { logError } from "../engine/log";
import { baseName, extensionOf, libraryRef, libraryRelPath, projectRef } from "../shared/asset-ref";
import { fontSlug, sameFamily } from "../shared/fonts";
import {
  familyNameFrom,
  planFontFiles,
  type FontCandidate,
  type PlannedFamily,
} from "../shared/fonts/plan";
import { fontFamilySchema, type FontFaceDef, type FontFamilyDef } from "../shared/fonts/types";
import { isRecord, isString } from "../shared/guards";
import type { FontInstalledFace, InspectedFonts } from "../shared/ipc";
import { firstFree } from "../shared/lists";
import { safeName } from "../shared/safe-name";
import { oneAtATime } from "../shared/timing";
import { openDialog } from "./dialogs";

const LICENSE_NAME = /(^|\/)(licen[cs]e|ofl|ffl|copying)[^/]*$/i;
const FORMAT: Partial<Record<string, FontFaceDef["format"]>> = {
  woff2: "woff2",
  woff: "woff",
  ttf: "truetype",
  otf: "opentype",
};
const FONT_EXTENSIONS = Object.keys(FORMAT);
const isFontFile = (name: string) => FONT_EXTENSIONS.includes(extensionOf(name));

export type FontBytes = { name: string; bytes: Uint8Array };

export type NewFamily = {
  family: string;
  category: FontFamilyDef["category"];
  source: FontFamilyDef["source"];
  files: Array<FontBytes & { face: Omit<FontFaceDef, "file"> }>;
  license?: FontBytes;
  licenseType?: string;
};

type FontFile = FontCandidate & { bytes: Uint8Array };

type Pending = {
  token: string;
  families: Array<PlannedFamily<FontFile>>;
  license?: FontBytes;
  source: FontFamilyDef["source"];
};

type FontTables = Pick<Font, "familyName" | "getName"> &
  Partial<Pick<Font, "subfamilyName" | "variationAxes" | "OS/2">>;

const librarySchema = z.object({ families: z.array(z.unknown()) });

export const fontsDir = () => path.join(app.getPath("userData"), "fonts");
const manifestPath = () => path.join(fontsDir(), "library.json");

let pending: Pending | null = null;
const changeLibrary = oneAtATime();

type Library = { families: FontFamilyDef[]; unreadable: unknown[] };

async function readLibrary(): Promise<Library> {
  const stored = await readJsonKeepingBroken(manifestPath(), librarySchema);
  const library: Library = { families: [], unreadable: [] };
  for (const entry of stored?.families ?? []) {
    const parsed = fontFamilySchema.safeParse(entry);
    if (parsed.success) library.families.push(parsed.data);
    else {
      logError("fonts", parsed.error, "a library entry couldn't be read and was kept as it is");
      library.unreadable.push(entry);
    }
  }
  return library;
}

export async function listLibrary(): Promise<FontFamilyDef[]> {
  return (await readLibrary()).families;
}

async function saveLibrary({ families, unreadable }: Library): Promise<void> {
  await mkdir(fontsDir(), { recursive: true });
  await writeJson(manifestPath(), { families: [...unreadable, ...families] });
}

const libraryIds = ({ families, unreadable }: Library): string[] => [
  ...families.map((f) => f.id),
  ...unreadable.flatMap((entry) => (isRecord(entry) && isString(entry.id) ? [entry.id] : [])),
];

function openFont(bytes: Uint8Array): FontTables | null {
  try {
    const opened = create(Buffer.from(bytes));
    return "fonts" in opened ? null : opened;
  } catch {
    return null;
  }
}

const isItalic = (font: FontTables): boolean =>
  Boolean(font["OS/2"]?.fsSelection?.italic) ||
  /italic|oblique/i.test(font.subfamilyName ?? "") ||
  Boolean(font.variationAxes?.ital);

function readFontCandidate(name: string, bytes: Uint8Array): FontCandidate | null {
  const font = openFont(bytes);
  if (!font) return null;
  const ext = extensionOf(name);
  const wght = font.variationAxes?.wght;
  const os2 = font["OS/2"];
  return {
    name,
    family: familyNameFrom(font.getName("preferredFamily", "en"), font.familyName),
    weight: wght
      ? `${Math.round(wght.min)} ${Math.round(wght.max)}`
      : String(os2?.usWeightClass ?? 400),
    style: isItalic(font) ? "italic" : "normal",
    variable: Boolean(wght),
    format: FORMAT[ext] ?? "truetype",
    size: bytes.byteLength,
  };
}

const withoutBytes = ({ bytes: _bytes, ...face }: FontFile): FontCandidate => face;

function collect(entries: FontBytes[], source: FontFamilyDef["source"]): InspectedFonts {
  const candidates: FontFile[] = [];
  let license: FontBytes | undefined;
  let unreadable = 0;
  for (const entry of entries) {
    if (isFontFile(entry.name)) {
      const info = readFontCandidate(entry.name, entry.bytes);
      if (info) candidates.push({ ...info, bytes: entry.bytes });
      else unreadable += 1;
    } else if (!license && LICENSE_NAME.test(entry.name)) {
      license = entry;
    }
  }
  const plan = planFontFiles(candidates);
  const token = randomUUID();
  pending = plan.length ? { token, families: plan, license, source } : null;
  return {
    token,
    families: plan.map((fam) => ({ family: fam.family, faces: fam.faces.map(withoutBytes) })),
    license: license && baseName(license.name),
    skipped: candidates.length - plan.reduce((n, f) => n + f.faces.length, 0),
    unreadable,
  };
}

export async function inspectFontFiles(
  win: BrowserWindow | null,
  paths?: string[],
): Promise<InspectedFonts | null> {
  const files = paths?.length
    ? paths
    : await openDialog(win, {
        title: "Add fonts",
        properties: ["openFile", "multiSelections"],
        filters: [{ name: "Fonts", extensions: [...FONT_EXTENSIONS, "zip"] }],
      });
  if (!files.length) return null;
  const entries: FontBytes[] = [];
  for (const file of files) {
    const bytes = new Uint8Array(await readFile(file));
    if (extensionOf(file) === "zip") {
      for (const [name, data] of Object.entries(unzipSync(bytes)))
        if (!name.endsWith("/") && !name.includes("__MACOSX")) entries.push({ name, bytes: data });
    } else {
      entries.push({ name: path.basename(file), bytes });
    }
  }
  return collect(entries, "file");
}

const isOpenType = (bytes: Uint8Array) => String.fromCharCode(...bytes.subarray(0, 4)) === "OTTO";

export function inspectInstalledFonts(faces: FontInstalledFace[]): InspectedFonts {
  return collect(
    faces.map((f) => {
      const bytes = new Uint8Array(f.data);
      return { name: `${f.postscriptName}.${isOpenType(bytes) ? "otf" : "ttf"}`, bytes };
    }),
    "installed",
  );
}

const uniqueId = (base: string, taken: string[]): string =>
  firstFree(
    (n) => (n === 1 ? base : `${base}-${n}`),
    (id) => taken.includes(id),
  );

export function guessCategory(family: string): FontFamilyDef["category"] {
  if (/mono|code/i.test(family)) return "monospace";
  if (/serif/i.test(family) && !/sans/i.test(family)) return "serif";
  return "sans-serif";
}

async function writeFamily(library: Library, fam: NewFamily): Promise<FontFamilyDef> {
  const existing = library.families.find((f) => sameFamily(f.family, fam.family));
  const id = existing?.id ?? uniqueId(fontSlug(fam.family), libraryIds(library));
  const dir = path.join(fontsDir(), id);
  await rm(dir, { recursive: true, force: true });
  await mkdir(dir, { recursive: true });
  const faces: FontFaceDef[] = [];
  for (const file of fam.files) {
    const base = safeName(baseName(file.name), { fallback: "font", lower: false });
    await writeFile(path.join(dir, base), file.bytes);
    faces.push({ ...file.face, file: libraryRef(`${id}/${base}`) });
  }
  if (fam.license) await writeFile(path.join(dir, "LICENSE.txt"), fam.license.bytes);
  return {
    id,
    family: fam.family,
    category: fam.category,
    source: fam.source,
    ...(fam.licenseType ? { license: fam.licenseType } : {}),
    ...(fam.license ? { licenseFile: libraryRef(`${id}/LICENSE.txt`) } : {}),
    faces,
  };
}

export function addFamilies(families: NewFamily[]): Promise<FontFamilyDef[]> {
  return changeLibrary(async () => {
    let library = await readLibrary();
    const added: FontFamilyDef[] = [];
    for (const fam of families) {
      const def = await writeFamily(library, fam);
      library = { ...library, families: [...library.families.filter((f) => f.id !== def.id), def] };
      added.push(def);
    }
    await saveLibrary(library);
    return added;
  });
}

export async function addInspectedFonts(
  token: string,
  families: string[],
): Promise<FontFamilyDef[]> {
  const data = pending;
  if (data?.token !== token)
    throw new Error("Those fonts are no longer ready to add. Choose them again.");
  pending = null;
  return addFamilies(
    data.families
      .filter((f) => families.includes(f.family))
      .map((fam) => ({
        family: fam.family,
        category: guessCategory(fam.family),
        source: data.source,
        files: fam.faces.map((face) => ({
          name: face.name,
          bytes: face.bytes,
          face: { weight: face.weight, style: face.style, format: face.format },
        })),
        license: data.license,
      })),
  );
}

export function releaseInspectedFonts(token: string): void {
  if (pending?.token === token) pending = null;
}

export function removeLibraryFont(id: string): Promise<void> {
  return changeLibrary(async () => {
    const library = await readLibrary();
    await rm(path.join(fontsDir(), id), { recursive: true, force: true });
    await saveLibrary({ ...library, families: library.families.filter((f) => f.id !== id) });
  });
}

export async function copyFontToProject(
  projectDir: string,
  def: FontFamilyDef,
): Promise<FontFamilyDef> {
  const dest = path.join(projectDir, "assets", "fonts", def.id);
  await mkdir(dest, { recursive: true });
  const toProject = async (ref: string) => {
    const rel = libraryRelPath(ref) ?? ref;
    const base = path.basename(rel);
    await copyFile(path.join(fontsDir(), rel), path.join(dest, base));
    return projectRef(`assets/fonts/${def.id}/${base}`);
  };
  const faces: FontFaceDef[] = [];
  for (const face of def.faces) faces.push({ ...face, file: await toProject(face.file) });
  return {
    ...def,
    faces,
    ...(def.licenseFile ? { licenseFile: await toProject(def.licenseFile) } : {}),
  };
}
