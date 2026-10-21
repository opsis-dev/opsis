import path from "node:path";
import { z } from "zod";

const GLB_MAGIC = 0x46546c67;
const GLB_JSON_CHUNK = 0x4e4f534a;
const GLB_HEADER_BYTES = 12;
const CHUNK_HEADER_BYTES = 8;

export const MODEL_EXTENSIONS = ["glb", "gltf"];

const gltfJsonSchema = z.object({
  buffers: z.array(z.object({ uri: z.string().optional() })).optional(),
  images: z.array(z.object({ uri: z.string().optional() })).optional(),
});

export type ModelFiles = { files: string[]; refused: string[] };

export function glbJson(bytes: Buffer): unknown {
  if (bytes.length < GLB_HEADER_BYTES + CHUNK_HEADER_BYTES || bytes.readUInt32LE(0) !== GLB_MAGIC)
    throw new Error("This isn't a 3D model file.");
  const length = bytes.readUInt32LE(GLB_HEADER_BYTES);
  const type = bytes.readUInt32LE(GLB_HEADER_BYTES + 4);
  if (type !== GLB_JSON_CHUNK) throw new Error("This 3D model file is damaged.");
  const start = GLB_HEADER_BYTES + CHUNK_HEADER_BYTES;
  return JSON.parse(bytes.subarray(start, start + length).toString("utf8"));
}

function localPath(uri: string): string | null {
  if (/^[a-z][a-z0-9+.-]*:/i.test(uri) || uri.startsWith("/") || uri.startsWith("\\")) return null;
  let decoded: string;
  try {
    decoded = decodeURIComponent(uri);
  } catch {
    return null;
  }
  const normal = path.posix.normalize(decoded.replaceAll("\\", "/"));
  if (normal.startsWith("../") || normal === ".." || path.isAbsolute(normal)) return null;
  return normal;
}

export function modelFiles(json: unknown): ModelFiles {
  const parsed = gltfJsonSchema.safeParse(json);
  if (!parsed.success) throw new Error("This 3D model file is damaged.");
  const uris = [...(parsed.data.buffers ?? []), ...(parsed.data.images ?? [])]
    .map((entry) => entry.uri)
    .filter((uri): uri is string => !!uri && !uri.startsWith("data:"));
  const files = new Set<string>();
  const refused: string[] = [];
  for (const uri of uris) {
    const file = localPath(uri);
    if (file) files.add(file);
    else refused.push(uri);
  }
  return { files: [...files], refused };
}
