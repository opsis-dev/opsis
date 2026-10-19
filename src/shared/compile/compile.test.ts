import { assert, describe, expect, it } from "vitest";
import { findById } from "../lists";
import type { TimelineSpec } from "../schema/timeline";
import type { Workflow } from "../schema/workflow";
import { getTemplate } from "../templates";
import { GraphBuilder } from "../templates/build";
import { itemsOf, makeLog, must, recordedTake, recordResult, TITLE } from "../test-fixtures";
import { flattenSpec } from "../timeline/flatten";
import { groupOf, isClip, recordingOf } from "./clip";
import { compileWorkflow, previewSpec } from "./compile";

const todoMvc = () => getTemplate("todomvc").build("TodoMVC demo");

const log = makeLog(
  [
    {
      id: "a",
      kind: "type",
      t: 1500,
      tEnd: 3200,
      point: { x: 960, y: 200 },
      rect: { x: 600, y: 170, w: 720, h: 60 },
      cluster: "add",
      text: "Record a real product flow",
    },
    {
      id: "b",
      kind: "click",
      t: 6000,
      tDown: 5900,
      tDepart: 5200,
      point: { x: 620, y: 270 },
      rect: { x: 600, y: 250, w: 40, h: 40 },
      cluster: "done",
      label: "Complete",
    },
    {
      id: "c",
      kind: "click",
      t: 8000,
      point: { x: 1100, y: 700 },
      rect: { x: 1060, y: 690, w: 90, h: 24 },
      cluster: "filter",
      label: "Completed filter",
    },
  ],
  12_000,
);

const recordingIn = (spec: TimelineSpec) => itemsOf(spec).map(recordingOf).find(Boolean);

describe("compileWorkflow — sample project", () => {
  it("compiles to one spec per Output node, with a placeholder before the first run", () => {
    const wf = todoMvc();
    const out = compileWorkflow(wf);
    expect(out.errors).toEqual({});
    const spec = out.specs.render;
    expect(spec).toBeDefined();
    expect(spec.width).toBe(1920);
    const components = itemsOf(spec).map((i) => i.component);
    expect(components).toContain("core/placeholder");
    expect(components).toContain("remocn/soft-blur-in");
    expect(components).toContain("remocn/shader-mesh-gradient");
    expect(components).toContain("core/title");
  });

  it("uses the recording once the capture node has a result", () => {
    const wf = todoMvc();
    const out = compileWorkflow(wf, { results: { record: recordResult(log) } });
    const props = recordingIn(out.specs.render);
    expect(props?.src).toBe("project:.opsis/cache/h/recording.mp4");
    expect(props?.tuning.sMax).toBe(1.74);
    expect(props?.keystrokes).toEqual({
      position: "bottom",
      typed: true,
      chords: true,
      keycap: "#0a0a0a",
      ink: "#ffffff",
      size: 1,
    });
  });

  it("marks every action of the recording on the timeline at its frame", () => {
    const wf = todoMvc();
    const out = compileWorkflow(wf, { results: { record: recordResult(log) } });
    const flat = flattenSpec(out.specs.render);
    const bar = must(
      flat.bars.find((b) => b.component === "core/recording"),
      "recording bar",
    );
    const marks = flat.marks.filter((m) => m.barKey === bar.key);
    expect(marks.map((m) => m.kind)).toEqual(["type", "click", "click"]);
    expect(marks[1].label).toBe("Click Complete");
    const speed = Number(bar.props.speed);
    const local = (marks[1].frame - bar.from) / bar.rate;
    expect(local + bar.contentStart).toBeCloseTo((6000 / 1000 / speed) * 30, 0);
  });

  it("passes the small preview copy along with the full recording", () => {
    const result = recordResult(log);
    const recording = result.outputs.recording;
    assert(recording.type === "recording", "recording output");
    recording.proxySrc = "project:.opsis/cache/h/recording.preview.mp4";
    const out = compileWorkflow(todoMvc(), { results: { record: result } });
    const props = recordingIn(out.specs.render);
    expect(props?.src).toBe("project:.opsis/cache/h/recording.mp4");
    expect(props?.proxySrc).toBe("project:.opsis/cache/h/recording.preview.mp4");
  });

  it("sequences clips with overlapping transitions (TransitionSeries semantics)", () => {
    const wf = todoMvc();
    const spec = compileWorkflow(wf, { results: { record: recordResult(log) } }).specs.render;
    const seq = spec.tracks[0].items;
    expect(seq).toHaveLength(3);
    const enter = must(seq[1].enter, "transition into the second clip");
    expect(enter.kind).toBe("remocn/whip-pan");
    expect(seq[0].exit?.kind).toBe("remocn/whip-pan");
    expect(seq[1].from).toBe(seq[0].from + seq[0].durationInFrames - enter.durationInFrames);
    expect(spec.durationInFrames).toBe(seq[2].from + seq[2].durationInFrames);
  });

  it("interpolates {{VARIABLES}} into params", () => {
    const wf = todoMvc();
    const spec = compileWorkflow(wf).specs.render;
    const outro = itemsOf(spec).find((i) => i.component === "core/title");
    expect(outro?.props.subtitle).toBe("https://demo.playwright.dev/todomvc");
  });
});

