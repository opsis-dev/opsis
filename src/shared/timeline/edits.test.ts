import { describe, expect, it } from "vitest";
import { randomId } from "../ids";
import { findById } from "../lists";
import { NODE_DEFS } from "../nodes/registry";
import type { EditHandleKind } from "../schema/timeline";
import type { Workflow } from "../schema/workflow";
import type { GraphBuilder } from "../templates/build";
import { buildFilm, flatFilm, must, TITLE } from "../test-fixtures";
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
import type { AudioBar, ChainLink, FlatTimeline, VideoBar } from "./flatten";
import { findHandle } from "./handles";

function ctxFor(workflow: Workflow): EditContext {
  const salt = randomId();
  let made = 0;
  return {
    workflow,
    fps: workflow.settings.fps,
    sourceNodeId: "out",
    makeId: (prefix) => `${prefix}_${salt}${++made}`,
  };
}

const applied = (r: EditResult): Workflow => {
  if (!r.ok) throw new Error(r.error);
  return r.workflow;
};

const len = (b: { from: number; to: number }) => b.to - b.from;

const barOf = (flat: FlatTimeline, nodeId: string): VideoBar =>
  must(
    flat.bars.find((b) => b.nodeId === nodeId),
    `bar for ${nodeId}`,
  );

const audioOf = (flat: FlatTimeline, nodeId: string): AudioBar =>
  must(
    flat.audio.find((a) => a.nodeId === nodeId),
    `sound for ${nodeId}`,
  );

const byStart = (flat: FlatTimeline) =>
  flat.bars.toSorted((a, b) => a.from - b.from).map((b) => [Math.round(b.from), Math.round(b.to)]);

function sequenceOf(clips: Array<[id: string, frames: number]>, params = {}): Workflow {
  return buildFilm((g) => {
    clips.forEach(([id, frames], i) => {
      g.node(id, TITLE, 0, i, { title: id.toUpperCase(), clipDuration: frames });
      g.link(id, "seq", "clip", `in${i + 1}`);
    });
    g.node("seq", "timeline.sequence", 2, 0, { transition: "none", ...params });
    return "seq";
  });
}

const DELTA = 6;

type HandleCase = {
  act: (ctx: EditContext, bar: VideoBar, link: ChainLink) => EditResult;
  expected: (link: ChainLink) => number;
  measured: (before: VideoBar, after: VideoBar) => number;
  tolerance: number;
};

const halfOf = (link: ChainLink) => Math.round(len(link) / 2);

const BAR_HANDLES = [
  "move",
  "trimStart",
  "trimEnd",
  "speed",
] as const satisfies readonly EditHandleKind[];

type BarHandle = (typeof BAR_HANDLES)[number];

const HANDLES: Record<BarHandle, HandleCase> = {
  move: {
    act: (ctx, bar) => moveBar(ctx, bar, DELTA),
    expected: () => DELTA,
    measured: (before, after) => after.from - before.from,
    tolerance: 0.5,
  },
  trimStart: {
    act: (ctx, bar) => trimBar(ctx, bar, "start", DELTA),
    expected: () => DELTA,
    measured: (before, after) => len(before) - len(after),
    tolerance: 0.5,
  },
  trimEnd: {
    act: (ctx, bar) => trimBar(ctx, bar, "end", -DELTA),
    expected: () => DELTA,
    measured: (before, after) => len(before) - len(after),
    tolerance: 0.5,
  },
  speed: {
    act: (ctx, bar, link) => speedBar(ctx, bar, halfOf(link)),
    expected: halfOf,
    measured: (_before, after) => len(after),
    tolerance: 1,
  },
};

function feedInputs(g: GraphBuilder, id: string, type: string): void {
  const def = must(
    NODE_DEFS.find((d) => d.type === type),
    `node def ${type}`,
  );
  Object.entries(def.inputs).forEach(([port, spec], i) => {
    const src = `${id}_${port}`;
    if (spec.type === "clip") {
      g.node(src, TITLE, 0, i + 1, { title: `Clip ${i + 1}`, clipDuration: 90 + i * 15 });
      g.link(src, id, "clip", port);
    }
    if (spec.type === "media") {
      g.node(src, "media.import", 0, i + 1, { path: "assets/clip.mp4" });
      g.link(src, id, "media", port);
    }
  });
}

