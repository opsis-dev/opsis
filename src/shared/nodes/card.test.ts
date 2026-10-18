import { describe, expect, it } from "vitest";
import { z } from "zod";
import { fieldMeta, unwrap } from "../elements/introspect";
import { DEFAULT_WORKFLOW_SETTINGS as settings } from "../schema/workflow";
import { makeLog, nodeResult, recordResult } from "../test-fixtures";
import { elementRow, MAX_CARD_ROWS, sentence } from "./card";
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
      /playwright|remotion|chromium|ffmpeg|ffprobe|lottie|after effects|page\.evaluate|safeStorage|\.mp4|\.json|crf|dpr\b/i;
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

describe("card length", () => {
  const LONG_TEXT = "A long piece of text that runs well past the width of a card";

  function candidates(field: z.ZodType, current: unknown): unknown[] {
    const inner = unwrap(field);
    if (inner instanceof z.ZodBoolean) return [false, true];
    if (inner instanceof z.ZodEnum) return inner.options;
    if (inner instanceof z.ZodNumber) return [0, 1, 5, 500];
    if (inner instanceof z.ZodString) return ["", "Save", "de-DE", LONG_TEXT];
    const item =
      inner instanceof z.ZodArray && inner.element instanceof z.ZodType
        ? unwrap(inner.element)
        : null;
    if (item instanceof z.ZodEnum) return [[], item.options];
    return [current];
  }

  function contexts(def: NodeDef) {
    const ports = Object.keys(def.inputs ?? {});
    const each = (n: number) => Object.fromEntries(ports.map((p) => [p, n]));
    const log = makeLog(
      [
        { id: "a", kind: "click", t: 1000 },
        { id: "b", kind: "type", t: 2000 },
      ],
      14_500,
    );
    return [
      { settings, connected: [] },
      {
        settings,
        connected: ports,
        clipFrames: 90,
        inputFrames: each(60),
        inputItems: each(3),
        secrets: ["work"],
        result: def.type === "capture.record" ? recordResult(log) : nodeResult(),
      },
      { settings, connected: ports, clipFrames: 12, inputFrames: each(8), inputItems: each(1) },
    ];
  }

  function longestCard(def: NodeDef): number {
    const card = def.card;
    if (!card) return 0;
    let longest = 0;
    for (const context of contexts(def)) {
      const rows = (params: Record<string, unknown>) => {
        const parsed = def.params.safeParse(params);
        return parsed.success
          ? card({ ...context, params: parsed.data }).filter(Boolean).length
          : -1;
      };
      let params: Record<string, unknown> = defaultParams(def.type);
      let best = rows(params);
      for (let improved = true; improved;) {
        improved = false;
        for (const [key, field] of Object.entries(def.params.shape))
          for (const value of candidates(field, params[key])) {
            const next = { ...params, [key]: value };
            const count = rows(next);
            if (count > best) [params, best, improved] = [next, count, true];
          }
      }
      longest = Math.max(longest, best);
    }
    return longest;
  }

  it("never builds more rows than a card shows, whatever its settings", () => {
    const over = NODE_DEFS.flatMap((def) => {
      const longest = longestCard(def);
      return longest > MAX_CARD_ROWS ? [`${def.type}: ${longest}`] : [];
    });
    expect(over).toEqual([]);
  });
});
