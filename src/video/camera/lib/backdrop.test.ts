import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { BACKDROPS } from "../../../shared/backdrops";
import { DEFAULT_BACKDROP, backdropFile, backdropSource, isLightBackdrop } from "./backdrop";

describe("backdropFile", () => {
  it("resolves a bare name to the shipped set", () => {
    assert.equal(backdropFile("cobalt"), "backdrops/cobalt.jpg");
  });

  it("falls back to the default when unset or blank", () => {
    assert.equal(backdropFile(), `backdrops/${DEFAULT_BACKDROP}.jpg`);
    assert.equal(backdropFile("  "), `backdrops/${DEFAULT_BACKDROP}.jpg`);
  });

  it("takes anything with an extension as your own file, so custom images need no code", () => {
    assert.equal(backdropFile("mine.jpg"), "backdrops/mine.jpg");
    assert.equal(backdropFile("mine.png"), "backdrops/mine.png");
  });

  it("resolves every shipped backdrop", () => {
    for (const b of BACKDROPS) assert.equal(backdropFile(b), `backdrops/${b}.jpg`);
  });
});

describe("backdropSource", () => {
  it("paints colours and gradients, loads addresses, and finds names in the shipped set", () => {
    assert.deepEqual(backdropSource("#123456"), { kind: "paint", css: "#123456" });
    assert.deepEqual(backdropSource("linear-gradient(red, blue)"), {
      kind: "paint",
      css: "linear-gradient(red, blue)",
    });
    assert.deepEqual(backdropSource("https://x.test/y.jpg"), {
      kind: "image",
      url: "https://x.test/y.jpg",
    });
    assert.deepEqual(backdropSource("cobalt"), { kind: "file", path: "backdrops/cobalt.jpg" });
    assert.deepEqual(backdropSource(undefined), {
      kind: "file",
      path: `backdrops/${DEFAULT_BACKDROP}.jpg`,
    });
  });
});

describe("isLightBackdrop", () => {
  it("flags the light ones, because a white rim cannot separate on them", () => {
    assert.equal(isLightBackdrop("canyon"), true);
    assert.equal(isLightBackdrop("glaze"), false);
    assert.equal(isLightBackdrop("unknown"), false);
  });

  it("reads light and dark colours, in short and long hex", () => {
    assert.equal(isLightBackdrop("#faf6f1"), true);
    assert.equal(isLightBackdrop("#fff"), true);
    assert.equal(isLightBackdrop("#0b0b10"), false);
    assert.equal(isLightBackdrop("rgb(250, 250, 250)"), false);
  });
});