type RoundTrip = {
  kind: BarHandle;
  owner: string;
  off: number;
  tolerance: number;
  addedNodes: number;
};

function roundTrips(type: string): RoundTrip[] {
  const wf = buildFilm((g) => {
    g.node("x", type, 3, 0);
    feedInputs(g, "x", type);
    return "x";
  });
  const before = flatFilm(wf);
  return before.bars.flatMap((bar) =>
    BAR_HANDLES.flatMap((kind) => {
      const owner = findHandle(bar, kind, "outer");
      const ref = owner?.handles?.[kind];
      const node = findById(wf.nodes, ref?.nodeId);
      if (!owner || !node) return [];
      const handle = HANDLES[kind];
      const next = applied(handle.act(ctxFor(wf), bar, owner));
      const after = must(
        flatFilm(next).bars.find((b) => b.key === bar.key),
        `${type} bar after its ${kind} edit`,
      );
      const off = handle.measured(bar, after) - handle.expected(owner);
      const addedNodes = next.nodes.length - wf.nodes.length;
      return [{ kind, owner: node.type, off, tolerance: handle.tolerance, addedNodes }];
    }),
  );
}

describe("timeline edit handles round-trip", () => {
  const clipNodes = NODE_DEFS.filter((d) => d.compile && d.outputs.clip && !d.hidden);
  const trips = new Map(clipNodes.map((d) => [d.type, roundTrips(d.type)]));
  const tripsOf = (type: string): RoundTrip[] => trips.get(type) ?? [];

  it.each(clipNodes.map((d) => [d.type]))("%s: every handle moves its bar by the delta", (type) => {
    const misses = tripsOf(type)
      .filter((t) => t.addedNodes !== 0 || Math.abs(t.off) > t.tolerance)
      .map((t) => `${t.kind}: off by ${t.off} frames, ${t.addedNodes} nodes added`);
    expect(misses).toEqual([]);
  });

  it("covers every kind of handle", () => {
    const owners = (kind: BarHandle) =>
      clipNodes.flatMap((d) => tripsOf(d.type).filter((t) => t.kind === kind)).map((t) => t.owner);
    expect(owners("move")).toContain("timeline.delay");
    expect(owners("trimStart")).toContain("timeline.trim");
    expect(owners("trimEnd")).toContain("timeline.trim");
    expect(owners("trimEnd")).toContain(TITLE);
    expect(owners("speed")).toContain("timeline.speed");
  });
});

const single = () =>
  buildFilm((g) => g.node("t", TITLE, 0, 0, { title: "Hello", clipDuration: 90 }));

