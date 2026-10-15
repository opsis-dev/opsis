import { strToU8, zipSync } from "fflate";
import { describe, expect, it } from "vitest";
import { readDotLottie } from "./dot-lottie";

const animation = {
  v: "5.7.0",
  fr: 30,
  ip: 0,
  op: 60,
  w: 100,
  h: 100,
  layers: [],
  assets: [{ id: "img_0", w: 10, h: 10, u: "/images/", p: "img_0.png", e: 0 }],
};

describe("dotLottie", () => {
  it("reads the first animation from the manifest and embeds its images", () => {
    const png = new Uint8Array([0x89, 0x50, 0x4e, 0x47]);
    const zip = zipSync({
      "manifest.json": strToU8(JSON.stringify({ animations: [{ id: "intro" }] })),
      "animations/intro.json": strToU8(JSON.stringify(animation)),
      "images/img_0.png": png,
    });
    const data = readDotLottie(zip) as unknown as typeof animation;
    expect(data.fr).toBe(30);
    expect(data.assets[0]).toMatchObject({ e: 1, u: "", p: "data:image/png;base64,iVBORw==" });
  });

  it("finds the animation without a manifest", () => {
    const zip = zipSync({ "animations/x.json": strToU8(JSON.stringify(animation)) });
    expect((readDotLottie(zip) as unknown as typeof animation).op).toBe(60);
  });

  it("explains an empty file", () => {
    const zip = zipSync({ "manifest.json": strToU8("{}") });
    expect(() => readDotLottie(zip)).toThrow(/no animation/);
  });
});
