import { registerFaces, unregisterFaces } from "@opsis/video/common/font-faces";
import { useEffect } from "react";
import { create } from "zustand";
import type { FontChoice } from "@/generated/bindings";
import { reportError } from "@/lib/toast";
import { fileUrl } from "../editor/file-url";

const registerPreview = (choice: FontChoice) =>
  void registerFaces(choice.preview, choice.def, fileUrl, () =>
    console.warn(`Couldn't load every style of ${choice.def.family} for its preview.`),
  );

export function usePreviewFont(choice: FontChoice | undefined): void {
  useEffect(() => {
    if (choice) registerPreview(choice);
  }, [choice]);
}

type LibraryState = {
  library: FontChoice[];
  status: "idle" | "loading" | "ready";
  refresh: () => Promise<void>;
  remove: (choice: FontChoice) => Promise<void>;
};

export const useFontLibrary = create<LibraryState>((set, get) => ({
  library: [],
  status: "idle",
  refresh: async () => {
    if (get().status === "idle") set({ status: "loading" });
    try {
      set({ library: await window.opsis.listFonts(), status: "ready" });
    } catch (e) {
      if (get().status === "loading") set({ status: "idle" });
      reportError("Couldn't read your font library", e);
    }
  },
  remove: async (choice) => {
    await window.opsis.removeFont(choice.def.id);
    unregisterFaces(choice.preview);
    await get().refresh();
  },
}));
