import { describe, expect, it } from "vitest";
import { safeName } from "#core/safe-name";

describe("safeName", () => {
  it("replaces unsafe runs, trims edge dashes and falls back", () => {
    expect(safeName("  My Font_v2.1 ", { fallback: "x" })).toBe("my-font_v2.1");
    expect(safeName("My Font_v2.1", { fallback: "x", alphabet: "alnum" })).toBe("my-font-v2-1");
    expect(safeName(" Login! ", { fallback: "x", lower: false })).toBe("Login");
    expect(safeName("***", { fallback: "x" })).toBe("x");
  });

  it("never names the folder itself, its parent or a hidden file", () => {
    expect(safeName(".", { fallback: "x" })).toBe("x");
    expect(safeName("..", { fallback: "x" })).toBe("x");
    expect(safeName("../../etc", { fallback: "x" })).toBe("etc");
    expect(safeName(".hidden", { fallback: "x" })).toBe("hidden");
    expect(safeName("v2.1.", { fallback: "x" })).toBe("v2.1");
  });
});
