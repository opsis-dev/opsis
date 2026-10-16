import { describe, expect, it } from "vitest";
import { compileWorkflow, previewSpec } from "../compile/compile";
import { NODE_DEFS } from "../nodes/registry";
import type { EditHandleKind } from "../schema/timeline";
import type { Workflow } from "../schema/workflow";
import { GraphBuilder } from "../templates/build";
import {
  moveAudio,
  moveBar,
  removalFor,
  rollCut,
  setAudioVolume,
  slipBar,
  speedBar,
  splitBar,
  trimAudio,
  trimBar,
  type EditContext,
  type EditResult,
} from "./edits";
import { findHandle, flattenSpec, type FlatTimeline, type VideoBar } from "./flatten";

const TITLE = "fx.core.title";

function flat(wf: Workflow): FlatTimeline {
  const result = compileWorkflow(wf);
  const target = previewSpec(wf, result, null);
  if (!target) throw new Error("nothing to preview");
  return flattenSpec(target.spec);
}

let counter = 0;
const ctxFor = (workflow: Workflow): EditContext => ({
  workflow,
  fps: workflow.settings.fps,
  sourceNodeId: "out",
  makeId: (prefix) => `${prefix}_t${++counter}`,
});

const applied = (r: EditResult): Workflow => {
  if (!r.ok) throw new Error(r.error);
  return r.workflow;
};

const len = (b: { from: number; to: number }) => b.to - b.from;

function build(fn: (g: GraphBuilder) => string): Workflow {
  const g = new GraphBuilder();
  const last = fn(g);
  g.node("out", "output.render", 9, 0);
  g.link(last, "out", "clip", "timeline");
  return g.build({ id: "wf", name: "Test" });
}

const DELTA = 6;

const KINDS: EditHandleKind[] = ["move", "trimStart", "trimEnd", "speed"];

function feedInputs(g: GraphBuilder, id: string, type: string): void {
  const def = NODE_DEFS.find((d) => d.type === type)!;
  Object.entries(def.inputs).forEach(([port, spec], i) => {
    if (spec.type !== "clip") return;
    const src = g.node(`${id}_${port}`, TITLE, 0, i + 1, {
      title: `Clip ${i + 1}`,
      clipDuration: 90 + i * 15,
    });
    g.link(src, id, "clip", port);
  });
}

describe("timeline edit handles round-trip", () => {
  const covered = new Map<EditHandleKind, Set<string>>(KINDS.map((k) => [k, new Set()]));
  const clipNodes = NODE_DEFS.filter((d) => d.compile && d.outputs.clip && !d.hidden);

  it.each(clipNodes.map((d) => [d.type]))("%s: every handle moves its bar by the delta", (type) => {
    const wf = build((g) => {
      g.node("x", type, 3, 0);
      feedInputs(g, "x", type);
      return "x";
    });
    const result = compileWorkflow(wf);
    if (!previewSpec(wf, result, null)) return;
    const before = flat(wf);
    for (const bar of before.bars) {
      for (const kind of KINDS) {
        const owner = findHandle(bar, kind, "outer");
        const ref = owner?.link.handles?.[kind];
        if (!owner || !ref || !wf.nodes.some((n) => n.id === ref.nodeId)) continue;
        const ctx = ctxFor(wf);
        let next: Workflow;
        if (kind === "move") next = applied(moveBar(ctx, bar, DELTA));
        else if (kind === "trimStart") next = applied(trimBar(ctx, bar, "start", DELTA));
        else if (kind === "trimEnd") next = applied(trimBar(ctx, bar, "end", -DELTA));
        else next = applied(speedBar(ctx, bar, Math.round(len(owner.link) / 2)));
        expect(next.nodes.length).toBe(wf.nodes.length);
        const after = flat(next).bars.find((b) => b.key === bar.key)!;
        expect(after).toBeDefined();
        const [actual, expected, tolerance] =
          kind === "move"
            ? [after.from - bar.from, DELTA, 0.5]
            : kind === "speed"
              ? [len(after), Math.round(len(owner.link) / 2), 1]
              : [len(bar) - len(after), DELTA, 0.5];
        expect({ type, kind, off: Math.abs(actual - expected) <= tolerance }).toEqual({
          type,
          kind,
          off: true,
        });
        covered.get(kind)!.add(ref.nodeId.split("_")[0] === "x" ? type : ref.nodeId);
      }
    }
  });

  it("covers every kind of handle", () => {
    for (const kind of KINDS) expect(covered.get(kind)!.size).toBeGreaterThan(0);
    expect(covered.get("move")).toContain("timeline.delay");
    expect(covered.get("trimStart")).toContain("timeline.trim");
    expect(covered.get("trimEnd")).toContain("timeline.trim");
    expect(covered.get("trimEnd")).toContain(TITLE);
    expect(covered.get("speed")).toContain("timeline.speed");
  });
});

