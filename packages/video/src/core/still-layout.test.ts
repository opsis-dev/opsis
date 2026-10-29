import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import {
  captureScale,
  containBox,
  paddedRegion,
  stillLayout,
  tallestShot,
  type Box,
  type Shot,
} from "#video/core/still-layout.ts";

const HD = { width: 1920, height: 1080 };
const CANVASES = [
  HD,
  { width: 1080, height: 1920 },
  { width: 1080, height: 1080 },
  { width: 3840, height: 2160 },
];

const size = (min: number, max: number) =>
  fc.record({ width: fc.integer({ min, max }), height: fc.integer({ min, max }) });

const canvas = fc.constantFrom(...CANVASES);

const anyShot = fc
  .tuple(size(50, 6000), fc.constantFrom(1, 1.5, 2, 3, 4, 8))
  .map(([s, scale]): Shot => ({ ...s, scale }));

const bodyShot = anyShot.chain((s) =>
  fc
    .record({
      x: fc.integer({ min: 0, max: Math.floor(s.width / 3) }),
      y: fc.integer({ min: 0, max: Math.floor(s.height / 3) }),
    })
    .chain(({ x, y }) =>
      fc
        .record({
          width: fc.integer({ min: 1, max: s.width - x }),
          height: fc.integer({ min: 1, max: s.height - y }),
        })
        .map((wh) => ({ ...s, body: { x, y, ...wh } })),
    ),
);

const aspect = (s: { width: number; height: number }) => s.width / s.height;

const elementBox = (shot: Shot & { body: Box }, img: Box): Box => {
  const s = img.width / shot.width;
  return {
    x: img.x + shot.body.x * s,
    y: img.y + shot.body.y * s,
    width: shot.body.width * s,
    height: shot.body.height * s,
  };
};

describe("containBox", () => {
  it("fits the shape inside the frame, touching two of its sides", () => {
    fc.assert(
      fc.property(size(1, 5000), fc.double({ min: 0.1, max: 10, noNaN: true }), (outer, ratio) => {
        const box = containBox(outer, ratio);
        expect(box.width).toBeLessThanOrEqual(outer.width + 1e-6);
        expect(box.height).toBeLessThanOrEqual(outer.height + 1e-6);
        expect(aspect(box)).toBeCloseTo(ratio, 6);
        expect(
          Math.abs(box.width - outer.width) < 1e-6 || Math.abs(box.height - outer.height) < 1e-6,
        ).toBe(true);
      }),
    );
  });
});

describe("window look", () => {
  it("centres the shot inside the fitted canvas in its own shape, or the canvas's when tall", () => {
    fc.assert(
      fc.property(anyShot, canvas, fc.double({ min: 0.5, max: 1, noNaN: true }), (s, c, fit) => {
        const box = stillLayout("window", s, c, fit);
        expect(box.width).toBeLessThanOrEqual(Math.round(c.width * fit) + 1);
        expect(box.height).toBeLessThanOrEqual(Math.round(c.height * fit) + 1);
        expect(Math.abs(box.x + box.width / 2 - c.width / 2)).toBeLessThanOrEqual(1);
        expect(Math.abs(box.y + box.height / 2 - c.height / 2)).toBeLessThanOrEqual(1);
        const tall = aspect(s) * 1.5 < aspect(c);
        const want = tall ? aspect(c) : aspect(s);
        expect(Math.abs(aspect(box) - want) / want).toBeLessThan(
          2 / Math.min(box.width, box.height),
        );
      }),
    );
  });
});

describe("cutout look", () => {
  it("centres the element at most four fifths of the canvas wide and never stretches it past one and a half times its pixels", () => {
    fc.assert(
      fc.property(bodyShot, canvas, (s, c) => {
        const img = stillLayout("cutout", s, c, 0.86);
        const body = elementBox(s, img);
        const rounding = 1 + Math.max(body.width / body.height, body.height / body.width) / 2;
        expect(body.width).toBeLessThanOrEqual(c.width * 0.8 + rounding);
        expect(body.height).toBeLessThanOrEqual(c.height * 0.8 + rounding);
        expect(img.width / s.width).toBeLessThanOrEqual(s.scale * 1.5 + 1e-9);
        expect(Math.abs(body.x + body.width / 2 - c.width / 2)).toBeLessThanOrEqual(1);
        expect(Math.abs(body.y + body.height / 2 - c.height / 2)).toBeLessThanOrEqual(1);
        expect(img.width / img.height).toBeCloseTo(aspect(s), 6);
      }),
    );
  });
});

