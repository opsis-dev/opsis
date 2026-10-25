import { coreElementMeta } from "#core/elements/core";
import { generatedElementMeta } from "#core/elements/elements.generated";
import type { ElementMeta } from "#core/elements/types";

export const ELEMENTS: ElementMeta[] = [...coreElementMeta, ...generatedElementMeta];

const byKey = new Map(ELEMENTS.map((e) => [e.key, e]));

export const getElementMeta = (key: string): ElementMeta | undefined => byKey.get(key);

export const elementNodeType = (key: string): string => `fx.${key.replace("/", ".")}`;
