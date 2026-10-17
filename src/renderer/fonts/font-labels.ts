import { FONT_CATEGORIES } from "@/shared/fonts/types";
import { isOneOf } from "@/renderer/lib/one-of";

const CATALOG_CATEGORIES = [...FONT_CATEGORIES, "icons"] as const;

const FONT_CATEGORY_LABELS: Record<(typeof CATALOG_CATEGORIES)[number], string> = {
  "sans-serif": "Sans serif",
  serif: "Serif",
  monospace: "Monospace",
  display: "Display",
  handwriting: "Handwriting",
  icons: "Icons",
};

export const categoryLabel = (category: string): string =>
  isOneOf(CATALOG_CATEGORIES, category) ? FONT_CATEGORY_LABELS[category] : "Other";

const WEIGHT_NAMES: Partial<Record<string, string>> = {
  "100": "Thin",
  "200": "Extra light",
  "300": "Light",
  "400": "Regular",
  "500": "Medium",
  "600": "Semibold",
  "700": "Bold",
  "800": "Extra bold",
  "900": "Black",
};

const weightName = (weight: string): string =>
  weight.includes(" ")
    ? `All weights (${weight.replace(" ", "–")})`
    : (WEIGHT_NAMES[weight] ?? weight);

export const describeFaces = (faces: Array<{ weight: string; style: string }>): string =>
  faces
    .map((f) => (f.style === "italic" ? `${weightName(f.weight)} italic` : weightName(f.weight)))
    .join(", ");
