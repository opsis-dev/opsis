import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { ico } from "./ico";

const image = fc.record({
  size: fc.integer({ min: 1, max: 256 }),
  png: fc.uint8Array({ minLength: 1, maxLength: 64 }),
});

describe("an icon file", () => {
  it("points each entry at its own image and names its side", () => {
    fc.assert(
      fc.property(fc.array(image, { minLength: 1, maxLength: 8 }), (images) => {
        const file = ico(images);
        const view = new DataView(file.buffer);
        expect(view.getUint16(2, true)).toBe(1);
        expect(view.getUint16(4, true)).toBe(images.length);
        images.forEach(({ size, png }, index) => {
          const entry = 6 + 16 * index;
          expect(view.getUint8(entry)).toBe(size >= 256 ? 0 : size);
          const length = view.getUint32(entry + 8, true);
          const offset = view.getUint32(entry + 12, true);
          expect(file.slice(offset, offset + length)).toEqual(png);
        });
      }),
    );
  });
});