describe("compileWorkflow — timeline nodes", () => {
  const titleWorkflow = (extra: (g: GraphBuilder, title: string) => string) => {
    const g = new GraphBuilder();
    const title = g.node("title", TITLE, 0, 0, { clipDuration: 90 });
    const last = extra(g, title);
    return { wf: g.build({ id: "t", name: "T" }), last };
  };

  const compiledClip = (wf: Workflow, id: string) => {
    const clip = compileWorkflow(wf).values[id]?.clip;
    if (!isClip(clip)) throw new Error(`${id} has no clip`);
    return clip;
  };

  it("trims via a group with startFrom", () => {
    const { wf, last } = titleWorkflow((g, title) => {
      const trim = g.node("trim", "timeline.trim", 1, 0, { startMs: 1000, durationMs: 1000 });
      g.link(title, trim);
      return trim;
    });
    const clip = compiledClip(wf, last);
    expect(clip.durationInFrames).toBe(30);
    expect(groupOf(clip.tracks[0].items[0])?.startFrom).toBe(30);
  });

  it("speeds up with a linear remap", () => {
    const { wf, last } = titleWorkflow((g, title) => {
      const speed = g.node("speed", "timeline.speed", 1, 0, { rate: 2 });
      g.link(title, speed);
      return speed;
    });
    const clip = compiledClip(wf, last);
    expect(clip.durationInFrames).toBe(45);
    expect(groupOf(clip.tracks[0].items[0])?.remap).toEqual([
      [0, 0],
      [45, 90],
    ]);
  });

  it("bypasses disabled nodes", () => {
    const { wf, last } = titleWorkflow((g, title) => {
      const loop = g.node("loop", "timeline.loop", 1, 0, { times: 3 });
      g.link(title, loop);
      return loop;
    });
    must(findById(wf.nodes, "loop"), "loop node").disabled = true;
    const clip = compiledClip(wf, last);
    expect(clip.durationInFrames).toBe(90);
  });

  it("reports invalid params per node instead of throwing", () => {
    const { wf } = titleWorkflow((_g, title) => title);
    wf.nodes[0].params.fontSize = "huge";
    const out = compileWorkflow(wf);
    expect(out.errors.title).toBe("Font size isn't valid");
  });

  it("previews the selected node's clip", () => {
    const { wf } = titleWorkflow((g, title) => {
      const loop = g.node("loop", "timeline.loop", 1, 0, { times: 2 });
      g.link(title, loop);
      return loop;
    });
    const out = compileWorkflow(wf);
    expect(previewSpec(wf.settings, out, "title")?.spec.durationInFrames).toBe(90);
    expect(previewSpec(wf.settings, out, null)?.spec.durationInFrames).toBe(180);
    expect(previewSpec(wf.settings, out, "title")?.spans.title).toEqual({ from: 0, to: 90 });
  });
});

