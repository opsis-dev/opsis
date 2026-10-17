import { describe, expect, it } from "vitest";
import { must, nodeResult, recordResult } from "../test-fixtures";
import { DEFAULT_WORKFLOW_SETTINGS as settings } from "../schema/workflow";
import { elementRow, sentence, type CardContext, type CardRow } from "./card";
import type { NodeDef } from "./define";
import { defaultParams, getNodeDef, NODE_DEFS } from "./registry";

function rows(
  type: string,
  params: Record<string, unknown>,
  extra: Partial<CardContext<Record<string, unknown>>> = {},
) {
  const card = must(getNodeDef(type)?.card, `card for ${type}`);
  return card({
    params: { ...defaultParams(type), ...params },
    settings,
    connected: [],
    ...extra,
  }).filter((r): r is CardRow => !!r);
}

describe("card formatting", () => {
  it("describes elements in words", () => {
    const named = (target: string, role?: string) => elementRow("crosshair", target, role).text;
    expect(named("Login", "button")).toBe("“Login” button");
    expect(named("Username", "textbox")).toBe("“Username” field");
    expect(named("What needs to be done?")).toBe("“What needs to be done?”");
    expect(named("css=#add-to-cart")).toBe("Custom element");
    expect(named("320, 180")).toBe("Point 320, 180");
    expect(elementRow("crosshair", "  ", undefined)).toMatchObject({
      text: "Pick an element",
      tone: "warn",
    });
  });

  it.each([
    ["Whip Pan", "Whip pan"],
    ["RGB Glitch Text", "RGB glitch text"],
    ["Slide In 3D", "Slide in 3D"],
  ])("writes the title %s in sentence case", (title, expected) => {
    expect(sentence(title)).toBe(expected);
  });
});

