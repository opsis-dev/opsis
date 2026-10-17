import { coreElementMeta } from "./core";
import { generatedElementMeta } from "./elements.generated";
import type { ElementMeta } from "./types";

export { CAPTIONS_ELEMENT } from "./core";
export type { ElementCategory, ElementMeta } from "./types";

export const ELEMENTS: ElementMeta[] = [...coreElementMeta, ...generatedElementMeta];

const byKey = new Map(ELEMENTS.map((e) => [e.key, e]));

export const getElementMeta = (key: string): ElementMeta | undefined => byKey.get(key);

export const elementNodeType = (key: string): string => `fx.${key.replace("/", ".")}`;
