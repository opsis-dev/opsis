import { describe, expect, it } from "vitest";
import type { NodeResult } from "../schema/values";
import type { WorkflowSettings } from "../schema/workflow";
import { elementName, secs, sentence, shortUrl } from "./card";
import { defaultParams, getNodeDef, NODE_DEFS, normalizeNode, normalizeWorkflow } from "./registry";

const settings: WorkflowSettings = { width: 1920, height: 1080, fps: 30, theme: "dark" };

function rows(type: string, params: Record<string, unknown>, extra: Record<string, unknown> = {}) {
  const def = getNodeDef(type)!;
  return def.card!({
    params: { ...defaultParams(type), ...params },
    raw: params,
    settings,
    connected: [],
    ...extra,
  })
    .filter((r) => r && typeof r === "object")
    .map((r) => r as { icon: string; text: string; tone?: string });
}

describe("card formatting", () => {
  it("writes durations in seconds and minutes", () => {
    expect(secs(500)).toBe("0.5 s");
    expect(secs(1300)).toBe("1.3 s");
    expect(secs(14_520)).toBe("14.5 s");
    expect(secs(65_000)).toBe("1 min 5 s");
    expect(secs(120_000)).toBe("2 min");
  });

  it("describes elements in words", () => {
    expect(elementName("Login", "button")).toBe("“Login” button");
    expect(elementName("Username", "textbox")).toBe("“Username” field");
    expect(elementName("What needs to be done?")).toBe("“What needs to be done?”");
    expect(elementName("css=#add-to-cart")).toBe("Custom element");
    expect(elementName("320, 180")).toBe("Point 320, 180");
    expect(elementName("  ")).toBeNull();
  });

  it("shortens addresses and titles", () => {
    expect(shortUrl("https://www.saucedemo.com/")).toBe("saucedemo.com");
    expect(sentence("Whip Pan")).toBe("Whip pan");
    expect(sentence("RGB Glitch Text")).toBe("RGB glitch text");
  });
});

describe("node cards", () => {
  it("renders every card from default settings without throwing", () => {
    const broken = NODE_DEFS.filter((d) => d.card).filter(
      (def) =>
        !Array.isArray(
          def.card!({ params: defaultParams(def.type), raw: {}, settings, connected: [] }),
        ),
    );
    expect(broken.map((d) => d.type)).toEqual([]);
  });

  it("gives every visible node a description and no internal names", () => {
    const visible = NODE_DEFS.filter((d) => !d.hidden);
    const banned =
      /playwright|remotion|chromium|ffmpeg|ffprobe|PLAN\.md|page\.evaluate|safeStorage|\.mp4|moov|crf|dpr\b/i;
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
    const record: NodeResult = {
      hash: "h",
      outputs: {
        recording: {
          type: "recording",
          src: "project:a.mp4",
          log: { viewport: { width: 1, height: 1 }, dpr: 1, durationMs: 14_500, actions: [{}, {}] },
          durationMs: 14_500,
          viewport: { width: 1920, height: 1080 },
          nodeId: "r",
        },
      } as unknown as NodeResult["outputs"],
      artifacts: [],
      finishedAt: 1,
      durationMs: 1,
    };
    expect(rows("capture.record", {}, { result: record }).map((r) => r.text)).toEqual([
      "30 fps",
      "Smooth cursor",
      "14.5 s",
      "2 actions",
    ]);
    const render: NodeResult = {
      hash: "h",
      outputs: { media: { type: "media", src: null, kind: "video", durationMs: 15_400 } },
      artifacts: [
        { kind: "video", path: "a.mp4", bytes: 1_000_000 },
        { kind: "video", path: "b.mp4", bytes: 500_000 },
      ],
      finishedAt: 1,
      durationMs: 1,
    };
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

  it("counts connected clips on Sequence", () => {
    expect(rows("timeline.sequence", {}, { connected: ["in1", "in2", "in3"] })[0].text).toBe(
      "3 clips",
    );
  });
});

describe("node names", () => {
  it("moves the old camera label onto the node name", () => {
    const node = normalizeNode({
      id: "t",
      type: "browser.type",
      position: { x: 0, y: 0 },
      params: { target: "x", label: "New todo" },
    });
    expect(node.label).toBe("New todo");
    expect(node.params).not.toHaveProperty("label");
  });

  it("renames old code-style variables to readable names", () => {
    const wf = normalizeWorkflow({
      version: 1,
      id: "w",
      name: "w",
      settings: { width: 1920, height: 1080, fps: 30, theme: "dark" },
      variables: { BASE_URL: "https://x.test", Custom: "1" },
      nodes: [
        {
          id: "n",
          type: "browser.navigate",
          position: { x: 0, y: 0 },
          params: { url: "{{BASE_URL}}/a" },
        },
        {
          id: "r",
          type: "output.render",
          position: { x: 0, y: 0 },
          params: { fileName: "{{PROJECT_NAME}}" },
        },
      ],
      edges: [],
    });
    expect(wf.variables).toEqual({ "Site address": "https://x.test", Custom: "1" });
    expect(wf.nodes[0].params.url).toBe("{{Site address}}/a");
    expect(wf.nodes[1].params.fileName).toBe("{{Project name}}");
  });

  it("keeps a label setting that belongs to the node", () => {
    const withLabel = NODE_DEFS.find((d) => "label" in d.params.shape);
    if (!withLabel) return;
    const node = normalizeNode({
      id: "e",
      type: withLabel.type,
      position: { x: 0, y: 0 },
      params: { label: "Keep me" },
    });
    expect(node.params.label).toBe("Keep me");
  });
});
