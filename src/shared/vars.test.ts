import { describe, expect, it } from "vitest";
import { interpolate, renameVariable, tokenize, untokenize } from "./vars";

describe("variables", () => {
  it("allows readable names with spaces", () => {
    expect(interpolate("{{Site address}}/login", { "Site address": "https://x.test" })).toBe(
      "https://x.test/login",
    );
  });

  it("splits text into words, variable chips and pauses and back", () => {
    const value = "Hi {{Product name}}¦ welcome";
    const tokens = tokenize(value, true);
    expect(tokens).toEqual([
      { kind: "text", text: "Hi " },
      { kind: "var", name: "Product name" },
      { kind: "pause" },
      { kind: "text", text: " welcome" },
    ]);
    expect(untokenize(tokens)).toBe(value);
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
