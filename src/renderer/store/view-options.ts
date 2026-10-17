import { create } from "zustand";
import { VARIANTS } from "@/shared/nodes/output";
import { isOneOf } from "@/renderer/lib/one-of";
import { isBoolean } from "@/shared/guards";
import { readPref, writePref } from "./view-prefs";

export const PREVIEW_QUALITIES = ["fast", "exact"] as const;
export type PreviewQuality = (typeof PREVIEW_QUALITIES)[number];
export const isPreviewQuality = (v: unknown): v is PreviewQuality => isOneOf(PREVIEW_QUALITIES, v);

export const PREVIEW_SHAPES = ["project", ...VARIANTS] as const;
export type PreviewShape = (typeof PREVIEW_SHAPES)[number];
export const isPreviewShape = (v: unknown): v is PreviewShape => isOneOf(PREVIEW_SHAPES, v);

type ViewOptions = {
  previewQuality: PreviewQuality;
  previewShape: PreviewShape;
  minimap: boolean;
};

type Persisted<T> = { fallback: T; valid: (v: unknown) => v is T };

const PERSISTED: { [K in keyof ViewOptions]: Persisted<ViewOptions[K]> } = {
  previewQuality: { fallback: "fast", valid: isPreviewQuality },
  previewShape: { fallback: "project", valid: isPreviewShape },
  minimap: { fallback: true, valid: isBoolean },
};

const stored = <K extends keyof ViewOptions>(key: K): ViewOptions[K] =>
  readPref(key, PERSISTED[key].fallback, PERSISTED[key].valid);

export const storedViewOptions = (): ViewOptions => ({
  previewQuality: stored("previewQuality"),
  previewShape: stored("previewShape"),
  minimap: stored("minimap"),
});

type ViewOptionsState = ViewOptions & {
  setPreviewQuality: (quality: PreviewQuality) => void;
  setPreviewShape: (shape: PreviewShape) => void;
  setMinimap: (minimap: boolean) => void;
};

export const useViewOptions = create<ViewOptionsState>()((set) => {
  const persisted =
    <K extends keyof ViewOptions>(key: K) =>
    (value: ViewOptions[K]) => {
      const patch: Partial<ViewOptions> = {};
      patch[key] = value;
      set(patch);
      writePref(key, value);
    };
  return {
    ...storedViewOptions(),
    setPreviewQuality: persisted("previewQuality"),
    setPreviewShape: persisted("previewShape"),
    setMinimap: persisted("minimap"),
  };
});
