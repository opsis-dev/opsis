import { describe, expect, it } from "vitest";
import { anyPortChoices, cutChoices } from "./string-field";
import type { FormContext } from "./types";

const ctx = {
  ports: [
    { port: "in1", label: "Clip 1" },
    { port: "in2", label: "Clip 2" },
    { port: "in9", label: "Clip 9" },
  ],
} as FormContext;

describe("port choices", () => {
  it("offers a cut before every connected clip but the first", () => {
    const choices = cutChoices(ctx, "in2");
    expect(choices.options).toEqual(["in2", "in9"]);
    expect(choices.format?.("in9")).toBe("Clip 9");
  });

  it("keeps a port that is no longer connected and says so", () => {
    const choices = cutChoices(ctx, "in4");
    expect(choices.options).toContain("in4");
    expect(choices.format?.("in4")).toBe("Not connected");
  });

  it("offers every connected port for a scene setting", () => {
    expect(anyPortChoices(ctx, "in1").options).toEqual(["in1", "in2", "in9"]);
  });
});
