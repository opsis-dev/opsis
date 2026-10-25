import type { CatalogCategory } from "@opsis/core/fonts/types";

const FONT_CATEGORY_LABELS: Record<CatalogCategory, string> = {
  "sans-serif": "Sans serif",
  serif: "Serif",
  monospace: "Monospace",
  display: "Display",
  handwriting: "Handwriting",
  icons: "Icons",
  other: "Other",
};

export const categoryLabel = (category: CatalogCategory): string => FONT_CATEGORY_LABELS[category];

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
