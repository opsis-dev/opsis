import type { BackdropName } from "../../../shared/backdrops";

export const DEFAULT_BACKDROP: BackdropName = "glaze";

export const LIGHT_BACKDROPS: readonly string[] = ["canyon", "mist", "chalk", "aurora"];

export const isLightBackdrop = (name: string): boolean => LIGHT_BACKDROPS.includes(name);

export function backdropFile(name: string = DEFAULT_BACKDROP): string {
  const trimmed = name.trim();
  if (trimmed === "") return `backdrops/${DEFAULT_BACKDROP}.jpg`;
  return trimmed.includes(".") ? `backdrops/${trimmed}` : `backdrops/${trimmed}.jpg`;
}

export function backdropProblem(value: unknown): string | null {
  if (value === undefined) return null;
  if (typeof value !== "string" || value.trim() === "")
    return "`backdrop` must be a non-empty string";
  if (/[/\\]/.test(value))
    return "`backdrop` is a name or a filename in public/backdrops/, not a path";
  return null;
}

export function isLightGround(hex: string): boolean {
  if (hex === "transparent") return true;
  const h = hex.replace("#", "");
  if (h.length !== 6) return false;
  const [r, g, b] = [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16) / 255);
  return 0.2126 * r + 0.7152 * g + 0.0722 * b > 0.6;
}
