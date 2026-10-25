import { CATALOG_CATEGORIES, FONT_CATEGORIES, type FontFamilyDef } from "@opsis/core/fonts/types";
import { parseJsonWith } from "@opsis/core/guards";
import type { FontCatalogEntry } from "@opsis/core/ipc";
import { z } from "zod";
import { logError } from "../engine/log";
import { addFamilies, guessCategory, type FontBytes, type NewFamily } from "./fonts";

const API = "https://api.fontsource.org/v1";
const CDN = "https://cdn.jsdelivr.net";

const catalogEntrySchema = z.object({
  id: z.string(),
  family: z.string(),
  category: z.enum(CATALOG_CATEGORIES).catch("other"),
  variable: z.boolean(),
  styles: z.array(z.string()),
  weights: z.array(z.number()),
  license: z.string(),
  type: z.string(),
}) satisfies z.ZodType<FontCatalogEntry>;

const catalogRowSchema = catalogEntrySchema.extend({ subsets: z.array(z.string()).optional() });

const fileUrlsSchema = z.object({ url: z.object({ woff2: z.string().optional() }) });

const detailSchema = catalogEntrySchema
  .pick({ id: true, family: true, category: true, variable: true, styles: true, weights: true })
  .extend({
    license: z.string().optional().catch(undefined),
    variants: z
      .partialRecord(
        z.string(),
        z.partialRecord(z.string(), z.partialRecord(z.string(), fileUrlsSchema)),
      )
      .default({}),
  });

type FontsourceDetail = z.infer<typeof detailSchema>;

const axesSchema = z.object({
  axes: z
    .partialRecord(z.string(), z.object({ min: z.coerce.string(), max: z.coerce.string() }))
    .optional(),
});

const CHECK_CONNECTION = "Check your internet connection.";

const online = <T>(work: Promise<T>, failure: string): Promise<T> =>
  work.catch((error: unknown) => {
    throw new Error(`${failure} ${CHECK_CONNECTION}`, { cause: error });
  });

async function fetchOnline(url: string, failure: string): Promise<Response> {
  const res = await online(fetch(url), failure);
  if (!res.ok) throw new Error(failure);
  return res;
}

let catalog: FontCatalogEntry[] | null = null;

export async function fontCatalog(): Promise<FontCatalogEntry[]> {
  if (catalog) return catalog;
  const res = await fetchOnline(`${API}/fonts`, "Couldn't load the font list.");
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
  const res = await fetchOnline(url, "A font file didn't download.");
  return new Uint8Array(await res.arrayBuffer());
}

async function weightAxis(id: string): Promise<{ min: string; max: string } | undefined> {
  const res = await fetchOnline(
    `${API}/variable/${encodeURIComponent(id)}`,
    "A font didn't download.",
  );
  return parseJsonWith(axesSchema, await res.text())?.data?.axes?.wght;
}

async function downloadFaces(detail: FontsourceDetail): Promise<NewFamily["files"]> {
  const { id } = detail;
  const styles = detail.styles.filter(
    (s): s is "normal" | "italic" => s === "normal" || s === "italic",
  );
  const axis = detail.variable ? await weightAxis(id) : undefined;
  if (axis)
    return Promise.all(
      styles.map(async (style) => ({
        face: { weight: `${axis.min} ${axis.max}`, style, format: "woff2" },
        name: `${id}-latin-wght-${style}.woff2`,
        bytes: await fetchBytes(
          `${CDN}/fontsource/fonts/${id}:vf@latest/latin-wght-${style}.woff2`,
        ),
      })),
    );
  const faces = detail.weights.flatMap((weight) =>
    styles.flatMap((style) => {
      const url = detail.variants[String(weight)]?.[style]?.latin?.url.woff2;
      return url ? [{ weight: String(weight), style, url }] : [];
    }),
  );
  return Promise.all(
    faces.map(async ({ weight, style, url }) => ({
      face: { weight, style, format: "woff2" },
      name: `${id}-latin-${weight}-${style}.woff2`,
      bytes: await fetchBytes(url),
    })),
  );
}

const downloadLicense = (id: string): Promise<FontBytes | undefined> =>
  fetchBytes(`${CDN}/npm/@fontsource/${id}/LICENSE`).then(
    (bytes) => ({ name: "LICENSE", bytes }),
    (error: unknown) => {
      logError("fonts", error, `${id} licence`);
      return undefined;
    },
  );

const categoryOf = (detail: FontsourceDetail): FontFamilyDef["category"] =>
  z.enum(FONT_CATEGORIES).safeParse(detail.category).data ?? guessCategory(detail.family);

export async function downloadOnlineFont(id: string): Promise<FontFamilyDef> {
  const res = await fetchOnline(
    `${API}/fonts/${encodeURIComponent(id)}`,
    "Couldn't find that font online.",
  );
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