describe("layers that keep their layout", () => {
  it("wrap the layers in a box at the project's size", () => {
    const g = new GraphBuilder();
    g.node("bg", "fx.core.solid", 0, 0, { clipDuration: 60 });
    g.node("t", TITLE, 0, 1, { clipDuration: 60 });
    g.node("stack", "timeline.stack", 1, 0, { keepLayout: true });
    g.link("bg", "stack", "clip", "in1").link("t", "stack", "clip", "in2");
    g.node("out", "output.render", 2, 0);
    g.link("stack", "out", "clip", "timeline");
    const spec = compileWorkflow(g.build({ id: "x", name: "x" })).specs.out;
    const props = groupOf(spec.tracks[0].items[0]);
    expect(props?.design).toEqual({ width: 1920, height: 1080 });
    expect(props?.tracks).toHaveLength(2);
    expect(
      flattenSpec(spec)
        .bars.map((b) => b.nodeId)
        .toSorted(),
    ).toEqual(["bg", "t"]);
  });
});

describe("sounds under a clip", () => {
  const tagged = recordResult({
    ...log,
    actions: log.actions.map((a) => (a.id === "b" ? { ...a, nodeId: "complete" } : a)),
  });

  const withSounds = (sounds: Array<[id: string, params: Record<string, unknown>]>) => {
    const g = new GraphBuilder();
    const cam = recordedTake(g, [
      { id: "complete", type: "browser.click", params: { target: "Complete" } },
    ]);
    g.node("mix", "audio.mix", 4, 1);
    g.link(cam, "mix");
    for (const [id, params] of sounds) {
      g.node(id, "audio.sfx", 4, 2, { src: "app:sfx/pop.wav", ...params });
      g.link(id, "mix", "audio", "sounds");
    }
    g.node("out", "output.render", 5, 1);
    g.link("mix", "out", "clip", "timeline");
    return compileWorkflow(g.build({ id: "x", name: "x" }), { results: { record: tagged } }).specs
      .out;
  };

  it("takes any number of sounds on one port", () => {
    const spec = withSounds(["s1", "s2", "s3", "s4"].map((id) => [id, { atMs: 500 }]));
    expect(spec.audio.map((a) => a.nodeId)).toEqual(["s1", "s2", "s3", "s4"]);
  });

  it("plays a sound effect on the action it's pinned to", () => {
    const spec = withSounds([["pinned", { atAction: "complete", offsetMs: -200 }]]);
    const mark = must(
      flattenSpec(spec).marks.find((m) => m.nodeId === "complete"),
      "mark of the pinned action",
    );
    const sound = spec.audio.find((a) => a.nodeId === "pinned");
    expect(sound?.from).toBe(Math.round(mark.frame) - 6);
  });
});

describe("a transition at each cut", () => {
  const sequence = (cuts: unknown[], clips = ["in1", "in2", "in3"]) => {
    const g = new GraphBuilder();
    for (const [i, port] of clips.entries()) {
      g.node(`c${i}`, TITLE, 0, i, { clipDuration: 60 });
      g.link(`c${i}`, "seq", "clip", port);
    }
    g.node("seq", "timeline.sequence", 1, 0, {
      transition: "core/fade",
      transitionDuration: 10,
      cuts,
    });
    g.node("out", "output.render", 2, 0);
    g.link("seq", "out", "clip", "timeline");
    return compileWorkflow(g.build({ id: "x", name: "x" })).specs.out.tracks[0].items;
  };
  const enters = (items: TimelineSpec["tracks"][number]["items"]) =>
    items.map((i) => (i.enter ? [i.enter.kind, i.enter.durationInFrames, !!i.enter.hold] : null));

  it("uses each cut's own transition and length, and the default elsewhere", () => {
    const items = sequence([
      { before: "in3", transition: "core/wipe", transitionDuration: 20, keepWhole: true },
    ]);
    expect(enters(items)).toEqual([null, ["core/fade", 10, false], ["core/wipe", 20, true]]);
    expect(items[1].exit?.kind).toBe("core/wipe");
  });

  it("plays a plain cut where a cut says none", () => {
    const items = sequence([{ before: "in2", transition: "none", transitionDuration: 15 }]);
    expect(enters(items)).toEqual([null, null, ["core/fade", 10, false]]);
  });

  it("ignores a cut for a port with no clip, and the last entry for a port wins", () => {
    const items = sequence(
      [
        { before: "in5", transition: "core/wipe", transitionDuration: 20 },
        { before: "in2", transition: "core/slide", transitionDuration: 12 },
        { before: "in2", transition: "core/iris", transitionDuration: 8 },
      ],
      ["in1", "in2"],
    );
    expect(enters(items)).toEqual([null, ["core/iris", 8, false]]);
  });
});
