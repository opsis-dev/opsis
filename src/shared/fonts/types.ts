import { z } from "zod";

export const FONT_CATEGORIES = [
  "sans-serif",
  "serif",
  "monospace",
  "display",
  "handwriting",
] as const;
export const FONT_SOURCES = ["bundled", "online", "file", "installed"] as const;

export const fontFaceSchema = z.object({
  file: z.string(),
  weight: z.string(),
  style: z.enum(["normal", "italic"]),
  format: z.enum(["woff2", "woff", "truetype", "opentype"]),
});
export type FontFaceDef = z.infer<typeof fontFaceSchema>;

export const fontFamilySchema = z.object({
  id: z.string(),
  family: z.string(),
  category: z.enum(FONT_CATEGORIES).default("sans-serif"),
  source: z.enum(FONT_SOURCES),
  license: z.string().optional(),
  licenseFile: z.string().optional(),
  faces: z.array(fontFaceSchema),
});
export type FontFamilyDef = z.infer<typeof fontFamilySchema>;

export const projectFontsSchema = z.object({
  main: z.string().default("Inter"),
  heading: z.string().default(""),
  code: z.string().default("JetBrains Mono"),
  families: z.array(fontFamilySchema).default([]),
});
export type ProjectFonts = z.infer<typeof projectFontsSchema>;

export const DEFAULT_PROJECT_FONTS: ProjectFonts = projectFontsSchema.parse({});
