import { z } from "zod";
import { THEME_MODES } from "./introspect";

export const themeValueSchema = z.object({
  mode: z.enum(THEME_MODES),
  colors: z.record(z.string(), z.string()),
});

export type ThemeValue = z.infer<typeof themeValueSchema>;

export const isThemeValue = (value: unknown): value is ThemeValue =>
  themeValueSchema.safeParse(value).success;

export function themeColors(value: ThemeValue): Record<string, string> {
  return Object.fromEntries(Object.entries(value.colors).filter(([, color]) => color.trim()));
}
