import { describe, expect, it } from "vitest";
import { atempoChain, fitFilter, gifFilter, joinFilter, speedArgs } from "./filters";

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

describe("fitFilter", () => {
  it("adds bars, crops or stretches to the size", () => {
    expect(fitFilter({ mode: "pad", width: 640, height: 360, background: "white" })).toBe(
      "scale=640:360:force_original_aspect_ratio=decrease,pad=640:360:(ow-iw)/2:(oh-ih)/2:color=white,setsar=1",
    );
    expect(fitFilter({ mode: "pad", width: 640, height: 360, background: "" })).toBe(
      "scale=640:360:force_original_aspect_ratio=decrease,pad=640:360:(ow-iw)/2:(oh-ih)/2,setsar=1",
    );
    expect(fitFilter({ mode: "crop", width: 640, height: 360, background: "black" })).toBe(
      "scale=640:360:force_original_aspect_ratio=increase,crop=640:360,setsar=1",
    );
    expect(fitFilter({ mode: "stretch", width: 640, height: 360, background: "black" })).toBe(
      "scale=640:360,setsar=1",
    );
  });
});

describe("gifFilter", () => {
  it("makes one palette for the whole animation", () => {
    expect(gifFilter({ fps: 15, width: 960, dither: "bayer" })).toBe(
      "fps=15,scale=960:-1:flags=lanczos,split[a][b];[a]palettegen=stats_mode=diff[pal];[b][pal]paletteuse=dither=bayer",
    );
  });
});

describe("odd sizes", () => {
  it("joins files at an even size, which the video encoder needs", () => {
    expect(joinFilter(2, { width: 321, height: 241, fps: 30, sound: false })).toContain(
      "scale=320:240",
    );
  });

  it("crops a sped-up video to an even size", () => {
    expect(speedArgs("in.mp4", "out.mp4", { rate: 2, fps: 30, sound: false })).toContain(
      "setpts=PTS/2,crop=trunc(iw/2)*2:trunc(ih/2)*2",
    );
  });
});
