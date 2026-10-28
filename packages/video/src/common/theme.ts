import { isRecord, isString } from "#video/common/guards.ts";
import type { Theme, ThemeMode } from "#video/generated/spec.ts";

export const THEME_MODES = ["light", "dark"] as const satisfies readonly ThemeMode[];

export const isTheme = (value: unknown): value is Theme =>
  isRecord(value) &&
  THEME_MODES.some((mode) => mode === value.mode) &&
  isRecord(value.colors) &&
  Object.values(value.colors).every(isString);

export function themeColors(value: Theme): Record<string, string> {
  return Object.fromEntries(Object.entries(value.colors).filter(([, color]) => color.trim()));
}
