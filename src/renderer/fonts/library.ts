import { create } from "zustand";
import { BUNDLED_FONTS, type FontFamilyDef } from "@/shared/fonts";
import { fileUrl } from "../editor/file-url";

export function fontFileUrl(ref: string): string {
  if (ref.startsWith("app:")) return new URL(ref.slice("app:".length), document.baseURI).href;
  if (ref.startsWith("library:"))
    return `demoflow-file://fonts/${ref
      .slice("library:".length)
      .split("/")
      .map(encodeURIComponent)
      .join("/")}`;
  return fileUrl(ref);
}

export const previewFamily = (def: FontFamilyDef) => `df-preview-${def.source}-${def.id}`;

const registered = new Set<string>();

export function registerPreview(def: FontFamilyDef): string {
  const name = previewFamily(def);
  if (registered.has(name) || typeof FontFace === "undefined") return name;
  registered.add(name);
  for (const face of def.faces) {
    const f = new FontFace(name, `url("${fontFileUrl(face.file)}")`, {
      weight: face.weight,
      style: face.style,
    });
    document.fonts.add(f);
    void f.load().catch(() => undefined);
  }
  return name;
}

type LibraryState = {
  library: FontFamilyDef[];
  loaded: boolean;
  refresh: () => Promise<void>;
};

export const useFontLibrary = create<LibraryState>((set) => ({
  library: [],
  loaded: false,
  refresh: async () => {
    const library = await window.demoflow.listFonts();
    set({ library, loaded: true });
  },
}));

export { BUNDLED_FONTS };
