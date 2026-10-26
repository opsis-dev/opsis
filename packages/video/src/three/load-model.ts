import { GLTFLoader, type GLTF } from "three/addons/loaders/GLTFLoader.js";
import { z } from "zod";
import { parseJsonWith } from "#video/common/guards.ts";
import { useHeldLoad } from "#video/core/loading.ts";

const GLB_MAGIC = 0x46546c67;
const GLB_JSON_START = 20;
const UNSUPPORTED_TEXTURES = ["KHR_texture_basisu"];

export const MODEL_UNREADABLE = "Couldn't open this 3D model.";
export const TEXTURES_UNSUPPORTED = "This model uses a texture format that isn't supported yet.";

const extensionsSchema = z.object({
  extensionsUsed: z.array(z.string()).optional(),
  extensionsRequired: z.array(z.string()).optional(),
});

export function modelExtensions(buffer: ArrayBuffer): string[] {
  const view = new DataView(buffer);
  const binary = buffer.byteLength >= GLB_JSON_START && view.getUint32(0, true) === GLB_MAGIC;
  const text = binary
    ? new TextDecoder().decode(new Uint8Array(buffer, GLB_JSON_START, view.getUint32(12, true)))
    : new TextDecoder().decode(buffer);
  const parsed = parseJsonWith(extensionsSchema, text);
  if (parsed === null) throw new Error(MODEL_UNREADABLE);
  if (!parsed.success) return [];
  return [...(parsed.data.extensionsUsed ?? []), ...(parsed.data.extensionsRequired ?? [])];
}

const folderOf = (url: string) => url.slice(0, url.lastIndexOf("/") + 1);

async function fetchModel(url: string): Promise<GLTF> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(MODEL_UNREADABLE);
  const buffer = await res.arrayBuffer();
  let extensions: string[];
  try {
    extensions = modelExtensions(buffer);
  } catch (error) {
    throw new Error(MODEL_UNREADABLE, { cause: error });
  }
  if (extensions.some((e) => UNSUPPORTED_TEXTURES.includes(e)))
    throw new Error(TEXTURES_UNSUPPORTED);
  return new GLTFLoader().parseAsync(buffer, folderOf(url));
}

const models = new Map<string, Promise<GLTF>>();

function loadModel(url: string): Promise<GLTF> {
  let pending = models.get(url);
  if (!pending) {
    pending = fetchModel(url).catch((error: unknown) => {
      models.delete(url);
      throw error;
    });
    models.set(url, pending);
  }
  return pending;
}

export function useModel(url: string | null): GLTF | null {
  const loaded = useHeldLoad(`Loading 3D model ${url}`, url, loadModel);
  return loaded && loaded.key === url ? loaded.value : null;
}
