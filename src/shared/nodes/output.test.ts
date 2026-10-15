import { describe, expect, it } from "vitest";
import { variantSize } from "./output";

describe("variantSize", () => {
  it("keeps the long edge and even dimensions", () => {
    const s = { width: 1920, height: 1080 };
    expect(variantSize("16:9", s)).toEqual({ width: 1920, height: 1080 });
    expect(variantSize("9:16", s)).toEqual({ width: 1080, height: 1920 });
    expect(variantSize("1:1", s)).toEqual({ width: 1920, height: 1920 });
    expect(variantSize("4:5", s)).toEqual({ width: 1536, height: 1920 });
  });
});
