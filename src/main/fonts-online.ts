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

const catalogRowSchema = catalogEntrySchema.extend({ subsets: z.array(z.string()).optional() });

const fileUrlsSchema = z.object({ woff2: z.string().optional() });

const detailSchema = catalogEntrySchema
  .pick({ id: true, family: true, category: true, variable: true, styles: true, weights: true })
  .extend({
    license: z.string().optional().catch(undefined),
    variants: z
      .partialRecord(
        z.string(),
        z.partialRecord(
          z.string(),
          z.partialRecord(z.string(), fileUrlsSchema.extend({ url: fileUrlsSchema.optional() })),
        ),
      )
      .default({}),
  });

type FontsourceDetail = z.infer<typeof detailSchema>;

const axesSchema = z.object({
  axes: z
    .partialRecord(z.string(), z.object({ min: z.coerce.string(), max: z.coerce.string() }))
    .optional(),
});

let catalog: FontCatalogEntry[] | null = null;

export async function fontCatalog(): Promise<FontCatalogEntry[]> {
  if (catalog) return catalog;
  const res = await fetch(`${API}/fonts`);
  if (!res.ok) throw new Error("Couldn't load the font list. Check your internet connection.");
  const rows = z.array(z.unknown()).parse(await res.json());
  catalog = rows.flatMap((row) => {
    const parsed = catalogRowSchema.safeParse(row);
    if (!parsed.success) return [];
    const { subsets, ...entry } = parsed.data;
    return !subsets || subsets.includes("latin") ? [entry] : [];
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
  return axesSchema.safeParse(await res.json()).data?.axes?.wght;
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
      const latin = detail.variants[String(weight)]?.[style]?.latin;
      const url = latin?.url?.woff2 ?? latin?.woff2;
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
  const parsed = detailSchema.safeParse(await res.json());
  if (!parsed.success) throw new Error("Couldn't read that font's details.");
  const detail = parsed.data;
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
