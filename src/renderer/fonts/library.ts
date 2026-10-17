import { useEffect } from "react";
import { toast } from "sonner";
import { create } from "zustand";
import { errorMessage } from "@/shared/errors";
import type { FontFamilyDef } from "@/shared/fonts/types";
import { encodePath } from "@/shared/asset-ref";
import { fileUrl } from "../editor/file-url";

export function fontFileUrl(ref: string): string {
  if (ref.startsWith("library:"))
    return `demoflow-file://fonts/${encodePath(ref.slice("library:".length))}`;
  return fileUrl(ref);
}

const previewFamily = (def: FontFamilyDef) => `df-preview-${def.source}-${def.id}`;

const registered = new Set<string>();

function registerPreview(def: FontFamilyDef): void {
  const name = previewFamily(def);
  if (registered.has(name)) return;
  registered.add(name);
  for (const face of def.faces) {
    const f = new FontFace(name, `url("${fontFileUrl(face.file)}")`, {
      weight: face.weight,
      style: face.style,
    });
    document.fonts.add(f);
    void f.load().catch(() => undefined);
  }
}

export function usePreviewFamily(def: FontFamilyDef | undefined): string | undefined {
  useEffect(() => {
    if (def) registerPreview(def);
  }, [def]);
  return def ? previewFamily(def) : undefined;
}

type LibraryState = {
  library: FontFamilyDef[];
  loaded: boolean;
  onlineAllowed: boolean;
  refresh: () => Promise<void>;
  ensureLoaded: () => void;
  allowOnline: () => void;
};

let loading = false;

export const useFontLibrary = create<LibraryState>((set, get) => ({
  library: [],
  loaded: false,
  onlineAllowed: false,
  refresh: async () => {
    const library = await window.demoflow.listFonts();
    set({ library, loaded: true });
  },
  ensureLoaded: () => {
    if (get().loaded || loading) return;
    loading = true;
    get()
      .refresh()
      .catch((e: unknown) =>
        toast.error("Could not read your font library", { description: errorMessage(e) }),
      )
      .finally(() => {
        loading = false;
      });
  },
  allowOnline: () => set({ onlineAllowed: true }),
}));
