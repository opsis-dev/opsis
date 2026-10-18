import { describe, expect, it } from "vitest";
import { fieldMeta } from "../elements/introspect";
import { DEFAULT_WORKFLOW_SETTINGS as settings } from "../schema/workflow";
import { nodeResult } from "../test-fixtures";
import { elementRow, sentence } from "./card";
import { cardRows, type NodeDef } from "./define";
import { defaultParams, NODE_DEFS } from "./registry";

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
    const empty = NODE_DEFS.filter((def) => def.card).filter((def) => {
      const out = cardRows(def, {
        params: defaultParams(def.type),
        settings,
        result,
        connected: Object.keys(def.inputs),
      });
      return !out.length || out.some((r) => !r.icon || !r.text);
    });
    expect(empty.map((d) => d.type)).toEqual([]);
  });

  it("gives every visible node a description and no internal names", () => {
    const visible = NODE_DEFS.filter((d) => !d.hidden);
    const banned =
      /playwright|remotion|chromium|ffmpeg|ffprobe|page\.evaluate|safeStorage|\.mp4|crf|dpr\b/i;
    const fieldCopy = (def: NodeDef) =>
      Object.values(def.params.shape).flatMap((field) => {
        const meta = fieldMeta(field);
        if (meta.hidden) return [];
        return [
          meta.description,
          meta.help,
          meta.zeroLabel,
          meta.presetsLabel,
          ...Object.values(meta.labels ?? {}),
          ...(meta.presets ?? []).map((p) => p.label),
        ];
      });
    expect(visible.filter((d) => !d.description).map((d) => d.type)).toEqual([]);
    expect(
      visible
        .filter((d) => banned.test([d.title, d.description, ...fieldCopy(d)].join(" ")))
        .map((d) => d.type),
    ).toEqual([]);
  });
});
