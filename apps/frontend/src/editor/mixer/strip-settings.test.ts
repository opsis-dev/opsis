import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { mixerBook } from "@/store/catalog";
import { stripSettings } from "./strip-settings";

const SETTINGS = ["comp", "eq", "fader", "highpass", "pan", "trim"];

describe("strip settings", () => {
  it("copies a strip's settings unchanged and never its routing", () => {
    const plain = mixerBook().defaultStrip;
    fc.assert(
      fc.property(
        fc.double({ min: -24, max: 24, noNaN: true }),
        fc.double({ min: -1, max: 1, noNaN: true }),
        fc.boolean(),
        fc.boolean(),
        fc.string(),
        (trim, pan, mute, solo, bus) => {
          const strip = { ...plain, trim, pan, mute, solo, bus };
          const copied = stripSettings(strip);
          expect(Object.keys(copied).toSorted()).toEqual(SETTINGS);
          for (const key of SETTINGS)
            expect(copied).toHaveProperty(key, strip[key as keyof typeof strip]);
        },
      ),
    );
  });
});
