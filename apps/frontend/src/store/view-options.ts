import { isBoolean } from "@opsis/video/common/guards";
import { create } from "zustand";
import { readPref, writePref } from "./view-prefs";

type ViewOptions = {
  minimap: boolean;
};

export const storedViewOptions = (): ViewOptions => ({
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
