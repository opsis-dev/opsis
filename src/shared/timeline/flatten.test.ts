import { describe, expect, it } from "vitest";
import { buildFilm, filmSpec, flatFilm, TITLE } from "../test-fixtures";
import { nodeSpans } from "./flatten";
import { findHandle } from "./handles";

describe("flattening", () => {
  it("maps bars through speed and trims", () => {
    const wf = buildFilm((g) => {
      g.node("t", TITLE, 0, 0, { clipDuration: 120 });
      g.node("trim", "timeline.trim", 1, 0, { startMs: 1000, durationMs: 0 });
      g.node("speed", "timeline.speed", 2, 0, { rate: 2 });
      g.link("t", "trim").link("trim", "speed");
      return "speed";
    });
    const [bar] = flatFilm(wf).bars;
    expect(bar.from).toBeCloseTo(0, 3);
    expect(bar.to).toBeCloseTo(45, 0);
    expect(bar.rate).toBeCloseTo(2, 3);
    expect(bar.contentStart).toBeCloseTo(30, 0);
    expect(bar.chain.map((l) => l.nodeId).filter(Boolean)).toEqual(["t", "trim", "speed"]);
  });

  it("stacks overlapping layers on separate lanes in drawing order", () => {
    const wf = buildFilm((g) => {
      g.node("a", TITLE, 0, 0, { clipDuration: 90 });
      g.node("b", TITLE, 0, 1, { clipDuration: 30 });
      g.node("layers", "timeline.stack", 2, 0, { offsetsMs: "0,500" });
      g.link("a", "layers", "clip", "in1").link("b", "layers", "clip", "in2");
      return "layers";
    });
    const f = flatFilm(wf);
    expect(f.lanes).toBe(2);
    const b = f.bars.find((x) => x.nodeId === "b");
    expect(b?.lane).toBe(1);
    expect(b?.from).toBeCloseTo(15, 0);
    expect(b && findHandle(b, "move", "outer")?.link.handles?.move).toMatchObject({
      nodeId: "layers",
      param: "offsetsMs",
      index: 1,
    });
  });

  it("repeats bars for every loop", () => {
    const wf = buildFilm((g) => {
      g.node("t", TITLE, 0, 0, { clipDuration: 30 });
      g.node("loop", "timeline.loop", 1, 0, { times: 3 });
      g.link("t", "loop");
      return "loop";
    });
    expect(flatFilm(wf).bars.map((b) => [Math.round(b.from), Math.round(b.to)])).toEqual([
      [0, 30],
      [30, 60],
      [60, 90],
    ]);
  });

  it("spans every loop and speed change of a node", () => {
    const wf = buildFilm((g) => {
      g.node("t", TITLE, 0, 0, { clipDuration: 60 });
      g.node("speed", "timeline.speed", 1, 0, { rate: 2 });
      g.node("loop", "timeline.loop", 2, 0, { times: 3 });
      g.link("t", "speed").link("speed", "loop");
      return "loop";
    });
    expect(nodeSpans(filmSpec(wf))).toEqual({
      t: { from: 0, to: 90 },
      speed: { from: 0, to: 90 },
      loop: { from: 0, to: 90 },
    });
  });

  it("marks transitions between sequenced clips", () => {
    const wf = buildFilm((g) => {
      g.node("a", TITLE, 0, 0, { clipDuration: 60 });
      g.node("b", TITLE, 0, 1, { clipDuration: 60 });
      g.node("seq", "timeline.sequence", 2, 0, { transition: "core/fade", transitionDuration: 15 });
      g.link("a", "seq", "clip", "in1").link("b", "seq", "clip", "in2");
      return "seq";
    });
    const f = flatFilm(wf);
    expect(f.transitions).toHaveLength(1);
    expect(f.transitions[0]).toMatchObject({
      from: 45,
      to: 60,
      kind: "core/fade",
      slot: { nodeId: "seq", port: "in2" },
    });
  });
});