describe("timeline edits that add nodes", () => {
  it("adds a Delay to move a clip that has none", () => {
    const wf = single();
    const [bar] = flatFilm(wf).bars;
    const r = moveBar(ctxFor(wf), bar, 12);
    const next = applied(r);
    expect(
      next.nodes.some((n) => n.type === "timeline.delay" && n.label === "Delay from timeline"),
    ).toBe(true);
    expect(flatFilm(next).bars[0].from).toBeCloseTo(12, 0);
    expect(r.ok && r.message).toMatch(/Delay from timeline/);
  });

  it("moves a sped-up clip by the frames dragged", () => {
    const wf = buildFilm((g) => {
      g.node("t", TITLE, 0, 0, { clipDuration: 90 });
      g.node("speed", "timeline.speed", 1, 0, { rate: 2 });
      g.link("t", "speed");
      return "speed";
    });
    const next = applied(moveBar(ctxFor(wf), barOf(flatFilm(wf), "t"), 12));
    expect(next.nodes.find((n) => n.type === "timeline.delay")?.params.ms).toBe(400);
    expect(barOf(flatFilm(next), "t").from).toBeCloseTo(12, 0);
  });

  it("adds a Trim to trim the start of a clip that has none", () => {
    const wf = single();
    const [bar] = flatFilm(wf).bars;
    const next = applied(trimBar(ctxFor(wf), bar, "start", 30));
    expect(next.nodes.some((n) => n.type === "timeline.trim")).toBe(true);
    expect(len(flatFilm(next).bars[0])).toBeCloseTo(60, 0);
  });

  it("adds a Speed to stretch a clip", () => {
    const wf = single();
    const [bar] = flatFilm(wf).bars;
    const next = applied(speedBar(ctxFor(wf), bar, 45));
    expect(next.nodes.find((n) => n.type === "timeline.speed")?.params.rate).toBe(2);
    expect(len(flatFilm(next).bars[0])).toBeCloseTo(45, 0);
  });

  it("speeds a clip up to 64× and no further", () => {
    const wf = single();
    const [bar] = flatFilm(wf).bars;
    const rate = (w: Workflow) => w.nodes.find((n) => n.type === "timeline.speed")?.params.rate;
    const fast = applied(speedBar(ctxFor(wf), bar, 2));
    expect(rate(fast)).toBe(45);
    expect(rate(applied(speedBar(ctxFor(fast), flatFilm(fast).bars[0], 1)))).toBe(64);
    expect(rate(applied(speedBar(ctxFor(wf), bar, 1)))).toBe(64);
  });

  it("slips the content without changing the length", () => {
    const wf = buildFilm((g) => {
      g.node("t", TITLE, 0, 0, { clipDuration: 90 });
      g.node("trim", "timeline.trim", 1, 0, { startMs: 0, durationMs: 1000 });
      g.link("t", "trim");
      return "trim";
    });
    const [trimmed] = flatFilm(wf).bars;
    expect(len(trimmed)).toBeCloseTo(30, 0);
    const slipped = flatFilm(applied(slipBar(ctxFor(wf), trimmed, 10))).bars[0];
    expect(len(slipped)).toBeCloseTo(30, 0);
    expect(slipped.contentStart - trimmed.contentStart).toBeCloseTo(10, 0);
  });

  it("adds a Trim to slip a clip that has none", () => {
    const wf = single();
    const [bar] = flatFilm(wf).bars;
    const next = applied(slipBar(ctxFor(wf), bar, 10));
    expect(next.nodes.some((n) => n.type === "timeline.trim")).toBe(true);
  });

  it("won't move a sequenced clip before the one before it", () => {
    const wf = sequenceOf([
      ["a", 60],
      ["b", 60],
    ]);
    expect(moveBar(ctxFor(wf), barOf(flatFilm(wf), "b"), -10).ok).toBe(false);
  });
});

describe("timeline splits and cuts", () => {
  it("splits a clip at the playhead into two parts in a sequence", () => {
    const wf = single();
    const [bar] = flatFilm(wf).bars;
    const next = applied(splitBar(ctxFor(wf), bar, 30));
    const after = flatFilm(next);
    expect(byStart(after)).toEqual([
      [0, 30],
      [30, 90],
    ]);
    expect(after.bars.toSorted((a, b) => a.from - b.from)[1].contentStart).toBeCloseTo(30, 0);
    const seq = next.nodes.find((n) => n.type === "timeline.sequence");
    expect(seq?.params.transition).toBe("none");
  });

  it("splits inside a cut-only sequence without nesting another one", () => {
    const wf = sequenceOf([
      ["a", 60],
      ["b", 60],
    ]);
    const next = applied(splitBar(ctxFor(wf), barOf(flatFilm(wf), "a"), 20));
    expect(next.nodes.filter((n) => n.type === "timeline.sequence")).toHaveLength(1);
    expect(byStart(flatFilm(next))).toEqual([
      [0, 20],
      [20, 60],
      [60, 120],
    ]);
  });

  it("splits in place inside a sequence with a transition, keeping each cut on its clip", () => {
    const wf = sequenceOf(
      [
        ["a", 60],
        ["b", 60],
        ["c", 60],
      ],
      {
        transition: "core/fade",
        transitionDuration: 10,
        cuts: [
          { before: "in3", transition: "core/wipe", transitionDuration: 20, keepWhole: false },
        ],
      },
    );
    const next = applied(splitBar(ctxFor(wf), barOf(flatFilm(wf), "a"), 20));
    const seqs = next.nodes.filter((n) => n.type === "timeline.sequence");
    expect(seqs).toHaveLength(1);
    expect(seqs[0].params.cuts).toEqual([
      { before: "in4", transition: "core/wipe", transitionDuration: 20, keepWhole: false },
      { before: "in2", transition: "none", transitionDuration: 15, keepWhole: false },
    ]);
    const after = flatFilm(next);
    expect(after.transitions.map((t) => [t.kind, t.slot?.port])).toEqual([
      ["core/fade", "in3"],
      ["core/wipe", "in4"],
    ]);
    expect(byStart(after)).toEqual([
      [0, 20],
      [20, 60],
      [50, 110],
      [90, 150],
    ]);
  });

  it("keeps a cut's own transition when rolling it", () => {
    const wf = sequenceOf(
      [
        ["a", 60],
        ["b", 60],
      ],
      {
        cuts: [{ before: "in2", transition: "core/wipe", transitionDuration: 10, keepWhole: true }],
      },
    );
    const before = flatFilm(wf);
    const next = applied(rollCut(ctxFor(wf), barOf(before, "a"), barOf(before, "b"), 12));
    const cuts = (w: Workflow) => findById(w.nodes, "seq")?.params.cuts;
    expect(cuts(next)).toEqual(cuts(wf));
    expect(flatFilm(next).transitions.map((t) => t.kind)).toEqual(["core/wipe"]);
  });

  it("rolls the cut between two clips without changing the total", () => {
    const wf = sequenceOf([
      ["a", 60],
      ["b", 60],
    ]);
    const before = flatFilm(wf);
    const [a, b] = [barOf(before, "a"), barOf(before, "b")];
    const after = flatFilm(applied(rollCut(ctxFor(wf), a, b, 12)));
    expect(barOf(after, "a").to).toBeCloseTo(72, 0);
    expect(barOf(after, "b").from).toBeCloseTo(72, 0);
    expect(barOf(after, "b").to).toBeCloseTo(120, 0);
    expect(rollCut(ctxFor(wf), a, b, -12).ok).toBe(false);
  });
});

