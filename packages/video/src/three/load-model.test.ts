import { describe, expect, it } from "vitest";
import { modelExtensions } from "#video/three/load-model.ts";

function encode(json: object) {
  return new TextEncoder().encode(JSON.stringify(json));
}

function glb(json: object): ArrayBuffer {
  const text = encode(json);
  const padded = new Uint8Array(text.length + ((4 - (text.length % 4)) % 4)).fill(0x20);
  padded.set(text);
  const out = new Uint8Array(20 + padded.length);
  const view = new DataView(out.buffer);
  view.setUint32(0, 0x46546c67, true);
  view.setUint32(4, 2, true);
  view.setUint32(8, out.length, true);
  view.setUint32(12, padded.length, true);
  view.setUint32(16, 0x4e4f534a, true);
  out.set(padded, 20);
  return out.buffer;
}

describe("what a 3D model needs to open", () => {
  it("reads the extensions of a binary model and of a text one", () => {
    const json = {
      extensionsUsed: ["KHR_draco_mesh_compression"],
      extensionsRequired: ["EXT_meshopt_compression"],
    };
    expect(modelExtensions(glb(json))).toEqual([
      "KHR_draco_mesh_compression",
      "EXT_meshopt_compression",
    ]);
    expect(modelExtensions(encode({ extensionsUsed: ["KHR_texture_basisu"] }).buffer)).toEqual([
      "KHR_texture_basisu",
    ]);
    expect(modelExtensions(encode({ asset: { version: "2.0" } }).buffer)).toEqual([]);
  });
});
