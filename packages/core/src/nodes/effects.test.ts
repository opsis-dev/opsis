import { describe, expect, it } from "vitest";
import { itemClip, recordingProps } from "../compile/clip";
import { compileClip, elementOf, itemsOf, nodeCard, zoomed } from "../test-fixtures";
import { parseParams } from "./params";
import { getNodeDef } from "./registry";

const blurred = (params: Record<string, unknown>[]) =>
  recordingProps(zoomed({ own: params.map((p) => ["effect.blurAreas", p] as const) }))[0];

describe("blur areas", () => {
  it("puts its areas on the take, with the strength as a share of the frame width", () => {
    const props = blurred([{ strength: 0.5, softness: 0, roundness: 0.2 }]);
    expect(props.blurAreas).toEqual([
      {
        x: 0.62,
        y: 0.06,
        width: 0.3,
        height: 0.08,
        strength: 0.02,
        softness: 0,
        roundness: 0.2,
      },
    ]);
  });

  it("adds up across nodes, each keeping its own times", () => {
    const props = blurred([
      { fromMs: 1000, untilMs: 2000 },
      { areas: [{ x: 0, y: 0, width: 0.1, height: 0.1 }], fromMs: 3000 },
    ]);
    expect(props.blurAreas?.map((a) => [a.fromMs, a.untilMs])).toEqual([
      [1000, 2000],
      [3000, undefined],
    ]);
  });

  it("refuses an end before its start", () => {
    const def = getNodeDef("effect.blurAreas");
    expect(def && parseParams(def.params, { fromMs: 2000, untilMs: 1000 })).toEqual({
      ok: false,
      problem: "Until must come after From",
    });
  });

  it("shows its areas and times on the card", () => {
    expect(nodeCard("effect.blurAreas", { fromMs: 500 }).map((r) => r.text)).toEqual([
      "1 area · 60% strength",
      "From 0.5 s",
    ]);
    expect(nodeCard("effect.blurAreas", { areas: [] })[0]).toMatchObject({
      text: "Draw an area to blur",
    });
  });
});

describe("picture look", () => {
  const video = itemClip("v", "core/video", { src: "project:a.mp4" }, 30);
  const image = itemClip("i", "core/image", { src: "project:a.png" }, 30);

  it("gives recordings, videos and images only the settings that change the picture", () => {
    const look = { warmth: 0.3, grain: 0.2 };
    const [rec] = recordingProps(compileClip("effect.look", look, { clip: zoomed() }));
    expect(rec.look).toEqual(look);
    for (const clip of [video, image]) {
      const item = elementOf(itemsOf(compileClip("effect.look", look, { clip }))[0]);
      expect(item.props.look).toEqual(look);
    }
  });

  it("leaves other items alone", () => {
    const title = itemClip("t", "core/title", { title: "Hi" }, 30);
    const [item] = itemsOf(compileClip("effect.look", { warmth: 0.3 }, { clip: title }));
    expect(item.props).not.toHaveProperty("look");
  });

  it("names the look on the card", () => {
    expect(nodeCard("effect.look", {})[0]).toMatchObject({ text: "Natural", tone: "muted" });
    expect(nodeCard("effect.look", { saturation: 0, contrast: 1.15 })[0].text).toBe(
      "Black and white",
    );
    expect(nodeCard("effect.look", { grain: 0.1 })[0].text).toBe("1 change");
  });
});
