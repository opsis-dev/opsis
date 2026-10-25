import { defaultParams } from "@opsis/core/nodes/registry";
import { describe, expect, it } from "vitest";
import { filterPath, gradeFilters, lookKindOf, type GradeSize } from "./grade";

const video: GradeSize = { width: 1920, height: 1080, still: false };
const still: GradeSize = { ...video, still: true };
const EVEN = "scale=trunc(iw/2)*2:trunc(ih/2)*2";

const entry = (type: string, params: Record<string, unknown> = {}, nodeId = type) => ({
  nodeId,
  type,
  params: { ...defaultParams(type), ...params },
});

const chainOf = (
  entries: ReturnType<typeof entry>[],
  size: GradeSize = video,
  resolve: (file: string) => string = (file) => `/project/${file}`,
) => gradeFilters(entries, size, resolve);

describe("grade filter chain", () => {
  it("copies the picture when nothing changes", () => {
    expect(chainOf([]).chain).toBe(`[0:v]${EVEN}[out]`);
    expect(chainOf([entry("grade.colour")]).chain).toBe(`[0:v]${EVEN}[out]`);
    expect(chainOf([entry("grade.colour")], still).chain).toBe("[0:v]null[out]");
  });

  it("writes each colour setting as its own filter, in a fixed order", () => {
    const { chain } = chainOf([
      entry("grade.colour", {
        curve: "vintage",
        hue: 20,
        vibrance: 0.5,
        warmth: 1,
        gamma: 1.2,
        saturation: 1.4,
        contrast: 1.3,
        brightness: 0.1,
        exposure: -0.5,
      }),
    ]);
    expect(chain).toBe(
      `[0:v]exposure=exposure=-0.5,eq=brightness=0.1:contrast=1.3:saturation=1.4:gamma=1.2,colortemperature=temperature=3000:mix=1,vibrance=intensity=0.5,hue=h=20,curves=preset=vintage,${EVEN}[out]`,
    );
  });

  it("puts only the changed settings in the eq filter", () => {
    expect(chainOf([entry("grade.colour", { contrast: 1.2 })]).chain).toBe(
      `[0:v]eq=contrast=1.2,${EVEN}[out]`,
    );
  });

  it("maps cool warmth to a higher colour temperature", () => {
    expect(chainOf([entry("grade.colour", { warmth: -1 })]).chain).toContain("temperature=10000");
  });

  it("tints tones by how far the colour is from grey", () => {
    const { chain } = chainOf([
      entry("grade.colour", { shadows: "#ff0000", shadowsAmount: 1 }),
      entry("grade.colour", { highlights: "#808080", highlightsAmount: 1 }),
      entry("grade.colour", { midtones: "", midtonesAmount: 1 }),
    ]);
    expect(chain).toBe(`[0:v]colorbalance=rs=0.667:gs=-0.333:bs=-0.333,${EVEN}[out]`);
  });

  it("keeps tone offsets inside the filter's range", () => {
    const [, offsets] = chainOf([
      entry("grade.colour", { shadows: "#ff0000", shadowsAmount: 1 }),
    ]).chain.match(/rs=(-?[\d.]+)/) ?? ["", "9"];
    expect(Math.abs(Number(offsets))).toBeLessThanOrEqual(1);
  });

  it("runs the steps in the order of the chain", () => {
    const { chain } = chainOf([
      entry("grade.texture", { vignette: 0.5 }),
      entry("grade.colour", { contrast: 1.2 }),
    ]);
    expect(chain.indexOf("vignette=")).toBeLessThan(chain.indexOf("eq="));
  });

  it("orders a texture step as sharpen, soften, dark edges, then grain", () => {
    const { chain } = chainOf([
      entry("grade.texture", {
        grain: 0.5,
        vignette: 0.5,
        vignetteX: 0.25,
        vignetteY: 0.75,
        soften: 0.5,
        sharpen: 0.5,
      }),
    ]);
    expect(chain).toBe(
      `[0:v]cas=strength=0.5,gblur=sigma=4,vignette=angle=0.785:x0=w*0.25:y0=h*0.75,noise=alls=20:allf=t+u,${EVEN}[out]`,
    );
  });

  it("sizes blur and fringing by the picture's width", () => {
    const small: GradeSize = { width: 960, height: 540, still: false };
    expect(chainOf([entry("grade.texture", { soften: 1 })], small).chain).toContain("sigma=4");
    expect(chainOf([entry("grade.lens", { fringe: 1 })], small).chain).toContain("rh=4:bh=-4");
    expect(chainOf([entry("grade.lens", { fringe: 1 })]).chain).toContain("rh=8:bh=-8");
  });

  it("never rounds a small fringe down to nothing", () => {
    expect(chainOf([entry("grade.lens", { fringe: 0.01 })]).chain).toContain("rh=1:bh=-1");
  });

  it("writes lens settings in order", () => {
    expect(chainOf([entry("grade.lens", { trails: 0.5, bulge: -1, fringe: 0.5 })]).chain).toBe(
      `[0:v]rgbashift=rh=4:bh=-4,lenscorrection=k1=-0.5:k2=0:i=bilinear,lagfun=decay=0.74,${EVEN}[out]`,
    );
  });

  it("leaves film grain and motion trails out of a still image", () => {
    const entries = [entry("grade.texture", { grain: 1 }), entry("grade.lens", { trails: 1 })];
    expect(chainOf(entries, still).chain).toBe("[0:v]null[out]");
    expect(chainOf(entries).chain).toContain("noise=");
    expect(chainOf(entries).chain).toContain("lagfun=");
  });

  it("converts to float colour for white balance and back", () => {
    expect(chainOf([entry("grade.autoFix", { whiteBalance: true })]).chain).toBe(
      `[0:v]format=gbrpf32le,grayworld,format=yuv420p,${EVEN}[out]`,
    );
    expect(chainOf([entry("grade.autoFix", { whiteBalance: true })], still).chain).toBe(
      "[0:v]format=gbrpf32le,grayworld,format=rgb24[out]",
    );
  });

  it("writes the other auto fixes", () => {
    expect(
      chainOf([
        entry("grade.autoFix", {
          levels: true,
          levelsStrength: 0.4,
          levelsSmoothing: 10,
          deband: true,
        }),
      ]).chain,
    ).toBe(`[0:v]normalize=smoothing=10:independence=0:strength=0.4,deband,${EVEN}[out]`);
    expect(chainOf([entry("grade.autoFix", { levels: true, levelsStrength: 0 })]).chain).toBe(
      `[0:v]${EVEN}[out]`,
    );
  });

  it("writes black and white, sepia and tint", () => {
    expect(chainOf([entry("grade.mono")]).chain).toBe(
      `[0:v]monochrome=cb=0:cr=0:size=1,${EVEN}[out]`,
    );
    expect(chainOf([entry("grade.mono", { kind: "sepia" })]).chain).toContain(
      "colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131",
    );
    expect(chainOf([entry("grade.mono", { kind: "tint", tint: "#ff0000" })]).chain).toBe(
      `[0:v]colorize=hue=0:saturation=1:lightness=0.5:mix=1,${EVEN}[out]`,
    );
  });

  it("falls back to the default tint when the colour is empty", () => {
    expect(chainOf([entry("grade.mono", { kind: "tint", tint: "" })]).chain).toContain(
      "colorize=hue=33.6",
    );
  });

  it("blends a step back into the picture below full strength", () => {
    expect(chainOf([entry("grade.mono", { kind: "sepia", strength: 0.5 })]).chain).toBe(
      `[0:v]split[g0][g1];[g1]colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131[g2];[g2][g0]blend=all_mode=normal:all_opacity=0.5[g3];[g3]${EVEN}[out]`,
    );
    expect(chainOf([entry("grade.mono", { strength: 0 })]).chain).toBe(`[0:v]${EVEN}[out]`);
  });

  it("joins plain steps and blends the others between them", () => {
    const { chain } = chainOf([
      entry("grade.colour", { contrast: 1.2 }),
      entry("grade.mono", { kind: "sepia", strength: 0.5 }),
      entry("grade.texture", { vignette: 0.6 }),
    ]);
    expect(chain).toBe(
      `[0:v]eq=contrast=1.2[g0];[g0]split[g1][g2];[g2]colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131[g3];[g3][g1]blend=all_mode=normal:all_opacity=0.5[g4];[g4]vignette=angle=0.942:x0=w*0.5:y0=h*0.5,${EVEN}[out]`,
    );
  });
});

