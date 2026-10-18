import { describe, expect, it } from "vitest";
import { emptyClip, groupOf } from "../compile/clip";
import { compileClip, must, nodeCard } from "../test-fixtures";

const SPRING_SETTLE = "fx.remocn.spring-settle";

describe("Spring settle", () => {
  it("plays the connected clips as scenes with its settings, pause and scene colours", () => {
    const clip = compileClip(
      SPRING_SETTLE,
      { gapFrames: 4, enterScale: 1.3, backgrounds: [{ scene: "in3", color: "#112233" }] },
      { in1: emptyClip(30), in3: emptyClip(20) },
    );
    expect(clip.durationInFrames).toBe(54);
    const sequencer = must(groupOf(clip.tracks[0].items[0])?.sequencer, "sequencer");
    expect(sequencer.component).toBe("remocn/spring-settle");
    expect(sequencer.props).toMatchObject({ enterScale: 1.3, gapFrames: 4 });
    expect(sequencer.backgrounds).toEqual([null, "#112233"]);
  });

  it("counts its scenes and the items in each on its card", () => {
    const rows = nodeCard(
      SPRING_SETTLE,
      { gapFrames: 15 },
      { connected: ["in1", "in2"], inputItems: { in1: 1, in2: 3 }, clipFrames: 90 },
    ).map((r) => r.text);
    expect(rows[0]).toBe("2 scenes");
    expect(rows[1]).toBe("Items per scene: 1 · 3");
    expect(rows.some((text) => text.endsWith("between scenes"))).toBe(true);
  });

  it("warns when no scene is connected", () => {
    expect(nodeCard(SPRING_SETTLE)[0]).toMatchObject({ text: "0 scenes", tone: "warn" });
  });
});
