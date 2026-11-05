import type { ReactNode } from "react";
import { createContext, createElement, useContext } from "react";

export interface WarmTheme {
  background: string;
  foreground: string;
  card: string;
  cardForeground: string;
  popover: string;
  popoverForeground: string;
  primary: string;
  primaryForeground: string;
  secondary: string;
  secondaryForeground: string;
  muted: string;
  mutedForeground: string;
  accent: string;
  accentForeground: string;
  destructive: string;
  destructiveForeground: string;
  border: string;
  input: string;
  ring: string;
  radius: number;
  fontFamily?: string;
}

const defaultLightTheme: WarmTheme = {
  background: "#faf9f6",
  foreground: "#141414",
  card: "#ffffff",
  cardForeground: "#141414",
  popover: "#ffffff",
  popoverForeground: "#141414",
  primary: "#3072db",
  primaryForeground: "#ffffff",
  secondary: "#f2f0eb",
  secondaryForeground: "#141414",
  muted: "#f2f0eb",
  mutedForeground: "#6e6a63",
  accent: "#f2f0eb",
  accentForeground: "#141414",
  destructive: "#d92d20",
  destructiveForeground: "#ffffff",
  border: "#d9d9d9",
  input: "#d9d9d9",
  ring: "#3072db",
  radius: 10,
};

const defaultDarkTheme: WarmTheme = {
  background: "#0a0a0b",
  foreground: "#fafafa",
  card: "#141417",
  cardForeground: "#fafafa",
  popover: "#141417",
  popoverForeground: "#fafafa",
  primary: "#3072db",
  primaryForeground: "#ffffff",
  secondary: "#1d1d21",
  secondaryForeground: "#fafafa",
  muted: "#1b1b1f",
  mutedForeground: "#a1a1aa",
  accent: "#1d1d21",
  accentForeground: "#fafafa",
  destructive: "#f97066",
  destructiveForeground: "#fafafa",
  border: "#26272b",
  input: "#26272b",
  ring: "#3072db",
  radius: 10,
};

interface WarmThemeContextValue {
  theme?: Partial<WarmTheme>;
  mode?: "light" | "dark";
}

const WarmThemeContext = createContext<WarmThemeContextValue>({});

export interface WarmUIProviderProps {
  theme?: Partial<WarmTheme>;
  mode?: "light" | "dark";
  children: ReactNode;
}

export function WarmUIProvider({ theme, mode, children }: WarmUIProviderProps) {
  return createElement(WarmThemeContext.Provider, { value: { theme, mode } }, children);
}

export function useWarmTheme(
  override?: Partial<WarmTheme>,
  modeOverride?: "light" | "dark",
): WarmTheme {
  const ctx = useContext(WarmThemeContext);
  const mode = modeOverride ?? ctx.mode ?? "light";
  const base = mode === "dark" ? defaultDarkTheme : defaultLightTheme;
  return { ...base, ...ctx.theme, ...override };
}