describe("bleed look", () => {
  it("covers the canvas, keeps the top and centres across", () => {
    fc.assert(
      fc.property(anyShot, canvas, (s, c) => {
        const box = stillLayout("bleed", s, c, 0.86);
        expect(box.width).toBeGreaterThanOrEqual(c.width - 1e-6);
        expect(box.height).toBeGreaterThanOrEqual(c.height - 1e-6);
        expect(box.y).toBe(0);
        expect(Math.abs(box.x + box.width / 2 - c.width / 2)).toBeLessThanOrEqual(1);
      }),
    );
  });
});

describe("captureScale", () => {
  it("asks for enough pixels for the largest export, at least the page's ratio, within the surface budget", () => {
    fc.assert(
      fc.property(
        fc.constantFrom("window" as const, "cutout" as const, "bleed" as const),
        size(50, 20_000),
        canvas,
        fc.constantFrom(1, 1.5, 2, 3),
        (look, region, c, least) => {
          const scale = captureScale(look, region, c, { fit: 0.86, least });
          expect(scale).toBeGreaterThanOrEqual(1);
          expect(scale).toBeLessThanOrEqual(8);
          const budget = 1920 * 1080 * 16;
          const area = region.width * region.height;
          expect(area * scale ** 2 <= budget + 1e-6 || scale === 1).toBe(true);
          const drawn = stillLayout(look, { ...region, scale: Number.POSITIVE_INFINITY }, c, 0.86);
          const enough = scale >= Math.max(least, drawn.width / region.width) - 1e-9;
          const atBudget = area * (scale + 0.1) ** 2 > budget;
          expect(enough || atBudget || scale === 8).toBe(true);
        },
      ),
    );
  });
});

describe("tallestShot", () => {
  it("keeps the tallest page a window or full bleed shows whole, and a cut-out four times its width", () => {
    fc.assert(
      fc.property(fc.integer({ min: 100, max: 4000 }), canvas, (width, c) => {
        const tallest = tallestShot("window", width, c);
        expect(aspect({ width, height: tallest }) * 1.5).toBeLessThanOrEqual(aspect(c) + 1e-9);
        expect(aspect({ width, height: tallest - 1 }) * 1.5).toBeGreaterThanOrEqual(
          aspect(c) - 1e-9,
        );
        expect(tallestShot("bleed", width, c)).toBe(tallest);
        expect(tallestShot("cutout", width, c)).toBe(width * 4);
      }),
    );
  });
});

describe("paddedRegion", () => {
  it("adds the space around an element in page coordinates and stops at the top and left edges", () => {
    fc.assert(
      fc.property(
        fc.record({
          x: fc.integer({ min: -100, max: 2000 }),
          y: fc.integer({ min: -100, max: 2000 }),
          width: fc.integer({ min: 1, max: 1000 }),
          height: fc.integer({ min: 1, max: 1000 }),
        }),
        fc.record({ x: fc.integer({ min: 0, max: 500 }), y: fc.integer({ min: 0, max: 5000 }) }),
        fc.integer({ min: 0, max: 80 }),
        (box, scroll, padding) => {
          const { region, body } = paddedRegion(box, scroll, padding);
          const left = box.x + scroll.x;
          const top = box.y + scroll.y;
          expect(region.x).toBe(Math.max(0, left - padding));
          expect(region.y).toBe(Math.max(0, top - padding));
          expect(region.x + body.x).toBe(left);
          expect(region.y + body.y).toBe(top);
          expect({ width: body.width, height: body.height }).toEqual({
            width: box.width,
            height: box.height,
          });
          expect(region.x + region.width).toBe(left + box.width + padding);
          expect(region.y + region.height).toBe(top + box.height + padding);
        },
      ),
    );
  });
});
