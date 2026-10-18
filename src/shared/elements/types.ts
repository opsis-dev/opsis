import type { z } from "zod";

export type ElementCategory =
  | "core"
  | "text"
  | "overlay"
  | "transition"
  | "background"
  | "effect"
  | "ui"
  | "data"
  | "scene"
  | "device";

export type ElementKind = "component" | "wrapper" | "presentation" | "sequencer";

export type ElementProps = Record<string, unknown>;

export interface ElementMeta {
  key: string;
  title: string;
  description?: string;
  category: ElementCategory;
  kind: ElementKind;
  source: { registry: string; url: string; license: string };
  schema: z.ZodObject;
  defaultProps: ElementProps;
  fixedProps: ElementProps;
  defaultDurationInFrames: number;
  thumbnail?: string;
}

export type ElementMetaInput = Omit<ElementMeta, "defaultProps">;

export function defineElementMeta(def: ElementMetaInput): ElementMeta {
  return { ...def, defaultProps: def.schema.parse({}) };
}
