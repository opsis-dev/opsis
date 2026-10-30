import * as fc from "fast-check";
import { strToU8, zipSync } from "fflate";
import { describe, expect, it } from "vitest";
import { animationFrame, readLottie } from "#video/core/dot-lottie.ts";

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
    expect(readLottie(zip)).toMatchObject({
      fr: 30,
      assets: [{ e: 1, u: "", p: "data:image/png;base64,iVBORw==" }],
    });
  });

  it("finds the animation without a manifest", () => {
    const zip = zipSync({ "animations/x.json": strToU8(JSON.stringify(animation)) });
    expect(readLottie(zip)).toMatchObject({ op: 60 });
  });

  it("refuses a file that isn't an animation", () => {
    const zip = zipSync({ "animations/x.json": strToU8(JSON.stringify({ layers: [] })) });
    expect(() => readLottie(zip)).toThrow(/isn't an animation/);
  });

  it("explains an empty file", () => {
    const zip = zipSync({ "manifest.json": strToU8("{}") });
    expect(() => readLottie(zip)).toThrow(/no animation/);
  });
});

const timed = fc.record({
  first: fc.integer({ min: 0, max: 200 }),
  length: fc.integer({ min: 1, max: 600 }),
  rate: fc.constantFrom(12, 24, 25, 30, 60),
  frame: fc.integer({ min: 0, max: 20_000 }),
  fps: fc.constantFrom(24, 25, 30, 50, 60),
  speed: fc.double({ min: 0.1, max: 4, noNaN: true }),
  loop: fc.boolean(),
});

function lottieOf(first: number, length: number, rate: number) {
  return { fr: rate, ip: first, op: first + length, w: 100, h: 100 };
}

describe("animationFrame", () => {
  it("stays inside the animation's own frames", () => {
    fc.assert(
      fc.property(timed, fc.boolean(), ({ first, length, rate, ...timing }, backwards) => {
        const at = animationFrame(lottieOf(first, length, rate), { ...timing, backwards });
        expect(at).toBeGreaterThanOrEqual(first);
        expect(at).toBeLessThanOrEqual(first + length - 1);
      }),
    );
  });

  it("plays backwards as the mirror of forwards", () => {
    fc.assert(
      fc.property(timed, ({ first, length, rate, ...timing }) => {
        const data = lottieOf(first, length, rate);
        const forwards = animationFrame(data, { ...timing, backwards: false });
        const backwards = animationFrame(data, { ...timing, backwards: true });
        expect(forwards - first + (backwards - first)).toBeCloseTo(length - 1, 6);
      }),
    );
  });

  it("holds the last frame once a non-looping animation ends", () => {
    fc.assert(
      fc.property(timed, ({ first, length, rate, fps, speed }) => {
        const data = lottieOf(first, length, rate);
        const end = Math.ceil(((length * fps) / (speed * rate)) * 2);
        const at = animationFrame(data, { frame: end, fps, speed, loop: false, backwards: false });
        expect(at).toBe(first + length - 1);
      }),
    );
  });
});
