import { isRecord, isString } from "#video/common/guards.ts";

export const THEME_MODES = ["light", "dark"] as const;

export type ThemeMode = (typeof THEME_MODES)[number];

export type ThemeValue = { mode: ThemeMode; colors: Record<string, string> };

export const isThemeValue = (value: unknown): value is ThemeValue =>
  isRecord(value) &&
  THEME_MODES.some((mode) => mode === value.mode) &&
  isRecord(value.colors) &&
  Object.values(value.colors).every(isString);

export function themeColors(value: ThemeValue): Record<string, string> {
  return Object.fromEntries(Object.entries(value.colors).filter(([, color]) => color.trim()));
}
