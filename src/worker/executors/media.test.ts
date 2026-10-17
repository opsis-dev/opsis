import { describe, expect, it } from "vitest";
import { atempoChain, joinFilter, speedArgs } from "./media";

describe("atempoChain", () => {
  it.each([
    [64, Array.from({ length: 6 }, () => "atempo=2.0000").join(",")],
    [0.25, "atempo=0.5000,atempo=0.5000"],
    [3, "atempo=2.0000,atempo=1.5000"],
  ])("splits %f× into stages the filter accepts", (rate, chain) => {
    expect(atempoChain(rate)).toBe(chain);
  });
});

describe("speedArgs", () => {
  it("keeps the source frame rate instead of the 25 fps default", () => {
    const args = speedArgs("in.mp4", "out.mp4", { rate: 64, fps: 29.97003, sound: false });
    expect(args.slice(args.indexOf("-r"), args.indexOf("-r") + 2)).toEqual(["-r", "29.97"]);
    expect(args).toContain("-an");
  });
});

describe("joinFilter", () => {
  it("converts every file to the first file's size and frame rate", () => {
    const filter = joinFilter(2, { width: 1920, height: 1080, fps: 60, sound: true });
    expect(filter.match(/fps=60\[/g)).toHaveLength(2);
    expect(filter).toContain("scale=1920:1080");
    expect(filter.endsWith("concat=n=2:v=1:a=1[v][a]")).toBe(true);
  });
});
