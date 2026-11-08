import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { cargoProfileVariables } from "./release";

const variableName = fc.stringMatching(/^[A-Z][A-Z0-9_]{0,20}$/);

describe("the release build's container", () => {
  it("receives every cargo profile setting and nothing else from the environment", () => {
    fc.assert(
      fc.property(
        fc.dictionary(variableName, fc.string()),
        fc.dictionary(variableName, fc.string()),
        (others, profile) => {
          const profiled = Object.fromEntries(
            Object.entries(profile).map(([name, value]) => [`CARGO_PROFILE_${name}`, value]),
          );
          const plain = Object.fromEntries(
            Object.entries(others).filter(([name]) => !name.startsWith("CARGO_PROFILE_")),
          );
          expect(cargoProfileVariables({ ...plain, ...profiled })).toEqual(profiled);
        },
      ),
    );
  });
});
