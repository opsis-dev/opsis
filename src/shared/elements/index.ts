import { coreElementMeta } from "./core";
import { generatedElementMeta } from "./elements.generated";
import type { ElementCategory, ElementMeta } from "./types";

export type { ElementCategory, ElementKind, ElementMeta, ElementProps } from "./types";

export const ELEMENTS: ElementMeta[] = [...coreElementMeta, ...generatedElementMeta];

const byKey = new Map(ELEMENTS.map((e) => [e.key, e]));

export const getElementMeta = (key: string): ElementMeta | undefined => byKey.get(key);

export const elementsIn = (category: ElementCategory): ElementMeta[] =>
  ELEMENTS.filter((e) => e.category === category);

export const elementNodeType = (key: string): string => `fx.${key.replace("/", ".")}`;
