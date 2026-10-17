import { create } from "zustand";
import { VARIANTS } from "@/shared/nodes/output";
import { isOneOf } from "@/renderer/lib/one-of";
import { readPref, writePref } from "./view-prefs";

const PREVIEW_QUALITIES = ["fast", "exact"] as const;
export type PreviewQuality = (typeof PREVIEW_QUALITIES)[number];
export const isPreviewQuality = (v: unknown): v is PreviewQuality => isOneOf(PREVIEW_QUALITIES, v);

const PREVIEW_SHAPES = ["project", ...VARIANTS] as const;
export type PreviewShape = (typeof PREVIEW_SHAPES)[number];
export const isPreviewShape = (v: unknown): v is PreviewShape => isOneOf(PREVIEW_SHAPES, v);

const isBoolean = (v: unknown): v is boolean => typeof v === "boolean";

type ViewOptions = {
  previewQuality: PreviewQuality;
  previewShape: PreviewShape;
  minimap: boolean;
};

export const storedViewOptions = (): ViewOptions => ({
  previewQuality: readPref("previewQuality", "fast", isPreviewQuality),
  previewShape: readPref("previewShape", "project", isPreviewShape),
  minimap: readPref("minimap", true, isBoolean),
});

type ViewOptionsState = ViewOptions & {
  setPreviewQuality: (quality: PreviewQuality) => void;
  setPreviewShape: (shape: PreviewShape) => void;
  setMinimap: (minimap: boolean) => void;
};

export const useViewOptions = create<ViewOptionsState>()((set) => ({
  ...storedViewOptions(),
  setPreviewQuality: (previewQuality) => {
    set({ previewQuality });
    writePref("previewQuality", previewQuality);
  },
  setPreviewShape: (previewShape) => {
    set({ previewShape });
    writePref("previewShape", previewShape);
  },
  setMinimap: (minimap) => {
    set({ minimap });
    writePref("minimap", minimap);
  },
}));