describe("timeline edits that add nodes", () => {
  const single = () => build((g) => g.node("t", TITLE, 0, 0, { title: "Hello", clipDuration: 90 }));

  it("adds a Delay to move a clip that has none", () => {
    const wf = single();
    const [bar] = flat(wf).bars;
    const r = moveBar(ctxFor(wf), bar, 12);
    const next = applied(r);
    expect(
      next.nodes.some((n) => n.type === "timeline.delay" && n.label === "Delay from timeline"),
    ).toBe(true);
    expect(flat(next).bars[0].from).toBeCloseTo(12, 0);
    expect(r.ok && r.message).toMatch(/Delay from timeline/);
  });

  it("adds a Trim to trim the start of a clip that has none", () => {
    const wf = single();
    const [bar] = flat(wf).bars;
    const next = applied(trimBar(ctxFor(wf), bar, "start", 30));
    expect(next.nodes.some((n) => n.type === "timeline.trim")).toBe(true);
    expect(len(flat(next).bars[0])).toBeCloseTo(60, 0);
  });

  it("adds a Speed to stretch a clip", () => {
    const wf = single();
    const [bar] = flat(wf).bars;
    const next = applied(speedBar(ctxFor(wf), bar, 45));
    expect(next.nodes.find((n) => n.type === "timeline.speed")?.params.rate).toBe(2);
    expect(len(flat(next).bars[0])).toBeCloseTo(45, 0);
  });

  it("speeds a clip up to 64× and no further", () => {
    const wf = single();
    const [bar] = flat(wf).bars;
    const fast = applied(speedBar(ctxFor(wf), bar, 2));
    expect(fast.nodes.find((n) => n.type === "timeline.speed")?.params.rate).toBe(45);
    const fastest = applied(speedBar(ctxFor(fast), flat(fast).bars[0], 1));
    expect(fastest.nodes.find((n) => n.type === "timeline.speed")?.params.rate).toBe(64);
    const capped = applied(speedBar(ctxFor(wf), bar, 1));
    expect(capped.nodes.find((n) => n.type === "timeline.speed")?.params.rate).toBe(64);
  });

  it("slips the content without changing the length", () => {
    const wf = build((g) => {
      g.node("t", TITLE, 0, 0, { clipDuration: 90 });
      g.node("trim", "timeline.trim", 1, 0, { startMs: 0, durationMs: 1000 });
      g.link("t", "trim");
      return "trim";
    });
    const [trimmed] = flat(wf).bars;
    expect(len(trimmed)).toBeCloseTo(30, 0);
    const next = applied(slipBar(ctxFor(wf), trimmed, 10));
    const slipped = flat(next).bars[0];
    expect(len(slipped)).toBeCloseTo(30, 0);
    expect(slipped.contentStart - trimmed.contentStart).toBeCloseTo(10, 0);
  });

  it("adds a Trim to slip a clip that has none", () => {
    const wf = single();
    const [bar] = flat(wf).bars;
    const next = applied(slipBar(ctxFor(wf), bar, 10));
    expect(next.nodes.some((n) => n.type === "timeline.trim")).toBe(true);
  });

  it("splits a clip at the playhead into two parts in a sequence", () => {
    const wf = single();
    const [bar] = flat(wf).bars;
    const next = applied(splitBar(ctxFor(wf), bar, 30));
    const bars = flat(next).bars.toSorted((a, b) => a.from - b.from);
    expect(bars).toHaveLength(2);
    expect(bars[0].from).toBeCloseTo(0, 0);
    expect(bars[0].to).toBeCloseTo(30, 0);
    expect(bars[1].from).toBeCloseTo(30, 0);
    expect(bars[1].to).toBeCloseTo(90, 0);
    expect(bars[1].contentStart).toBeCloseTo(30, 0);
    const seq = next.nodes.find((n) => n.type === "timeline.sequence")!;
    expect(seq.params.transition).toBe("none");
  });

  it("splits inside a cut-only sequence without nesting another one", () => {
    const wf = build((g) => {
      g.node("a", TITLE, 0, 0, { title: "A", clipDuration: 60 });
      g.node("b", TITLE, 0, 1, { title: "B", clipDuration: 60 });
      g.node("seq", "timeline.sequence", 2, 0, { transition: "none" });
      g.link("a", "seq", "clip", "in1").link("b", "seq", "clip", "in2");
      return "seq";
    });
    const first = flat(wf).bars.find((b) => b.nodeId === "a")!;
    const next = applied(splitBar(ctxFor(wf), first, 20));
    expect(next.nodes.filter((n) => n.type === "timeline.sequence")).toHaveLength(1);
    const bars = flat(next).bars.toSorted((a, b) => a.from - b.from);
    expect(bars.map((b) => [Math.round(b.from), Math.round(b.to)])).toEqual([
      [0, 20],
      [20, 60],
      [60, 120],
    ]);
  });

  it("splits in place inside a sequence with a transition, keeping each cut on its clip", () => {
    const wf = build((g) => {
      g.node("a", TITLE, 0, 0, { title: "A", clipDuration: 60 });
      g.node("b", TITLE, 0, 1, { title: "B", clipDuration: 60 });
      g.node("c", TITLE, 0, 2, { title: "C", clipDuration: 60 });
      g.node("seq", "timeline.sequence", 2, 0, {
        transition: "core/fade",
        transitionDuration: 10,
        cuts: [
          { before: "in3", transition: "core/wipe", transitionDuration: 20, keepWhole: false },
        ],
      });
      g.link("a", "seq", "clip", "in1")
        .link("b", "seq", "clip", "in2")
        .link("c", "seq", "clip", "in3");
      return "seq";
    });
    const first = flat(wf).bars.find((b) => b.nodeId === "a")!;
    const next = applied(splitBar(ctxFor(wf), first, 20));
    const seqs = next.nodes.filter((n) => n.type === "timeline.sequence");
    expect(seqs).toHaveLength(1);
    expect(seqs[0].params.cuts).toEqual([
      { before: "in4", transition: "core/wipe", transitionDuration: 20, keepWhole: false },
      { before: "in2", transition: "none", transitionDuration: 15, keepWhole: false },
    ]);
    const marks = flat(next).transitions.map((t) => [t.kind, t.slot?.port]);
    expect(marks).toEqual([
      ["core/fade", "in3"],
      ["core/wipe", "in4"],
    ]);
    const bars = flat(next).bars.toSorted((x, y) => x.from - y.from);
    expect(bars.map((x) => [Math.round(x.from), Math.round(x.to)])).toEqual([
      [0, 20],
      [20, 60],
      [50, 110],
      [90, 150],
    ]);
  });

  it("keeps a cut's own transition when rolling it", () => {
    const wf = build((g) => {
      g.node("a", TITLE, 0, 0, { title: "A", clipDuration: 60 });
      g.node("b", TITLE, 0, 1, { title: "B", clipDuration: 60 });
      g.node("seq", "timeline.sequence", 2, 0, {
        transition: "none",
        cuts: [{ before: "in2", transition: "core/wipe", transitionDuration: 10, keepWhole: true }],
      });
      g.link("a", "seq", "clip", "in1").link("b", "seq", "clip", "in2");
      return "seq";
    });
    const bars = flat(wf).bars;
    const a = bars.find((b) => b.nodeId === "a")!;
    const b = bars.find((x) => x.nodeId === "b")!;
    const next = applied(rollCut(ctxFor(wf), a, b, 12));
    expect(next.nodes.find((n) => n.id === "seq")!.params.cuts).toEqual(
      wf.nodes.find((n) => n.id === "seq")!.params.cuts,
    );
    expect(flat(next).transitions.map((t) => t.kind)).toEqual(["core/wipe"]);
  });

  it("rolls the cut between two clips without changing the total", () => {
    const wf = build((g) => {
      g.node("a", TITLE, 0, 0, { title: "A", clipDuration: 60 });
      g.node("b", TITLE, 0, 1, { title: "B", clipDuration: 60 });
      g.node("seq", "timeline.sequence", 2, 0, { transition: "none" });
      g.link("a", "seq", "clip", "in1").link("b", "seq", "clip", "in2");
      return "seq";
    });
    const bars = flat(wf).bars;
    const a = bars.find((b) => b.nodeId === "a")!;
    const b = bars.find((x) => x.nodeId === "b")!;
    const next = applied(rollCut(ctxFor(wf), a, b, 12));
    const after = flat(next);
    const a2 = after.bars.find((x) => x.nodeId === "a")!;
    const b2 = after.bars.find((x) => x.nodeId === "b")!;
    expect(a2.to).toBeCloseTo(72, 0);
    expect(b2.from).toBeCloseTo(72, 0);
    expect(b2.to).toBeCloseTo(120, 0);
    expect(rollCut(ctxFor(wf), a, b, -12).ok).toBe(false);
  });

  it("won't move a sequenced clip before the one before it", () => {
    const wf = build((g) => {
      g.node("a", TITLE, 0, 0, { clipDuration: 60 });
      g.node("b", TITLE, 0, 1, { clipDuration: 60 });
      g.node("seq", "timeline.sequence", 2, 0, { transition: "none" });
      g.link("a", "seq", "clip", "in1").link("b", "seq", "clip", "in2");
      return "seq";
    });
    const b = flat(wf).bars.find((x) => x.nodeId === "b")!;
    const r = moveBar(ctxFor(wf), b, -10);
    expect(r.ok).toBe(false);
  });

  it("removes the nodes that only feed one clip", () => {
    const wf = build((g) => {
      g.node("a", TITLE, 0, 0, { clipDuration: 60 });
      g.node("trim", "timeline.trim", 1, 0, { startMs: 0, durationMs: 1000 });
      g.node("b", TITLE, 0, 1, { clipDuration: 60 });
      g.node("seq", "timeline.sequence", 2, 0, { transition: "none" });
      g.link("a", "trim").link("trim", "seq", "clip", "in1").link("b", "seq", "clip", "in2");
      return "seq";
    });
    const a = flat(wf).bars.find((x) => x.nodeId === "a")!;
    expect(removalFor(ctxFor(wf), a).nodeIds).toEqual(["a", "trim"]);
  });
});