describe("node cards", () => {
  it("shows at least one well-formed row on every card from default settings", () => {
    const result = nodeResult({
      outputs: { media: { type: "media", src: null, kind: "video", durationMs: 1000 } },
    });
    const withCard = NODE_DEFS.filter(
      (d): d is NodeDef & Required<Pick<NodeDef, "card">> => !!d.card,
    );
    const empty = withCard.filter((def) => {
      const out = def
        .card({
          params: defaultParams(def.type),
          settings,
          result,
          connected: Object.keys(def.inputs),
        })
        .filter((r): r is CardRow => !!r);
      return !out.length || out.some((r) => !r.icon || !r.text);
    });
    expect(empty.map((d) => d.type)).toEqual([]);
  });

  it("gives every visible node a description and no internal names", () => {
    const visible = NODE_DEFS.filter((d) => !d.hidden);
    const banned =
      /playwright|remotion|chromium|ffmpeg|ffprobe|page\.evaluate|safeStorage|\.mp4|crf|dpr\b/i;
    expect(visible.filter((d) => !d.description).map((d) => d.type)).toEqual([]);
    expect(
      visible.filter((d) => banned.test(`${d.title} ${d.description}`)).map((d) => d.type),
    ).toEqual([]);
  });

  it("shows what the Type text node does", () => {
    const out = rows("browser.type", {
      target: "Username",
      targetRole: "textbox",
      text: "tom¦smith",
      pressEnter: true,
      after: 500,
      cluster: "login",
    });
    expect(out.map((r) => [r.icon, r.text])).toEqual([
      ["crosshair", "“Username” field"],
      ["keyboard", "“tom … smith”"],
      ["corner-down-left", "Then Enter"],
      ["timer", "Pause 0.5 s"],
      ["zoom-in", "Zoom group “login”"],
    ]);
  });

  it("asks for the missing element in the warning tone", () => {
    expect(rows("browser.click", { target: "" })[0]).toMatchObject({
      text: "Pick an element",
      tone: "warn",
    });
  });

  it("adds run results to Record take and Export video", () => {
    const record = recordResult({
      viewport: { width: 1920, height: 1080 },
      dpr: 1,
      t0: 0,
      durationMs: 14_500,
      actions: [
        { id: "a", kind: "click", t: 1000 },
        { id: "b", kind: "click", t: 2000 },
      ],
    });
    expect(rows("capture.record", {}, { result: record }).map((r) => r.text)).toEqual([
      "30 fps",
      "Smooth cursor",
      "14.5 s",
      "2 actions",
    ]);
    const render = nodeResult({
      outputs: { media: { type: "media", src: null, kind: "video", durationMs: 15_400 } },
      artifacts: [
        { kind: "video", path: "a.mp4", bytes: 1_000_000 },
        { kind: "video", path: "b.mp4", bytes: 500_000 },
      ],
    });
    expect(
      rows("output.render", { variants: ["16:9", "9:16"] }, { result: render }).map((r) => r.text),
    ).toEqual(["16:9 + 9:16", "MP4 · 1080p · 30 fps", "15.4 s", "1.4 MB"]);
  });

  it("tells whether a sign-in is saved", () => {
    expect(
      rows("capture.loginOnce", { secretName: "sauce demo" }, { secrets: ["sauce-demo"] }).at(-1),
    ).toMatchObject({ text: "Signed in" });
    expect(
      rows("capture.loginOnce", { secretName: "other" }, { secrets: [] }).at(-1),
    ).toMatchObject({
      text: "Not signed in yet",
      tone: "warn",
    });
  });

  it("warns on Speed that the Preview plays slowly from 4×", () => {
    expect(rows("timeline.speed", { rate: 2 }).map((r) => r.text)).toEqual(["2×"]);
    expect(rows("timeline.speed", { rate: 4 }).map((r) => [r.text, r.tone])).toEqual([
      ["4×", undefined],
      ["Very fast: the Preview plays slowly. The export is smooth.", "muted"],
    ]);
  });

  it("shows what a Drag picks up and where it goes", () => {
    const texts = (params: Record<string, unknown>) =>
      rows("browser.drag", { target: "Deal card", after: 0, ...params }).map((r) => r.text);
    expect(texts({ mode: "element", to: "Won" })).toEqual(["“Deal card”", "Onto “Won”"]);
    expect(texts({ mode: "element" })).toEqual(["“Deal card”", "Pick where to drop it"]);
    expect(texts({ mode: "by", dx: 300, dy: -40 })).toEqual(["“Deal card”", "By 300, -40 px"]);
    expect(texts({ mode: "path", path: "0, 0\n40, 40\n120, -80", cluster: "Board" })).toEqual([
      "“Deal card”",
      "Along 3 points",
      "Zoom group “Board”",
    ]);
    expect(texts({ mode: "path", path: "" })).toEqual(["“Deal card”", "Add points to the path"]);
  });

  it("counts connected clips on Sequence", () => {
    expect(rows("timeline.sequence", {}, { connected: ["in1", "in2", "in3"] })[0].text).toBe(
      "3 clips",
    );
  });

  it("counts Sequence cuts with their own transition and warns about the worst one", () => {
    const cuts = [
      { before: "in3", transition: "core/wipe", transitionDuration: 50, keepWhole: false },
      { before: "in5", transition: "core/iris", transitionDuration: 10, keepWhole: false },
    ];
    const texts = rows(
      "timeline.sequence",
      { transitionDuration: 10, cuts },
      {
        connected: ["in1", "in2", "in3"],
        inputFrames: { in1: 300, in2: 100, in3: 300 },
      },
    ).map((r) => r.text);
    expect(texts).toEqual([
      "3 clips",
      "Fade · 0.33 s",
      "1 cut uses its own transition",
      "Covers 50% of a 3.33 s clip",
    ]);
    const held = rows(
      "timeline.sequence",
      { transitionDuration: 10, cuts: [{ ...cuts[0], keepWhole: true }] },
      { connected: ["in1", "in2", "in3"], inputFrames: { in1: 300, in2: 100, in3: 300 } },
    );
    expect(held.some((r) => r.text.startsWith("Covers"))).toBe(false);
  });
});
