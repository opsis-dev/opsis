import { describe, expect, it } from "vitest";
import { interpolate, renameVariable, stringsIn, tokenize } from "#core/vars";

describe("variables", () => {
  it("allows readable names with spaces", () => {
    expect(interpolate("{{Site address}}/login", { "Site address": "https://x.test" })).toBe(
      "https://x.test/login",
    );
  });

  it("leaves names that aren't variables alone", () => {
    expect(interpolate("{{constructor}} {{x}}", { x: "1" })).toBe("{{constructor}} 1");
  });

  it("splits text into words, variable chips and pauses", () => {
    expect(tokenize("Hi {{Product name}}¦ welcome", true)).toEqual([
      { kind: "text", text: "Hi " },
      { kind: "var", name: "Product name" },
      { kind: "pause" },
      { kind: "text", text: " welcome" },
    ]);
    expect(tokenize("a¦b")).toEqual([{ kind: "text", text: "a¦b" }]);
  });

  it("renames references everywhere in params", () => {
    expect(
      renameVariable(
        { url: "{{Site}}/x", list: ["{{Site}}", "{{Other}}"] },
        "Site",
        "Site address",
      ),
    ).toEqual({ url: "{{Site address}}/x", list: ["{{Site address}}", "{{Other}}"] });
  });
});

describe("stringsIn", () => {
  it("collects nested strings that pass the test", () => {
    const value = { a: "project:a.mp4", b: ["x", { c: "project:b.png" }], d: 3, e: null };
    expect(stringsIn(value, (text) => text.startsWith("project:"))).toEqual([
      "project:a.mp4",
      "project:b.png",
    ]);
  });
});
