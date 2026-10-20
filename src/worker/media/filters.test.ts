import { describe, expect, it } from "vitest";
import { atempoChain, ffmpegColor, fitFilter, gifFilter, joinFilter, speedArgs } from "./filters";

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
  const clip = { video: true, audio: true, durationMs: 2000 };
  const to = { width: 1920, height: 1080, fps: 60 };

  it("converts every file to the first file's size and frame rate", () => {
    const { filter } = joinFilter([clip, clip], to);
    expect(filter.match(/fps=60,/g)).toHaveLength(2);
    expect(filter).toContain("scale=1920:1080");
    expect(filter.endsWith("concat=n=2:v=1:a=1[v][a]")).toBe(true);
  });

  it("keeps the sound when one file has none, filling it with silence", () => {
    const { filter, audio } = joinFilter([clip, { ...clip, audio: false }], to);
    expect(audio).toBe(true);
    expect(filter).toContain("anullsrc=r=48000:cl=stereo,atrim=duration=2[a1]");
    expect(filter).toContain("[0:a]aresample=48000");
  });

  it("shows black under a sound-only file between videos", () => {
    const { filter } = joinFilter([{ ...clip, video: false, durationMs: 1500 }, clip], to);
    expect(filter).toContain("color=c=black:s=1920x1080:r=60:d=1.5,format=yuv420p[v0]");
  });

  it("joins sound files into sound only", () => {
    const sound = { video: false, audio: true, durationMs: 1000 };
    const join = joinFilter([sound, sound], to);
    expect(join).toMatchObject({ video: false, audio: true });
    expect(join.filter.endsWith("concat=n=2:v=0:a=1[a]")).toBe(true);
  });
});

describe("fitFilter", () => {
  it("adds bars, crops or stretches to the size", () => {
    expect(fitFilter({ mode: "pad", width: 640, height: 360, background: "white" })).toBe(
      "scale=640:360:force_original_aspect_ratio=decrease,pad=640:360:(ow-iw)/2:(oh-ih)/2:color=#ffffff,setsar=1",
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
    const part = { video: true, audio: false, durationMs: 1000 };
    expect(joinFilter([part, part], { width: 321, height: 241, fps: 30 }).filter).toContain(
      "scale=320:240",
    );
  });

  it("crops a sped-up video to an even size", () => {
    expect(speedArgs("in.mp4", "out.mp4", { rate: 2, fps: 30, sound: false })).toContain(
      "setpts=PTS/2,crop=trunc(iw/2)*2:trunc(ih/2)*2",
    );
  });
});

describe("bar colours", () => {
  it("writes any colour the picker makes as hex for the filter", () => {
    expect(ffmpegColor("rgb(255, 0, 0)")).toBe("#ff0000");
    expect(ffmpegColor("hsl(120, 100%, 50%)")).toBe("#00ff00");
    expect(ffmpegColor("oklch(0.628 0.258 29.2)")).toMatch(/^#f[a-f0-9]{5}$/);
    expect(ffmpegColor("#00000080")).toBe("#00000080");
    expect(ffmpegColor("black")).toBe("#000000");
  });

  it("puts the colour into the pad filter without commas or brackets", () => {
    const filter = fitFilter({ mode: "pad", width: 640, height: 360, background: "rgb(1, 2, 3)" });
    expect(filter).toContain(":color=#010203,");
  });
});
