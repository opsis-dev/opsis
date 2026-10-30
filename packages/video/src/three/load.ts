import { SRGBColorSpace, TextureLoader, type Texture } from "three";
import { SVGLoader, type SVGResult } from "three/addons/loaders/SVGLoader.js";
import { useHeldLoad } from "#video/core/loading.ts";

export function isSvg(src: string): boolean {
  return /\.svg(\?|#|$)/i.test(src) || src.startsWith("data:image/svg");
}

async function loadSvg(src: string): Promise<SVGResult> {
  const res = await fetch(src);
  if (!res.ok) throw new Error(`Couldn't load the logo (${res.status}).`);
  return new SVGLoader().parse(await res.text());
}

async function loadTexture(src: string): Promise<Texture> {
  const texture = await new TextureLoader().loadAsync(src);
  texture.colorSpace = SRGBColorSpace;
  texture.anisotropy = 8;
  return texture;
}

export function useSvg(src: string | null): SVGResult | null {
  const loaded = useHeldLoad(`Loading logo ${src}`, src, loadSvg);
  return loaded && loaded.key === src ? loaded.value : null;
}

export function useTexture(src: string | null): Texture | null {
  const loaded = useHeldLoad(`Loading image ${src}`, src, loadTexture);
  return loaded && loaded.key === src ? loaded.value : null;
}
