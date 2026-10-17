import { describe, expect, it } from "vitest";
import {
  BASE_POSE,
  cameraEase,
  interpolatePose,
  moveDuration,
  poseToCss,
  poseTravelPx,
  posesNearlyEqual,
} from "./camera";

describe("cameraEase", () => {
  it("rises steadily from rest to the target and clamps outside the move", () => {
    let prev = cameraEase(-1);
    for (let i = 0; i <= 60; i++) {
      const v = cameraEase(i / 60);
      expect(v >= prev - 1e-9, `fell at ${i / 60}`).toBe(true);
      prev = v;
    }
    expect(cameraEase(2)).toBe(cameraEase(1));
  });

  it("has a short ease-in (velocity does not peak at t=0)", () => {
    const p01 = cameraEase(0.05);
    const p10 = cameraEase(0.1);
    const p15 = cameraEase(0.15);
    const early = p10 - p01;
    const next = p15 - p10;
    expect(next > early * 0.4, "should still be accelerating through the first 15%").toBe(true);
    expect(p01 < 0.12, "first 5% must not dump most of the distance").toBe(true);
  });
});

describe("poseTravelPx", () => {
  it("measures travel in design pixels whatever the recording size", () => {
    const to = { scale: 1.5, cx: 0.6, cy: 0.48 };
    expect(Math.abs(poseTravelPx(BASE_POSE, to, { width: 1920, height: 1080 }) - 192) < 1e-9).toBe(
      true,
    );
    expect(Math.abs(poseTravelPx(BASE_POSE, to, { width: 3840, height: 2160 }) - 192) < 1e-9).toBe(
      true,
    );
  });
});

describe("posesNearlyEqual", () => {
  it("ignores tiny differences and catches real moves", () => {
    expect(posesNearlyEqual(BASE_POSE, { scale: 1.005, cx: 0.505, cy: 0.485 })).toBe(true);
    expect(!posesNearlyEqual(BASE_POSE, { scale: 1.2, cx: 0.5, cy: 0.48 })).toBe(true);
  });
});

describe("interpolatePose", () => {
  it("drives scale and center with the same progress", () => {
    const a = { scale: 1, cx: 0.2, cy: 0.3 };
    const b = { scale: 2, cx: 0.8, cy: 0.9 };
    const mid = interpolatePose(a, b, 0.5);
    const p = (mid.scale - 1) / 1;
    expect(Math.abs((mid.cx - 0.2) / 0.6 - p) < 1e-9).toBe(true);
    expect(Math.abs((mid.cy - 0.3) / 0.6 - p) < 1e-9).toBe(true);
  });
});

describe("poseToCss", () => {
  it("is the identity at base scale", () => {
    const css = poseToCss(BASE_POSE, 0);
    expect(css.scale).toBe(1);
    expect(css.translateX).toBe(0);
    expect(css.translateY).toBe(0);
  });

  it("remaps content Y into the window group when chrome is present", () => {
    const noChrome = poseToCss({ scale: 1.5, cx: 0.5, cy: 0.5 }, 0);
    const chrome = poseToCss({ scale: 1.5, cx: 0.5, cy: 0.5 }, 0.1);
    expect(noChrome.translateY).not.toBe(chrome.translateY);
  });

  it("applies the base fit to scale and translate together", () => {
    const css = poseToCss({ scale: 1.5, cx: 0.4, cy: 0.4 }, 0, 0.9);
    expect(Math.abs(css.scale - 1.35) < 1e-9).toBe(true);
    expect(Math.abs(css.translateX - 1.35 * (0.5 - 0.4)) < 1e-9).toBe(true);
  });

  it("stays continuous as a zoom-out passes through base scale", () => {
    let prev = poseToCss({ scale: 1.6, cx: 0.35, cy: 0.62 }, 0.034, 0.9);
    for (let i = 1; i <= 200; i++) {
      const S = 1.6 - (0.6 * i) / 200;
      const css = poseToCss({ scale: S, cx: 0.35, cy: 0.62 }, 0.034, 0.9);
      expect(Number.isFinite(css.translateX) && Number.isFinite(css.translateY)).toBe(true);
      expect(
        Math.abs(css.translateX - prev.translateX) < 0.01,
        `translateX snapped at scale ${S}: ${prev.translateX} -> ${css.translateX}`,
      ).toBe(true);
      expect(
        Math.abs(css.translateY - prev.translateY) < 0.01,
        `translateY snapped at scale ${S}: ${prev.translateY} -> ${css.translateY}`,
      ).toBe(true);
      prev = css;
    }
    expect(prev.translateX).toBe(0);
    expect(prev.translateY).toBe(0);
  });
});

describe("moveDuration", () => {
  const vp = { width: 1920, height: 1080 };

  it("scales with displacement and keeps zoom-out longer than zoom-in", () => {
    const from = BASE_POSE;
    const to = { scale: 1.55, cx: 0.35, cy: 0.4 };
    const inn = moveDuration(from, to, "in", vp);
    const out = moveDuration(to, from, "out", vp);
    const tiny = moveDuration(from, { scale: 1.15, cx: 0.5, cy: 0.48 }, "in", vp);
    expect(out > inn, `${out} should be > ${inn}`).toBe(true);
    expect(inn >= 0.55 && inn <= 1.25).toBe(true);
    expect(tiny < inn).toBe(true);
  });
});
