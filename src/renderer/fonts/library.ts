import { useEffect } from "react";
import { create } from "zustand";
import { libraryRelPath, type FontFamilyDef } from "@/shared/fonts/types";
import { FILE_SCHEME, encodePath } from "@/shared/asset-ref";
import { reportError } from "@/renderer/lib/toast";
import { fileUrl } from "../editor/file-url";

export function fontFileUrl(ref: string): string {
  const rel = libraryRelPath(ref);
  return rel === null ? fileUrl(ref) : `${FILE_SCHEME}://fonts/${encodePath(rel)}`;
}

const previewFamily = (def: FontFamilyDef) => `df-preview-${def.source}-${def.id}`;

const facesKey = (def: FontFamilyDef) =>
  def.faces.map((f) => `${f.file} ${f.weight} ${f.style}`).join("\n");

const registered = new Map<string, { key: string; faces: FontFace[] }>();

function registerPreview(def: FontFamilyDef): void {
  const name = previewFamily(def);
  const key = facesKey(def);
  const current = registered.get(name);
  if (current?.key === key) return;
  for (const face of current?.faces ?? []) document.fonts.delete(face);
  const faces = def.faces.map(
    (face) =>
      new FontFace(name, `url("${fontFileUrl(face.file)}")`, {
        weight: face.weight,
        style: face.style,
      }),
  );
  for (const face of faces) {
    document.fonts.add(face);
    void face.load().catch(() => undefined);
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
  onlineAllowed: boolean;
  refresh: () => Promise<void>;
  ensureLoaded: () => void;
  allowOnline: () => void;
};

export const useFontLibrary = create<LibraryState>((set, get) => ({
  library: [],
  status: "idle",
  onlineAllowed: false,
  refresh: async () => {
    if (get().status === "idle") set({ status: "loading" });
    try {
      set({ library: await window.demoflow.listFonts(), status: "ready" });
    } catch (e) {
      if (get().status === "loading") set({ status: "idle" });
      reportError("Could not read your font library", e);
    }
  },
  ensureLoaded: () => {
    if (get().status === "idle") void get().refresh();
  },
  allowOnline: () => set({ onlineAllowed: true }),
}));