describe("audio edits", () => {
  const withSound = () =>
    build((g) => {
      g.node("t", TITLE, 0, 0, { clipDuration: 300 });
      g.node("music", "audio.music", 0, 1, { src: "assets/song.mp3", startMs: 1000, fadeInMs: 0 });
      g.node("fx", "audio.sfx", 0, 2, { src: "app:sfx/whoosh.wav", atMs: 2000 });
      g.node("mix", "audio.mix", 2, 0);
      g.link("t", "mix")
        .link("music", "mix", "audio", "sounds")
        .link("fx", "mix", "audio", "sounds");
      return "mix";
    });

  it("moves, trims and sets the volume of music and sound effects", () => {
    const wf = withSound();
    const before = flat(wf);
    const music = before.audio.find((a) => a.nodeId === "music")!;
    const fx = before.audio.find((a) => a.nodeId === "fx")!;
    expect(music.from).toBe(30);
    expect(fx.from).toBe(60);

    const moved = flat(applied(moveAudio(ctxFor(wf), fx, DELTA))).audio.find(
      (a) => a.nodeId === "fx",
    )!;
    expect(moved.from - fx.from).toBe(DELTA);

    const trimmed = flat(applied(trimAudio(ctxFor(wf), music, "start", DELTA))).audio.find(
      (a) => a.nodeId === "music",
    )!;
    expect(trimmed.from - music.from).toBe(DELTA);
    expect(trimmed.trimBefore - music.trimBefore).toBe(DELTA);
    expect(len(music) - len(trimmed)).toBe(DELTA);

    const ended = flat(applied(trimAudio(ctxFor(wf), music, "end", -DELTA))).audio.find(
      (a) => a.nodeId === "music",
    )!;
    expect(len(music) - len(ended)).toBe(DELTA);

    const louder = applied(setAudioVolume(ctxFor(wf), fx, 1.5));
    expect(louder.nodes.find((n) => n.id === "fx")!.params.volume).toBe(1.5);
  });

  it("doesn't trim sound effects", () => {
    const wf = withSound();
    const fx = flat(wf).audio.find((a) => a.nodeId === "fx")!;
    expect(trimAudio(ctxFor(wf), fx, "end", -3).ok).toBe(false);
  });
});

