import { describe, expect, it } from "vitest";
import { compileWorkflow } from "../compile/compile";
import { fieldMeta } from "../elements/introspect";
import { findById } from "../lists";
import { asValue, type GradeValue } from "../schema/values";
import { GraphBuilder } from "../templates/build";
import { compileNode, nodeCard, must } from "../test-fixtures";
import { gradeChangesNothing, gradeNeedsVideo, gradeNodes, isNeutralEntry } from "./grade";
import { canConnect } from "./ports";
import { defaultParams, getNodeDef } from "./registry";

const entry = (type: string, params: Record<string, unknown> = {}) => ({
  nodeId: type,
  type,
  params: must(getNodeDef(type), type).params.parse({ ...defaultParams(type), ...params }),
});

const texts = (type: string, params: Record<string, unknown> = {}) =>
  nodeCard(type, params).map((r) => r.text);

describe("grade nodes", () => {
  it("is its own port type that only grade nodes make", () => {
    expect(canConnect("grade", "grade")).toBe(true);
    for (const source of ["clip", "media", "recording-style", "recording"] as const)
      expect(canConnect(source, "grade")).toBe(false);
    expect(canConnect("grade", "recording-style")).toBe(false);
    expect(canConnect("grade", "media")).toBe(false);
  });

  it("puts every grade node in the colour grade category, from a grade to a grade", () => {
    for (const def of gradeNodes) {
      expect(def.category).toBe("Grade");
      expect(Object.values(def.inputs).map((p) => p.type)).toEqual(["grade"]);
      expect(Object.values(def.outputs).map((p) => p.type)).toEqual(["grade"]);
      expect(def.inputs.grade.optional).toBe(true);
    }
  });

  it("builds the list in the order of the chain", () => {
    const g = new GraphBuilder();
    g.node("a", "grade.colour", 0, 0, { contrast: 1.2 });
    g.node("b", "grade.mono", 1, 0);
    g.node("c", "grade.texture", 2, 0, { grain: 0.4 });
    g.link("a", "b", "grade", "grade").link("b", "c", "grade", "grade");
    const out = compileWorkflow(g.build({ id: "w", name: "w" }));
    const grade = asValue(out.values.c?.grade, "grade") as GradeValue;
    expect(grade.entries.map((e) => [e.nodeId, e.type])).toEqual([
      ["a", "grade.colour"],
      ["b", "grade.mono"],
      ["c", "grade.texture"],
    ]);
    expect(grade.entries[0].params).toMatchObject({ contrast: 1.2 });
  });

  it("passes a skipped node's input through, so it never enters the list", () => {
    const g = new GraphBuilder();
    g.node("a", "grade.colour", 0, 0);
    g.node("b", "grade.mono", 1, 0);
    g.node("c", "grade.lens", 2, 0);
    g.link("a", "b", "grade", "grade").link("b", "c", "grade", "grade");
    const wf = g.build({ id: "w", name: "w" });
    must(findById(wf.nodes, "b"), "b").disabled = true;
    const out = compileWorkflow(wf);
    const grade = asValue(out.values.c?.grade, "grade") as GradeValue;
    expect(grade.entries.map((e) => e.nodeId)).toEqual(["a", "c"]);
  });

  it("starts a list when nothing is connected", () => {
    const out = compileNode("grade.autoFix", {});
    expect(out.grade).toMatchObject({ type: "grade", entries: [{ type: "grade.autoFix" }] });
  });

  it("changes nothing at default settings, except the nodes that always change the picture", () => {
    const neutral = gradeNodes
      .filter((def) => isNeutralEntry(entry(def.type)))
      .map((def) => def.type);
    expect(neutral).toEqual(["grade.colour", "grade.texture", "grade.lens", "grade.autoFix"]);
  });

  it("calls a grade neutral only when every step is", () => {
    expect(gradeChangesNothing([])).toBe(true);
    expect(gradeChangesNothing([entry("grade.colour"), entry("grade.lens")])).toBe(true);
    expect(gradeChangesNothing([entry("grade.colour"), entry("grade.mono")])).toBe(false);
    expect(gradeChangesNothing([entry("grade.mono", { strength: 0 })])).toBe(true);
  });

  it("treats a tone colour as no change until it has an amount, and never when empty", () => {
    expect(isNeutralEntry(entry("grade.colour", { shadowsAmount: 0.5 }))).toBe(false);
    expect(isNeutralEntry(entry("grade.colour", { shadows: "", shadowsAmount: 0.5 }))).toBe(true);
  });

  it("knows which steps need a video", () => {
    expect(gradeNeedsVideo([entry("grade.lens", { trails: 0.4 })])).toBe(true);
    expect(gradeNeedsVideo([entry("grade.texture", { grain: 0.4 })])).toBe(true);
    expect(
      gradeNeedsVideo([entry("grade.texture", { vignette: 0.4 }), entry("grade.colour")]),
    ).toBe(false);
  });
});

