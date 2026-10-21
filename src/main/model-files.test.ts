import { describe, expect, it } from "vitest";
import { glbJson, modelFiles } from "./model-files";

const glb = (json: object): Buffer => {
  const text = Buffer.from(JSON.stringify(json));
  const padded = Buffer.concat([text, Buffer.alloc((4 - (text.length % 4)) % 4, 0x20)]);
  const header = Buffer.alloc(20);
  header.writeUInt32LE(0x46546c67, 0);
  header.writeUInt32LE(2, 4);
  header.writeUInt32LE(20 + padded.length, 8);
  header.writeUInt32LE(padded.length, 12);
  header.writeUInt32LE(0x4e4f534a, 16);
  return Buffer.concat([header, padded]);
};

describe("the files a 3D model needs next to it", () => {
  it("lists its buffers and images, but not the ones inside the file", () => {
    expect(
      modelFiles({
        buffers: [{ uri: "scene.bin" }, { uri: "data:application/octet-stream;base64,AAAA" }],
        images: [{ uri: "textures/wood%20grain.png" }, {}, { uri: "scene.bin" }],
      }),
    ).toEqual({ files: ["scene.bin", "textures/wood grain.png"], refused: [] });
  });

  it("refuses files outside the model's folder and anything on the internet", () => {
    expect(
      modelFiles({
        images: [
          { uri: "../secret.png" },
          { uri: "maps/../../up.png" },
          { uri: "/etc/passwd" },
          { uri: "https://example.com/a.png" },
          { uri: "file:///tmp/a.png" },
          { uri: "C:\\Users\\a.png" },
          { uri: "maps/./ok.png" },
        ],
      }),
    ).toEqual({
      files: ["maps/ok.png"],
      refused: [
        "../secret.png",
        "maps/../../up.png",
        "/etc/passwd",
        "https://example.com/a.png",
        "file:///tmp/a.png",
        "C:\\Users\\a.png",
      ],
    });
  });

  it("reads the description inside a binary model", () => {
    expect(modelFiles(glbJson(glb({ images: [{ uri: "skin.jpg" }] })))).toEqual({
      files: ["skin.jpg"],
      refused: [],
    });
    expect(() => glbJson(Buffer.from("not a model at all"))).toThrow("This isn't a 3D model file.");
  });
});
