import { create } from "zustand";
import { VARIANTS } from "@/shared/nodes/output";
import { isBoolean, isOneOf } from "@/shared/guards";
import { readPref, writePref } from "./view-prefs";

export const PREVIEW_QUALITIES = ["fast", "exact"] as const;
export type PreviewQuality = (typeof PREVIEW_QUALITIES)[number];
const isPreviewQuality = (v: unknown): v is PreviewQuality => isOneOf(PREVIEW_QUALITIES, v);

export const PREVIEW_SHAPES = ["project", ...VARIANTS] as const;
export type PreviewShape = (typeof PREVIEW_SHAPES)[number];
const isPreviewShape = (v: unknown): v is PreviewShape => isOneOf(PREVIEW_SHAPES, v);

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

const VIEW_OPTION_KEYS = Object.keys(PERSISTED) as Array<keyof ViewOptions>;

const stored = <K extends keyof ViewOptions>(key: K): ViewOptions[K] =>
  readPref(key, PERSISTED[key].fallback, PERSISTED[key].valid);

export const storedViewOptions = (): ViewOptions =>
  Object.fromEntries(VIEW_OPTION_KEYS.map((key) => [key, stored(key)])) as ViewOptions;

type ViewOptionsState = ViewOptions & {
  setViewOption: <K extends keyof ViewOptions>(key: K, value: ViewOptions[K]) => void;
};

export const useViewOptions = create<ViewOptionsState>()((set) => ({
  ...storedViewOptions(),
  setViewOption: (key, value) => {
    const patch: Partial<ViewOptions> = {};
    patch[key] = value;
    set(patch);
    writePref(key, value);
  },
}));
