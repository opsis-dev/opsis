import type { FontFamilyDef } from "@opsis/core/fonts/types";
import { registerFaces, unregisterFaces } from "@opsis/video/spec";
import { useEffect } from "react";
import { create } from "zustand";
import { reportError } from "@/lib/toast";
import { fontFileUrl } from "../editor/file-url";

const previewFamily = (def: FontFamilyDef) => `df-preview-${def.source}-${def.id}`;

const registerPreview = (def: FontFamilyDef) =>
  void registerFaces(previewFamily(def), def, fontFileUrl, () =>
    console.warn(`Couldn't load every style of ${def.family} for its preview.`),
  );

export function usePreviewFamily(def: FontFamilyDef | undefined): string | undefined {
  useEffect(() => {
    if (def) registerPreview(def);
  }, [def]);
  return def ? previewFamily(def) : undefined;
}

type LibraryState = {
  library: FontFamilyDef[];
  status: "idle" | "loading" | "ready";
  refresh: () => Promise<void>;
  ensureLoaded: () => void;
  remove: (def: FontFamilyDef) => Promise<void>;
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
  ensureLoaded: () => {
    if (get().status === "idle") void get().refresh();
  },
  remove: async (def) => {
    await window.opsis.removeFont(def.id);
    unregisterFaces(previewFamily(def));
    await get().refresh();
  },
}));
