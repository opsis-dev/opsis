import { isBoolean, isOneOf } from "@opsis/video/common/guards";
import { pick } from "@opsis/video/common/objects";
import { create } from "zustand";
import { VARIANTS } from "@/core/nodes";
import { PREVIEW_QUALITIES } from "@/core/timeline";
import type { PreviewQuality } from "@/generated/bindings";
import { readPref, writePref } from "./view-prefs";

const isPreviewQuality = (v: unknown): v is PreviewQuality => isOneOf(PREVIEW_QUALITIES, v);

export const PREVIEW_SHAPES = ["project", ...VARIANTS] as const;
type PreviewShape = (typeof PREVIEW_SHAPES)[number];
export const isPreviewShape = (v: unknown): v is PreviewShape => isOneOf(PREVIEW_SHAPES, v);

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
  setViewOption: <K extends keyof ViewOptions>(key: K, value: ViewOptions[K]) => void;
  toggleMinimap: () => void;
};

export const useViewOptions = create<ViewOptionsState>()((set, get) => ({
  ...storedViewOptions(),
  setViewOption: (key, value) => {
    set((s) => ({ ...s, [key]: value }));
    writePref(key, value);
  },
  toggleMinimap: () => get().setViewOption("minimap", !get().minimap),
}));

const SYNCED_VIEW_OPTIONS = ["previewQuality", "previewShape"] as const;

export type SyncedView = Pick<ViewOptions, (typeof SYNCED_VIEW_OPTIONS)[number]>;

export const syncedView = (): SyncedView => pick(useViewOptions.getState(), SYNCED_VIEW_OPTIONS);