describe("flattening", () => {
  it("maps bars through speed and trims", () => {
    const wf = build((g) => {
      g.node("t", TITLE, 0, 0, { clipDuration: 120 });
      g.node("trim", "timeline.trim", 1, 0, { startMs: 1000, durationMs: 0 });
      g.node("speed", "timeline.speed", 2, 0, { rate: 2 });
      g.link("t", "trim").link("trim", "speed");
      return "speed";
    });
    const [bar] = flat(wf).bars;
    expect(bar.from).toBeCloseTo(0, 3);
    expect(bar.to).toBeCloseTo(45, 0);
    expect(bar.rate).toBeCloseTo(2, 3);
    expect(bar.contentStart).toBeCloseTo(30, 0);
    expect(bar.chain.map((l) => l.nodeId).filter(Boolean)).toEqual(["t", "trim", "speed"]);
  });

  it("stacks overlapping layers on separate lanes in drawing order", () => {
    const wf = build((g) => {
      g.node("a", TITLE, 0, 0, { clipDuration: 90 });
      g.node("b", TITLE, 0, 1, { clipDuration: 30 });
      g.node("layers", "timeline.stack", 2, 0, { offsetsMs: "0,500" });
      g.link("a", "layers", "clip", "in1").link("b", "layers", "clip", "in2");
      return "layers";
    });
    const f = flat(wf);
    expect(f.lanes).toBe(2);
    const b = f.bars.find((x) => x.nodeId === "b") as VideoBar;
    expect(b.lane).toBe(1);
    expect(b.from).toBeCloseTo(15, 0);
    expect(findHandle(b, "move", "outer")?.link.handles?.move).toMatchObject({
      nodeId: "layers",
      param: "offsetsMs",
      index: 1,
    });
  });

  it("repeats bars for every loop", () => {
    const wf = build((g) => {
      g.node("t", TITLE, 0, 0, { clipDuration: 30 });
      g.node("loop", "timeline.loop", 1, 0, { times: 3 });
      g.link("t", "loop");
      return "loop";
    });
    const f = flat(wf);
    expect(f.bars.map((b) => [Math.round(b.from), Math.round(b.to)])).toEqual([
      [0, 30],
      [30, 60],
      [60, 90],
    ]);
  });

  it("marks transitions between sequenced clips", () => {
    const wf = build((g) => {
      g.node("a", TITLE, 0, 0, { clipDuration: 60 });
      g.node("b", TITLE, 0, 1, { clipDuration: 60 });
      g.node("seq", "timeline.sequence", 2, 0, { transition: "core/fade", transitionDuration: 15 });
      g.link("a", "seq", "clip", "in1").link("b", "seq", "clip", "in2");
      return "seq";
    });
    const f = flat(wf);
    expect(f.transitions).toHaveLength(1);
    expect(f.transitions[0]).toMatchObject({
      from: 45,
      to: 60,
      kind: "core/fade",
      slot: { nodeId: "seq", port: "in2" },
    });
  });
});