describe("grade cards", () => {
  it("shows a muted No change until something changes the picture", () => {
    const rows = nodeCard("grade.colour");
    expect(rows[0]).toMatchObject({ text: "No change", tone: "muted" });
  });

  it("lists the settings that change the picture", () => {
    expect(texts("grade.colour", { contrast: 1.3, warmth: 0.5, curve: "vintage" })).toEqual([
      "3 changes · Contrast, Warmth, Tone curve",
      "Vintage",
    ]);
    expect(texts("grade.texture", { grain: 0.3 })).toEqual(["1 change · Film grain"]);
    expect(texts("grade.lens", { trails: 0.3, fringe: 0.2 })).toEqual([
      "2 changes · Colour fringing, Motion trails",
    ]);
    expect(texts("grade.autoFix", { deband: true })).toEqual(["1 change · Remove banding"]);
  });

  it("asks for a look file, then shows it with its strength", () => {
    expect(nodeCard("grade.lookFile")[0]).toMatchObject({
      text: "Choose a look file",
      tone: "warn",
    });
    expect(texts("grade.lookFile", { file: "looks/warm.cube", strength: 0.5 })).toEqual([
      "warm.cube",
      "50% strength",
    ]);
  });

  it("names the kind of black and white, with a swatch for a tint", () => {
    expect(texts("grade.mono")).toEqual(["Black and white"]);
    expect(nodeCard("grade.mono", { kind: "tint", tint: "#ff0000" })[0]).toMatchObject({
      text: "Tint",
      swatches: ["#ff0000"],
    });
    expect(nodeCard("grade.mono", { kind: "tint", tint: "" })[0].swatches).toEqual(["#d8a15b"]);
  });

  it("reads the preset from the look values", () => {
    const presets = fieldMeta(
      must(getNodeDef("grade.colour"), "colour").params.shape.brightness,
    ).presets;
    expect(presets?.map((p) => p.label)).toEqual([
      "Natural",
      "Warm",
      "Cool",
      "Punchy",
      "Soft",
      "Black and white",
      "Old film",
    ]);
    expect(
      presets?.every(
        (p) => Object.keys(p.values).join() === "brightness,contrast,saturation,warmth",
      ),
    ).toBe(true);
  });
});

describe("Apply grade card", () => {
  const grade = (...steps: Array<[string, Record<string, unknown>?]>) => ({
    inputGrade: { grade: steps.map(([type, params]) => entry(type, params)) },
  });

  it("asks for a grade", () => {
    expect(nodeCard("media.applyGrade")[0]).toMatchObject({
      text: "Connect a grade",
      tone: "warn",
    });
  });

  it("counts the steps and names them once", () => {
    const rows = nodeCard(
      "media.applyGrade",
      {},
      grade(
        ["grade.colour", { contrast: 1.2 }],
        ["grade.colour", { warmth: 0.2 }],
        ["grade.texture", { grain: 0.2 }],
      ),
    );
    expect(rows.map((r) => r.text)).toEqual([
      "3 steps: Colour and light, Texture",
      "MP4 · High quality",
    ]);
  });

  it("warns when the grade changes nothing", () => {
    const rows = nodeCard("media.applyGrade", {}, grade(["grade.colour"], ["grade.lens"]));
    expect(rows.map((r) => r.text)).toContain("The grade changes nothing");
  });

  it("says HEVC may not preview, since some computers can't play it in the app", () => {
    expect(texts("media.applyGrade", { codec: "h265" }).join()).toBe(
      "Connect a grade,MP4 (HEVC) · High quality · may not preview",
    );
  });

  it("says a ProRes file has no Preview, and writes a PNG for an image", () => {
    const steps = grade(["grade.mono"]);
    expect(texts("media.applyGrade", { codec: "prores" }).join()).toBe(
      "Connect a grade,ProRes · no Preview",
    );
    expect(
      nodeCard("media.applyGrade", {}, { ...steps, inputMedia: { media: { kind: "image" } } })[1],
    ).toMatchObject({ text: "PNG" });
  });

  it("warns that trails and grain need a video when the file is an image", () => {
    const steps = grade(["grade.lens", { trails: 0.5 }]);
    const rows = nodeCard(
      "media.applyGrade",
      {},
      { ...steps, inputMedia: { media: { kind: "image" } } },
    );
    expect(rows.at(-1)).toMatchObject({ text: "Trails and film grain need a video", tone: "warn" });
    const video = nodeCard(
      "media.applyGrade",
      {},
      { ...steps, inputMedia: { media: { kind: "video" } } },
    );
    expect(video.map((r) => r.text)).not.toContain("Trails and film grain need a video");
  });
});
