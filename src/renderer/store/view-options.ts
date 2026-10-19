import { create } from "zustand";
import { isBoolean, isOneOf } from "@/shared/guards";
import { VARIANTS } from "@/shared/nodes/output";
import { pick } from "@/shared/objects";
import { PREVIEW_QUALITIES, type PreviewQuality } from "@/shared/schema/timeline";
import { readPref, writePref } from "./view-prefs";

const isPreviewQuality = (v: unknown): v is PreviewQuality => isOneOf(PREVIEW_QUALITIES, v);

export const PREVIEW_SHAPES = ["project", ...VARIANTS] as const;
type PreviewShape = (typeof PREVIEW_SHAPES)[number];
const isPreviewShape = (v: unknown): v is PreviewShape => isOneOf(PREVIEW_SHAPES, v);

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