describe("look files", () => {
  it("reads a look table through a 3D table filter", () => {
    const { chain, extraInputs } = chainOf([entry("grade.lookFile", { file: "looks/warm.cube" })]);
    expect(chain).toBe(`[0:v]lut3d=file=/project/looks/warm.cube:interp=tetrahedral,${EVEN}[out]`);
    expect(extraInputs).toEqual([]);
  });

  it("takes a look image as a second input and holds its last frame", () => {
    const { chain, extraInputs } = chainOf([entry("grade.lookFile", { file: "looks/film.png" })]);
    expect(chain).toBe(`[0:v][1:v]haldclut=repeatlast=1[g0];[g0]${EVEN}[out]`);
    expect(extraInputs).toEqual(["/project/looks/film.png"]);
  });

  it("numbers look images in the order of the chain, and blends below full strength", () => {
    const { chain, extraInputs } = chainOf([
      entry("grade.lookFile", { file: "a.png", strength: 0.5 }),
      entry("grade.lookFile", { file: "b.png" }),
    ]);
    expect(extraInputs).toEqual(["/project/a.png", "/project/b.png"]);
    expect(chain).toBe(
      `[0:v]split[g0][g1];[g1][1:v]haldclut=repeatlast=1[g2];[g2][g0]blend=all_mode=normal:all_opacity=0.5[g3];[g3][2:v]haldclut=repeatlast=1[g4];[g4]${EVEN}[out]`,
    );
  });

  it("blends a look table below full strength", () => {
    expect(chainOf([entry("grade.lookFile", { file: "a.cube", strength: 0.5 })]).chain).toBe(
      `[0:v]split[g0][g1];[g1]lut3d=file=/project/a.cube:interp=tetrahedral[g2];[g2][g0]blend=all_mode=normal:all_opacity=0.5[g3];[g3]${EVEN}[out]`,
    );
  });

  it("skips a look with no strength, and refuses other file types", () => {
    expect(chainOf([entry("grade.lookFile", { file: "a.cube", strength: 0 })]).chain).toBe(
      `[0:v]${EVEN}[out]`,
    );
    expect(() => chainOf([entry("grade.lookFile", { file: "look.jpg" })])).toThrow(
      "A look file ends in .cube, .3dl or .png.",
    );
  });

  it("tells look files apart by their ending", () => {
    expect(lookKindOf("a.CUBE")).toBe("table");
    expect(lookKindOf("a.3dl")).toBe("table");
    expect(lookKindOf("a.png")).toBe("image");
    expect(lookKindOf("a.jpg")).toBeNull();
  });

  it("escapes paths for the filter graph", () => {
    expect(filterPath("/a b/it's.cube")).toBe(String.raw`/a b/it\\\'s.cube`);
    expect(filterPath(String.raw`C:\looks\a.cube`)).toBe(String.raw`C\\:\\\\looks\\\\a.cube`);
    expect(filterPath("a,b;c[d]")).toBe(String.raw`a\,b\;c\[d\]`);
  });
});
