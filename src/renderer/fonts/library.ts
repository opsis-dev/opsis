import { useEffect } from "react";
import { create } from "zustand";
import { reportError } from "@/renderer/lib/toast";
import { errorMessage } from "@/shared/errors";
import type { FontFamilyDef } from "@/shared/fonts/types";
import { fontFileUrl } from "../editor/file-url";

const previewFamily = (def: FontFamilyDef) => `df-preview-${def.source}-${def.id}`;

const facesKey = (def: FontFamilyDef) =>
  def.faces.map((f) => `${f.file} ${f.weight} ${f.style}`).join("\n");

const registered = new Map<string, { key: string; faces: FontFace[] }>();

function unregisterPreview(name: string): void {
  for (const face of registered.get(name)?.faces ?? []) document.fonts.delete(face);
  registered.delete(name);
}

const warnUnreadable = (def: FontFamilyDef, e: unknown) =>
  console.warn(`Couldn't load ${def.family} for its preview: ${errorMessage(e)}`);

function registerPreview(def: FontFamilyDef): void {
  const name = previewFamily(def);
  const key = facesKey(def);
  if (registered.get(name)?.key === key) return;
  unregisterPreview(name);
  const faces = def.faces.map(
    (face) =>
      new FontFace(name, `url("${fontFileUrl(face.file)}")`, {
        weight: face.weight,
        style: face.style,
      }),
  );
  for (const face of faces) {
    document.fonts.add(face);
    void face.load().catch((e: unknown) => warnUnreadable(def, e));
  }
  registered.set(name, { key, faces });
}

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
      set({ library: await window.demoflow.listFonts(), status: "ready" });
    } catch (e) {
      if (get().status === "loading") set({ status: "idle" });
      reportError("Couldn't read your font library", e);
    }
  },
  ensureLoaded: () => {
    if (get().status === "idle") void get().refresh();
  },
  remove: async (def) => {
    await window.demoflow.removeFont(def.id);
    unregisterPreview(previewFamily(def));
    await get().refresh();
  },
}));
