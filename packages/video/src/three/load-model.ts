import { useHeldLoad } from "@opsis/runtime/holds.ts";
import { GLTFLoader, type GLTF } from "three/addons/loaders/GLTFLoader.js";
import { KTX2Loader, type KTX2LoaderWorkerConfig } from "three/addons/loaders/KTX2Loader.js";

const MODEL_UNREADABLE = "Couldn't open this 3D model.";

const PLAIN_PIXELS: KTX2LoaderWorkerConfig = {
  astcSupported: false,
  astcHDRSupported: false,
  etc1Supported: false,
  etc2Supported: false,
  dxtSupported: false,
  bptcSupported: false,
  pvrtcSupported: false,
};

let textureLoader: KTX2Loader | null = null;

function compressedTextures(): KTX2Loader {
  if (!textureLoader) {
    textureLoader = new KTX2Loader();
    textureLoader.workerConfig = PLAIN_PIXELS;
  }
  return textureLoader;
}

function folderOf(url: string) {
  return url.slice(0, url.lastIndexOf("/") + 1);
}

async function fetchModel(url: string): Promise<GLTF> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(MODEL_UNREADABLE);
  const buffer = await res.arrayBuffer();
  try {
    return await new GLTFLoader()
      .setKTX2Loader(compressedTextures())
      .parseAsync(buffer, folderOf(url));
  } catch (error) {
    throw new Error(MODEL_UNREADABLE, { cause: error });
  }
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
