import { describe, expect, it } from "vitest";
import type { ActionLog } from "../schema/actions";
import type { TimelineSpec } from "../schema/timeline";
import type { NodeResult } from "../schema/values";
import { todoMvcSample } from "../templates";
import { GraphBuilder } from "../templates/build";
import type { GroupProps } from "./clip";
import { compileWorkflow, previewSpec } from "./compile";
import type { RecordingProps } from "./recording";
import { flattenSpec } from "../timeline/flatten";

const log: ActionLog = {
  viewport: { width: 1920, height: 1080 },
  dpr: 2,
  t0: 0,
  durationMs: 12_000,
  actions: [
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
  cursorTrack: [
    { t: 0, x: 960, y: 540 },
    { t: 1500, x: 960, y: 200 },
  ],
};

const recordResult = (hash = "h"): NodeResult => ({
  hash,
  outputs: {
    recording: {
      type: "recording",
      src: "project:.demoflow/cache/h/recording.mp4",
      log,
      durationMs: log.durationMs,
      viewport: log.viewport,
      nodeId: "record",
    },
  },
  artifacts: [],
  finishedAt: 0,
  durationMs: 0,
});

function walkItems(spec: TimelineSpec) {
  const items: Array<{ component: string; props: Record<string, unknown> }> = [];
  const visit = (tracks: TimelineSpec["tracks"]) => {
    for (const t of tracks)
      for (const i of t.items) {
        items.push(i);
        if (i.component === "core/group") visit((i.props as GroupProps).tracks);
      }
  };
  visit(spec.tracks);
  return items;
}

describe("compileWorkflow — sample project", () => {
  it("compiles to one spec per Output node, with a placeholder before the first run", () => {
    const wf = todoMvcSample();
    const out = compileWorkflow(wf);
    expect(out.errors).toEqual({});
    const spec = out.specs.render;
    expect(spec).toBeDefined();
    expect(spec.width).toBe(1920);
    const components = walkItems(spec).map((i) => i.component);
    expect(components).toContain("core/placeholder");
    expect(components).toContain("remocn/soft-blur-in");
    expect(components).toContain("remocn/shader-mesh-gradient");
    expect(components).toContain("core/title");
  });

  it("uses the recording once the capture node has a result", () => {
    const wf = todoMvcSample();
    const out = compileWorkflow(wf, { results: { record: recordResult() } });
    const rec = walkItems(out.specs.render).find((i) => i.component === "core/recording");
    expect(rec).toBeDefined();
    const props = rec!.props as RecordingProps;
    expect(props.src).toBe("project:.demoflow/cache/h/recording.mp4");
    expect(props.tuning.sMax).toBe(1.74);
    expect(props.keystrokes).toEqual({ position: "bottom", typed: true, chords: true });
  });

  it("marks every action of the recording on the timeline at its frame", () => {
    const wf = todoMvcSample();
    const out = compileWorkflow(wf, { results: { record: recordResult() } });
    const flat = flattenSpec(out.specs.render);
    const bar = flat.bars.find((b) => b.component === "core/recording")!;
    const marks = flat.marks.filter((m) => m.barKey === bar.key);
    expect(marks.map((m) => m.kind)).toEqual(["type", "click", "click"]);
    expect(marks[1].label).toBe("Click Complete");
    const speed = (bar.props as unknown as RecordingProps).speed;
    const local = (marks[1].frame - bar.from) / bar.rate;
    expect(local + bar.contentStart).toBeCloseTo((6000 / 1000 / speed) * 30, 0);
  });

  it("passes the small preview copy along with the full recording", () => {
    const result = recordResult();
    const recording = result.outputs.recording as Record<string, unknown>;
    recording.proxySrc = "project:.demoflow/cache/h/recording.preview.mp4";
    const out = compileWorkflow(todoMvcSample(), { results: { record: result } });
    const rec = walkItems(out.specs.render).find((i) => i.component === "core/recording");
    const props = rec!.props as RecordingProps;
    expect(props.src).toBe("project:.demoflow/cache/h/recording.mp4");
    expect(props.proxySrc).toBe("project:.demoflow/cache/h/recording.preview.mp4");
  });

  it("sequences clips with overlapping transitions (TransitionSeries semantics)", () => {
    const wf = todoMvcSample();
    const spec = compileWorkflow(wf, { results: { record: recordResult() } }).specs.render;
    const seq = spec.tracks[0].items;
    expect(seq).toHaveLength(3);
    expect(seq[1].enter?.kind).toBe("remocn/whip-pan");
    expect(seq[0].exit?.kind).toBe("remocn/whip-pan");
    expect(seq[1].from).toBe(
      seq[0].from + seq[0].durationInFrames - seq[1].enter!.durationInFrames,
    );
    expect(spec.durationInFrames).toBe(seq[2].from + seq[2].durationInFrames);
  });

  it("interpolates {{VARIABLES}} into params", () => {
    const wf = todoMvcSample();
    const spec = compileWorkflow(wf).specs.render;
    const outro = walkItems(spec).find((i) => i.component === "core/title");
    expect(outro?.props.subtitle).toBe("https://demo.playwright.dev/todomvc");
  });
});

describe("compileWorkflow — timeline nodes", () => {
  const titleWorkflow = (extra: (g: GraphBuilder, title: string) => string) => {
    const g = new GraphBuilder();
    const title = g.node("title", "fx.core.title", 0, 0, { clipDuration: 90 });
    const last = extra(g, title);
    return { wf: g.build({ id: "t", name: "T" }), last };
  };

  it("trims via a group with startFrom", () => {
    const { wf, last } = titleWorkflow((g, title) => {
      const trim = g.node("trim", "timeline.trim", 1, 0, { startMs: 1000, durationMs: 1000 });
      g.link(title, trim);
      return trim;
    });
    const out = compileWorkflow(wf);
    const clip = out.values[last].clip;
    expect(clip?.type).toBe("clip");
    if (clip?.type !== "clip") return;
    expect(clip.durationInFrames).toBe(30);
    expect((clip.tracks[0].items[0].props as GroupProps).startFrom).toBe(30);
  });

  it("speeds up with a linear remap", () => {
    const { wf, last } = titleWorkflow((g, title) => {
      const speed = g.node("speed", "timeline.speed", 1, 0, { rate: 2 });
      g.link(title, speed);
      return speed;
    });
    const clip = compileWorkflow(wf).values[last].clip;
    if (clip?.type !== "clip") throw new Error("no clip");
    expect(clip.durationInFrames).toBe(45);
    expect((clip.tracks[0].items[0].props as GroupProps).remap).toEqual([
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
    wf.nodes.find((n) => n.id === "loop")!.disabled = true;
    const clip = compileWorkflow(wf).values[last].clip;
    if (clip?.type !== "clip") throw new Error("no clip");
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
    expect(previewSpec(wf, out, "title")?.spec.durationInFrames).toBe(90);
    expect(previewSpec(wf, out, null)?.spec.durationInFrames).toBe(180);
    expect(previewSpec(wf, out, "title")?.spans.title).toEqual({ from: 0, to: 90 });
  });
});

describe("layers that keep their layout", () => {
  it("wrap the layers in a box at the project's size", () => {
    const g = new GraphBuilder();
    g.node("bg", "fx.core.solid", 0, 0, { clipDuration: 60 });
    g.node("t", "fx.core.title", 0, 1, { clipDuration: 60 });
    g.node("stack", "timeline.stack", 1, 0, { keepLayout: true });
    g.link("bg", "stack", "clip", "in1").link("t", "stack", "clip", "in2");
    g.node("out", "output.render", 2, 0);
    g.link("stack", "out", "clip", "timeline");
    const spec = compileWorkflow(g.build({ id: "x", name: "x" })).specs.out;
    const outer = spec.tracks[0].items[0];
    const props = outer.props as GroupProps;
    expect(props.design).toEqual({ width: 1920, height: 1080 });
    expect(props.tracks).toHaveLength(2);
    expect(
      flattenSpec(spec)
        .bars.map((b) => b.nodeId)
        .toSorted(),
    ).toEqual(["bg", "t"]);
  });
});

describe("sounds under a clip", () => {
  const take = (g: GraphBuilder) => {
    const s = g.node("session", "browser.session", 0, 0);
    const nav = g.node("open", "browser.navigate", 1, 0, { url: "https://example.com" });
    const click = g.node("complete", "browser.click", 2, 0, { target: "Complete" });
    const rec = g.node("record", "capture.record", 3, 0);
    g.chain([s, nav, click, rec]);
    const cam = g.node("camera", "camera.autoZoom", 3, 1, { speed: 1 });
    g.link(rec, cam, "recording", "recording");
    return cam;
  };
  const tagged: NodeResult = (() => {
    const r = recordResult();
    const rec = r.outputs.recording as { log: ActionLog };
    rec.log = {
      ...log,
      actions: log.actions.map((a) => (a.id === "b" ? { ...a, nodeId: "complete" } : a)),
    };
    return r;
  })();

  it("takes any number of sounds on one port", () => {
    const g = new GraphBuilder();
    const cam = take(g);
    g.node("mix", "audio.mix", 4, 1);
    g.link(cam, "mix");
    for (const id of ["s1", "s2", "s3", "s4"]) {
      g.node(id, "audio.sfx", 4, 2, { src: "app:sfx/pop.wav", atMs: 500 });
      g.link(id, "mix", "audio", "sounds");
    }
    g.node("out", "output.render", 5, 1);
    g.link("mix", "out", "clip", "timeline");
    const out = compileWorkflow(g.build({ id: "x", name: "x" }), { results: { record: tagged } });
    expect(out.specs.out.audio.map((a) => a.nodeId)).toEqual(["s1", "s2", "s3", "s4"]);
  });

  it("plays a sound effect on the action it's pinned to", () => {
    const g = new GraphBuilder();
    const cam = take(g);
    g.node("mix", "audio.mix", 4, 1);
    g.link(cam, "mix");
    g.node("pinned", "audio.sfx", 4, 2, {
      src: "app:sfx/pop.wav",
      atAction: "complete",
      offsetMs: -200,
    });
    g.link("pinned", "mix", "audio", "sounds");
    g.node("out", "output.render", 5, 1);
    g.link("mix", "out", "clip", "timeline");
    const out = compileWorkflow(g.build({ id: "x", name: "x" }), { results: { record: tagged } });
    const mark = flattenSpec(out.specs.out).marks.find((m) => m.nodeId === "complete")!;
    const sound = out.specs.out.audio.find((a) => a.nodeId === "pinned")!;
    expect(sound.from).toBe(Math.round(mark.frame) - 6);
  });
});
