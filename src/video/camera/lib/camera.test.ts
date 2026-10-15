import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { BASE_POSE, cameraEase, interpolatePose, moveDuration, poseToCss } from "./camera";

describe("cameraEase", () => {
  it("matches the measured Screen Studio-style curve within tolerance", () => {
    assert.ok(Math.abs(cameraEase(0.1) - 0.19) < 0.05, `0.10T = ${cameraEase(0.1)}`);
    assert.ok(Math.abs(cameraEase(0.5) - 0.89) < 0.05, `0.50T = ${cameraEase(0.5)}`);
    assert.ok(cameraEase(0) === 0);
    assert.ok(cameraEase(1) === 1);
  });

  it("has a short ease-in (velocity does not peak at t=0)", () => {
    const p01 = cameraEase(0.05);
    const p10 = cameraEase(0.1);
    const p15 = cameraEase(0.15);
    const early = p10 - p01;
    const next = p15 - p10;
    assert.ok(next > early * 0.4, "should still be accelerating through the first 15%");
    assert.ok(p01 < 0.12, "first 5% must not dump most of the distance");
  });
});

describe("interpolatePose", () => {
  it("drives scale and center with the same progress", () => {
    const a = { scale: 1, cx: 0.2, cy: 0.3 };
    const b = { scale: 2, cx: 0.8, cy: 0.9 };
    const mid = interpolatePose(a, b, 0.5);
    const p = (mid.scale - 1) / 1;
    assert.ok(Math.abs((mid.cx - 0.2) / 0.6 - p) < 1e-9);
    assert.ok(Math.abs((mid.cy - 0.3) / 0.6 - p) < 1e-9);
  });
});

describe("poseToCss", () => {
  it("is the identity at base scale", () => {
    const css = poseToCss(BASE_POSE, 0);
    assert.equal(css.scale, 1);
    assert.equal(css.translateX, 0);
    assert.equal(css.translateY, 0);
  });

  it("remaps content Y into the window group when chrome is present", () => {
    const noChrome = poseToCss({ scale: 1.5, cx: 0.5, cy: 0.5 }, 0);
    const chrome = poseToCss({ scale: 1.5, cx: 0.5, cy: 0.5 }, 0.1);
    assert.notEqual(noChrome.translateY, chrome.translateY);
  });

  it("applies the base fit to scale and translate together", () => {
    const css = poseToCss({ scale: 1.5, cx: 0.4, cy: 0.4 }, 0, 0.9);
    assert.ok(Math.abs(css.scale - 1.35) < 1e-9);
    assert.ok(Math.abs(css.translateX - 1.35 * (0.5 - 0.4)) < 1e-9);
  });

  it("stays continuous as a zoom-out passes through base scale", () => {
    let prev = poseToCss({ scale: 1.6, cx: 0.35, cy: 0.62 }, 0.034, 0.9);
    for (let i = 1; i <= 200; i++) {
      const S = 1.6 - (0.6 * i) / 200;
      const css = poseToCss({ scale: S, cx: 0.35, cy: 0.62 }, 0.034, 0.9);
      assert.ok(Number.isFinite(css.translateX) && Number.isFinite(css.translateY));
      assert.ok(
        Math.abs(css.translateX - prev.translateX) < 0.01,
        `translateX snapped at scale ${S}: ${prev.translateX} -> ${css.translateX}`,
      );
      assert.ok(
        Math.abs(css.translateY - prev.translateY) < 0.01,
        `translateY snapped at scale ${S}: ${prev.translateY} -> ${css.translateY}`,
      );
      prev = css;
    }
    assert.equal(prev.translateX, 0);
    assert.equal(prev.translateY, 0);
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
    assert.ok(out > inn, `${out} should be > ${inn}`);
    assert.ok(inn >= 0.55 && inn <= 1.25);
    assert.ok(tiny < inn);
  });
});
