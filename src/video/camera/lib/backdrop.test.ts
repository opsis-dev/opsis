import { existsSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { BACKDROPS, DEFAULT_BACKDROP } from "../../../shared/backdrops";
import { backdropFile, backdropSource, isLightBackdrop } from "./backdrop";
import { STAGE_GROUND } from "./style";

const PUBLIC_DIR = path.resolve(import.meta.dirname, "../../../../public");

describe("backdropFile", () => {
  it("resolves a bare name to the shipped set", () => {
    expect(backdropFile("cobalt")).toBe("backdrops/cobalt.jpg");
  });

  it("falls back to the default when unset or blank", () => {
    expect(backdropFile()).toBe(`backdrops/${DEFAULT_BACKDROP}.jpg`);
    expect(backdropFile("  ")).toBe(`backdrops/${DEFAULT_BACKDROP}.jpg`);
  });

  it("takes anything with an extension as your own file, so custom images need no code", () => {
    expect(backdropFile("mine.jpg")).toBe("backdrops/mine.jpg");
    expect(backdropFile("mine.png")).toBe("backdrops/mine.png");
  });

  it.each(BACKDROPS)("finds the shipped %s image", (b) => {
    expect(existsSync(path.join(PUBLIC_DIR, backdropFile(b)))).toBe(true);
  });
});

describe("backdropSource", () => {
  it("paints colours and gradients, loads addresses, and finds names in the shipped set", () => {
    expect(backdropSource("#123456")).toEqual({ kind: "paint", css: "#123456" });
    expect(backdropSource("linear-gradient(red, blue)")).toEqual({
      kind: "paint",
      css: "linear-gradient(red, blue)",
    });
    expect(backdropSource("https://x.test/y.jpg")).toEqual({
      kind: "image",
      url: "https://x.test/y.jpg",
    });
    expect(backdropSource("cobalt")).toEqual({ kind: "file", path: "backdrops/cobalt.jpg" });
    expect(backdropSource(undefined)).toEqual({
      kind: "file",
      path: `backdrops/${DEFAULT_BACKDROP}.jpg`,
    });
  });

  it("ignores spaces around a colour or an address", () => {
    expect(backdropSource(" #123456 ")).toEqual({ kind: "paint", css: "#123456" });
    expect(backdropSource(" https://x.test/y.jpg")).toEqual({
      kind: "image",
      url: "https://x.test/y.jpg",
    });
    expect(backdropSource(" cobalt ")).toEqual({ kind: "file", path: "backdrops/cobalt.jpg" });
  });
});

describe("isLightBackdrop", () => {
  it("flags the light ones, because a white rim cannot separate on them", () => {
    expect(isLightBackdrop("canyon")).toBe(true);
    expect(isLightBackdrop("glaze")).toBe(false);
    expect(isLightBackdrop("unknown")).toBe(false);
  });

  it("reads light and dark colours, in short and long hex and rgb()", () => {
    expect(isLightBackdrop(STAGE_GROUND)).toBe(true);
    expect(isLightBackdrop("#fff")).toBe(true);
    expect(isLightBackdrop("#0b0b10")).toBe(false);
    expect(isLightBackdrop("rgb(250, 250, 250)")).toBe(true);
    expect(isLightBackdrop("rgba(250 250 250 / 0.9)")).toBe(true);
    expect(isLightBackdrop("rgb(20, 20, 30)")).toBe(false);
  });
});

describe("isLightBackdrop with padding", () => {
  it("reads a colour with spaces around it", () => {
    expect(isLightBackdrop(" #ffffff ")).toBe(true);
    expect(isLightBackdrop(" #101013 ")).toBe(false);
  });
});
