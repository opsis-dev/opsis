import { describe, expect, it } from "vitest";
import {
  bleedLayout,
  captureScale,
  containBox,
  CUTOUT_FIT,
  cutoutLayout,
  maxScaleFor,
  windowBox,
  windowLayout,
} from "./still-layout";

const HD = { width: 1920, height: 1080 };

describe("containBox", () => {
  it("fits the recording's shape inside the frame", () => {
    expect(containBox({ width: 1080, height: 1920 }, 1.6)).toEqual({ width: 1080, height: 675 });
    expect(containBox({ width: 1920, height: 1080 }, 1.6)).toEqual({ width: 1728, height: 1080 });
  });
});

describe("windowBox", () => {
  it("is width-limited when the shot is wider than the canvas", () => {
    expect(windowBox(0.25, HD, 0.8)).toEqual({ width: 1536, height: 384 });
  });

  it("is height-limited when the shot is taller than the canvas", () => {
    expect(windowBox(2, HD, 0.8)).toEqual({ width: 432, height: 864 });
  });
});

describe("windowLayout", () => {
  it("keeps a wide shot's whole shape, centred", () => {
    const box = windowLayout({ width: 1200, height: 300, scale: 2 }, HD, 0.86);
    expect(box.width / box.height).toBeCloseTo(4, 1);
    expect(Math.abs(box.x + box.width / 2 - 960)).toBeLessThanOrEqual(1);
    expect(Math.abs(box.y + box.height / 2 - 540)).toBeLessThanOrEqual(1);
  });

  it("shows a slightly taller shot whole, narrower than the canvas", () => {
    const box = windowLayout({ width: 1440, height: 900, scale: 2 }, HD, 0.86);
    expect(box.height).toBe(Math.round(1080 * 0.86));
    expect(box.width / box.height).toBeCloseTo(1.6, 2);
  });

  it("gives a tall page the canvas shape, so it is cut only at the bottom", () => {
    const box = windowLayout({ width: 1920, height: 8000, scale: 1 }, HD, 0.86);
    expect(box.width).toBe(Math.round(1920 * 0.86));
    expect(box.height).toBe(Math.round(1080 * 0.86));
  });
});

describe("cutoutLayout", () => {
  const shot = {
    width: 348,
    height: 138,
    scale: 4,
    body: { x: 24, y: 24, width: 300, height: 90 },
  };

  it("fits the element, not its padding, to the cut-out size and centres it", () => {
    const { body, img } = cutoutLayout(shot, HD);
    expect(body.width).toBeCloseTo(1920 * CUTOUT_FIT, 0);
    expect(body.x + body.width / 2).toBeCloseTo(960, 0);
    expect(body.y + body.height / 2).toBeCloseTo(540, 0);
    expect(img.x).toBeLessThan(body.x);
    expect(img.width / img.height).toBeCloseTo(348 / 138, 3);
  });

  it("never stretches the picture more than one and a half times", () => {
    const small = cutoutLayout({ ...shot, scale: 1 }, HD);
    expect(small.body.width).toBeCloseTo(300 * 1.5, 3);
    expect(small.stretched).toBe(true);
    expect(cutoutLayout({ ...shot, scale: 8 }, HD).stretched).toBe(false);
  });

  it("treats a shot without an element as all body", () => {
    const { body, img } = cutoutLayout({ width: 1600, height: 900, scale: 2 }, HD);
    expect(body).toEqual(img);
  });
});

describe("bleedLayout", () => {
  it("covers the canvas and keeps the top of a tall page", () => {
    const box = bleedLayout({ width: 1440, height: 6000, scale: 2 }, HD);
    expect(box.width).toBeCloseTo(1920, 3);
    expect(box.y).toBe(0);
    expect(box.height).toBeGreaterThan(1080);
  });

  it("covers the height of a wide shot, centred across", () => {
    const box = bleedLayout({ width: 4000, height: 1000, scale: 1 }, HD);
    expect(box.height).toBeCloseTo(1080, 3);
    expect(box.x + box.width / 2).toBeCloseTo(960, 3);
  });
});

describe("captureScale", () => {
  const canvas = { width: 3840, height: 2160 };

  it("asks for enough pixels to draw the shot at the largest export", () => {
    expect(
      captureScale("window", { width: 1920, height: 1080 }, canvas, { fit: 0.86, least: 1 }),
    ).toBe(2);
    expect(
      captureScale("cutout", { width: 348, height: 138 }, canvas, {
        body: { x: 24, y: 24, width: 300, height: 90 },
        fit: 0.86,
        least: 1,
      }),
    ).toBe(8);
  });

  it("never goes below the page's own pixel ratio", () => {
    expect(captureScale("window", { width: 1920, height: 1080 }, HD, { fit: 0.86, least: 2 })).toBe(
      2,
    );
  });

  it("caps the surface for very large regions", () => {
    expect(maxScaleFor({ width: 1920, height: 1080 })).toBe(4);
    expect(
      captureScale("bleed", { width: 1920, height: 20_000 }, canvas, { fit: 0.86, least: 2 }),
    ).toBe(1);
  });
});
