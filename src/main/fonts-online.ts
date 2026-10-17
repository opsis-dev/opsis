import { z } from "zod";
import { FONT_CATEGORIES, type FontFamilyDef } from "../shared/fonts/types";
import type { FontCatalogEntry } from "../shared/ipc";
import { addFamilies, guessCategory, type FontBytes, type NewFamily } from "./fonts";

const API = "https://api.fontsource.org/v1";
const CDN = "https://cdn.jsdelivr.net";

const catalogEntrySchema = z.object({
  id: z.string(),
  family: z.string(),
  category: z.string(),
  variable: z.boolean(),
  styles: z.array(z.string()),
  weights: z.array(z.number()),
  license: z.string(),
  type: z.string(),
}) satisfies z.ZodType<FontCatalogEntry>;

const subsetsSchema = z.object({ subsets: z.array(z.string()).optional() });

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

let catalog: FontCatalogEntry[] | null = null;

const isLatin = (row: unknown) =>
  subsetsSchema.safeParse(row).data?.subsets?.includes("latin") ?? true;

export async function fontCatalog(): Promise<FontCatalogEntry[]> {
  if (catalog) return catalog;
  const res = await fetch(`${API}/fonts`);
  if (!res.ok) throw new Error("Couldn't load the font list. Check your internet connection.");
  const rows = z.array(z.unknown()).parse(await res.json());
  catalog = rows.flatMap((row) => {
    const entry = catalogEntrySchema.safeParse(row);
    return entry.success && isLatin(row) ? [entry.data] : [];
  });
  return catalog;
}

async function fetchBytes(url: string): Promise<Uint8Array> {
  const res = await fetch(url);
  if (!res.ok) throw new Error("A font file didn't download. Check your internet connection.");
  return new Uint8Array(await res.arrayBuffer());
}

async function weightAxis(id: string): Promise<{ min: string; max: string } | undefined> {
  const res = await fetch(`${API}/variable/${encodeURIComponent(id)}`);
  if (!res.ok) return undefined;
  return ((await res.json()) as { axes?: Record<string, { min: string; max: string }> }).axes?.wght;
}

async function downloadFaces(detail: FontsourceDetail): Promise<NewFamily["files"]> {
  const { id } = detail;
  const styles = detail.styles.filter(
    (s): s is "normal" | "italic" => s === "normal" || s === "italic",
  );
  const files: NewFamily["files"] = [];
  const axis = detail.variable ? await weightAxis(id) : undefined;
  if (axis) {
    for (const style of styles)
      files.push({
        face: { weight: `${axis.min} ${axis.max}`, style, format: "woff2" },
        name: `${id}-latin-wght-${style}.woff2`,
        bytes: await fetchBytes(
          `${CDN}/fontsource/fonts/${id}:vf@latest/latin-wght-${style}.woff2`,
        ),
      });
    return files;
  }
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
  return files;
}

const downloadLicense = (id: string): Promise<FontBytes | undefined> =>
  fetchBytes(`${CDN}/npm/@fontsource/${id}/LICENSE`).then(
    (bytes) => ({ name: "LICENSE", bytes }),
    () => undefined,
  );

const categoryOf = (detail: FontsourceDetail): FontFamilyDef["category"] =>
  z.enum(FONT_CATEGORIES).safeParse(detail.category).data ?? guessCategory(detail.family);

export async function downloadOnlineFont(id: string): Promise<FontFamilyDef> {
  const res = await fetch(`${API}/fonts/${encodeURIComponent(id)}`);
  if (!res.ok) throw new Error("Couldn't find that font online.");
  const detail = (await res.json()) as FontsourceDetail;
  const files = await downloadFaces(detail);
  if (!files.length) throw new Error("That font has no Latin files to download.");
  const [def] = await addFamilies([
    {
      family: detail.family,
      category: categoryOf(detail),
      source: "online",
      files,
      license: await downloadLicense(id),
      licenseType: detail.license,
    },
  ]);
  return def;
}
