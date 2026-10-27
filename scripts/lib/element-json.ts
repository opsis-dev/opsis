import { writeFileSync } from "node:fs";
import path from "node:path";
import type { ElementCategory, ElementKind } from "./element-meta";
import { ROOT } from "./tool-config";

export const ELEMENTS_JSON = path.join(
  ROOT,
  "crates",
  "compile",
  "src",
  "elements",
  "registry.generated.json",
);

export type Json = Record<string, unknown>;

export type ElementEntry = {
  key: string;
  title: string;
  description?: string;
  category: ElementCategory;
  kind: ElementKind;
  source: { registry: string; url: string; license: string };
  schema: Json;
  defaultProps: Json;
  fixedProps: Json;
  kindProps?: Record<string, string>;
  defaultDurationInFrames: number;
  design?: { width: number; height: number; exact?: boolean };
  optionalClip?: true;
};

export const elementsJsonText = (elements: readonly ElementEntry[]): string =>
  `${JSON.stringify(elements, null, 1)}\n`;

export function writeElementsJson(elements: readonly ElementEntry[]): void {
  writeFileSync(ELEMENTS_JSON, elementsJsonText(elements));
  console.log(`wrote ${path.relative(ROOT, ELEMENTS_JSON)}`);
}
