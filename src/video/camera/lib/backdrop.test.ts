import { existsSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { BACKDROPS, DEFAULT_BACKDROP } from "../../../shared/backdrops";
import { backdropSource, isLightBackdrop } from "./backdrop";

const PUBLIC_DIR = path.resolve(import.meta.dirname, "../../../../public");

const fileOf = (backdrop?: string) => {
  const source = backdropSource(backdrop);
  return source.kind === "file" ? source.path : null;
};

describe("backdropSource", () => {
  it("falls back to the default when unset or blank", () => {
    expect(fileOf()).toBe(`backdrops/${DEFAULT_BACKDROP}.jpg`);
    expect(fileOf("  ")).toBe(`backdrops/${DEFAULT_BACKDROP}.jpg`);
  });

  it("takes anything with an extension as your own file, so custom images need no code", () => {
    expect(fileOf("mine.jpg")).toBe("backdrops/mine.jpg");
    expect(fileOf("mine.png")).toBe("backdrops/mine.png");
  });

  it.each(BACKDROPS)("finds the shipped %s image", (b) => {
    expect(existsSync(path.join(PUBLIC_DIR, fileOf(b) ?? ""))).toBe(true);
  });

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

  it("paints named colours instead of looking for an image of that name", () => {
    expect(backdropSource("red")).toEqual({ kind: "paint", css: "red" });
    expect(backdropSource("white")).toEqual({ kind: "paint", css: "white" });
    expect(isLightBackdrop("white")).toBe(true);
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
    expect(isLightBackdrop("#faf6f1")).toBe(true);
    expect(isLightBackdrop("#fff")).toBe(true);
    expect(isLightBackdrop("#0b0b10")).toBe(false);
    expect(isLightBackdrop("rgb(250, 250, 250)")).toBe(true);
    expect(isLightBackdrop("rgba(250 250 250 / 0.9)")).toBe(true);
    expect(isLightBackdrop("rgb(20, 20, 30)")).toBe(false);
    expect(isLightBackdrop(" #ffffff ")).toBe(true);
    expect(isLightBackdrop(undefined)).toBe(false);
  });
});
