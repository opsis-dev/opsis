import { describe, expect, it } from "vitest";
import {
  captureScale,
  containBox,
  paddedRegion,
  stillLayout,
  tallestShot,
  type Box,
  type Shot,
} from "./still-layout";

const HD = { width: 1920, height: 1080 };

describe("containBox", () => {
  it("fits the recording's shape inside the frame", () => {
    expect(containBox({ width: 1080, height: 1920 }, 1.6)).toEqual({ width: 1080, height: 675 });
    expect(containBox({ width: 1920, height: 1080 }, 1.6)).toEqual({ width: 1728, height: 1080 });
  });
});

const elementBox = (shot: Shot & { body: Box }, img: Box): Box => {
  const s = img.width / shot.width;
  return {
    x: img.x + shot.body.x * s,
    y: img.y + shot.body.y * s,
    width: shot.body.width * s,
    height: shot.body.height * s,
  };
};

describe("window look", () => {
  it("is width-limited when the shot is wider than the canvas", () => {
    const box = stillLayout("window", { width: 400, height: 100, scale: 1 }, HD, 0.8);
    expect(box).toMatchObject({ width: 1536, height: 384 });
  });

  it("keeps a wide shot's whole shape, centred", () => {
    const box = stillLayout("window", { width: 1200, height: 300, scale: 2 }, HD, 0.86);
    expect(box.width / box.height).toBeCloseTo(4, 1);
    expect(Math.abs(box.x + box.width / 2 - 960)).toBeLessThanOrEqual(1);
    expect(Math.abs(box.y + box.height / 2 - 540)).toBeLessThanOrEqual(1);
  });

  it("shows a slightly taller shot whole, narrower than the canvas", () => {
    const box = stillLayout("window", { width: 1440, height: 900, scale: 2 }, HD, 0.86);
    expect(box.height).toBe(Math.round(1080 * 0.86));
    expect(box.width / box.height).toBeCloseTo(1.6, 2);
  });

  it("gives a tall page the canvas shape, so it is cut only at the bottom", () => {
    const box = stillLayout("window", { width: 1920, height: 8000, scale: 1 }, HD, 0.86);
    expect(box.width).toBe(Math.round(1920 * 0.86));
    expect(box.height).toBe(Math.round(1080 * 0.86));
  });
});

describe("cutout look", () => {
  const shot = {
    width: 348,
    height: 138,
    scale: 4,
    body: { x: 24, y: 24, width: 300, height: 90 },
  };

  it("fits the element, not its padding, to the cut-out size and centres it", () => {
    const img = stillLayout("cutout", shot, HD, 0.86);
    const body = elementBox(shot, img);
    expect(body.width).toBeCloseTo(1920 * 0.8, 0);
    expect(body.x + body.width / 2).toBeCloseTo(960, 0);
    expect(body.y + body.height / 2).toBeCloseTo(540, 0);
    expect(img.x).toBeLessThan(body.x);
    expect(img.width / img.height).toBeCloseTo(348 / 138, 3);
  });

  it("is height-limited when the element is taller than the canvas", () => {
    const img = stillLayout("cutout", { width: 100, height: 200, scale: 100 }, HD, 0.86);
    expect(img).toEqual({ x: 744, y: 108, width: 432, height: 864 });
  });

  it("never stretches the picture more than one and a half times", () => {
    const small = stillLayout("cutout", { ...shot, scale: 1 }, HD, 0.86);
    expect(elementBox(shot, small).width).toBeCloseTo(300 * 1.5, 3);
    const sharp = stillLayout("cutout", { ...shot, scale: 8 }, HD, 0.86);
    expect(sharp.width / shot.width).toBeLessThan(8);
  });

  it("treats a shot without an element as all body", () => {
    const img = stillLayout("cutout", { width: 1600, height: 900, scale: 2 }, HD, 0.86);
    expect(img.width).toBeCloseTo(1920 * 0.8, 3);
    expect(img.x + img.width / 2).toBeCloseTo(960, 3);
    expect(img.y + img.height / 2).toBeCloseTo(540, 3);
  });
});

describe("bleed look", () => {
  it("covers the canvas and keeps the top of a tall page", () => {
    const box = stillLayout("bleed", { width: 1440, height: 6000, scale: 2 }, HD, 0.86);
    expect(box.width).toBeCloseTo(1920, 3);
    expect(box.y).toBe(0);
    expect(box.height).toBeGreaterThan(1080);
  });

  it("covers the height of a wide shot, centred across", () => {
    const box = stillLayout("bleed", { width: 4000, height: 1000, scale: 1 }, HD, 0.86);
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
    const huge = { width: 20_000, height: 11_250 };
    expect(captureScale("window", HD, huge, { fit: 1, least: 1 })).toBe(4);
    expect(
      captureScale("bleed", { width: 1920, height: 20_000 }, canvas, { fit: 0.86, least: 2 }),
    ).toBe(1);
  });
});

describe("tallestShot", () => {
  const canvas = { width: 3840, height: 2160 };

  it("keeps as much of a tall page as a window or full bleed can show", () => {
    expect(tallestShot("window", 1920, canvas)).toBe(1620);
    expect(tallestShot("bleed", 1920, canvas)).toBe(1620);
  });

  it("lets a cut-out keep a page four times taller than wide", () => {
    expect(tallestShot("cutout", 1920, canvas)).toBe(7680);
  });
});

describe("paddedRegion", () => {
  it("adds the space around an element in page coordinates", () => {
    const box = { x: 100, y: 50, width: 200, height: 40 };
    expect(paddedRegion(box, { x: 0, y: 300 }, 24)).toEqual({
      region: { x: 76, y: 326, width: 248, height: 88 },
      body: { x: 24, y: 24, width: 200, height: 40 },
    });
  });

  it("stops the space at the page's top and left edges", () => {
    const box = { x: 10, y: 5, width: 200, height: 40 };
    expect(paddedRegion(box, { x: 0, y: 0 }, 24)).toEqual({
      region: { x: 0, y: 0, width: 234, height: 69 },
      body: { x: 10, y: 5, width: 200, height: 40 },
    });
  });
});