describe("removing clips from the timeline", () => {
  it("removes the nodes that only feed one clip", () => {
    const wf = buildFilm((g) => {
      g.node("a", TITLE, 0, 0, { clipDuration: 60 });
      g.node("trim", "timeline.trim", 1, 0, { startMs: 0, durationMs: 1000 });
      g.node("b", TITLE, 0, 1, { clipDuration: 60 });
      g.node("seq", "timeline.sequence", 2, 0, { transition: "none" });
      g.link("a", "trim").link("trim", "seq", "clip", "in1").link("b", "seq", "clip", "in2");
      return "seq";
    });
    expect(removalFor(ctxFor(wf), barOf(flatFilm(wf), "a")).nodeIds).toEqual(["a", "trim"]);
  });
});

describe("audio edits", () => {
  const withSound = () =>
    buildFilm((g) => {
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
    const before = flatFilm(wf);
    const music = audioOf(before, "music");
    const fx = audioOf(before, "fx");
    expect(music.from).toBe(30);
    expect(fx.from).toBe(60);

    const moved = audioOf(flatFilm(applied(moveAudio(ctxFor(wf), fx, DELTA))), "fx");
    expect(moved.from - fx.from).toBe(DELTA);

    const trimmed = audioOf(
      flatFilm(applied(trimAudio(ctxFor(wf), music, "start", DELTA))),
      "music",
    );
    expect(trimmed.from - music.from).toBe(DELTA);
    expect(trimmed.trimBefore - music.trimBefore).toBe(DELTA);
    expect(len(music) - len(trimmed)).toBe(DELTA);

    const ended = audioOf(flatFilm(applied(trimAudio(ctxFor(wf), music, "end", -DELTA))), "music");
    expect(len(music) - len(ended)).toBe(DELTA);

    const louder = applied(setAudioVolume(ctxFor(wf), fx, 1.5));
    expect(findById(louder.nodes, "fx")?.params.volume).toBe(1.5);
  });

  it("doesn't trim sound effects", () => {
    const wf = withSound();
    expect(trimAudio(ctxFor(wf), audioOf(flatFilm(wf), "fx"), "end", -3).ok).toBe(false);
  });
});

describe("owned handles", () => {
  it("moves through a new node when the move setting's node is gone, even if another handle's node is there", () => {
    const wf = sequenceOf([
      ["a", 60],
      ["b", 60],
    ]);
    const bar = barOf(flatFilm(wf), "b");
    const owner = must(findHandle(bar, "trimEnd", "outer"), "trim end handle");
    const stale: ChainLink = {
      ...owner,
      handles: { ...owner.handles, move: { nodeId: "gone", param: "ms", unit: "ms" } },
    };
    const staleBar: VideoBar = { ...bar, chain: bar.chain.map((l) => (l === owner ? stale : l)) };
    const moved = applied(moveBar(ctxFor(wf), staleBar, DELTA));
    expect(moved.nodes.filter((n) => n.type === "timeline.delay")).toHaveLength(1);
  });
});
