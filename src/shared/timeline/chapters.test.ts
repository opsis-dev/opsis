import { describe, expect, it } from "vitest";
import { compileWorkflow } from "../compile/compile";
import type { Action, ActionLog } from "../schema/actions";
import { GraphBuilder } from "../templates/build";
import { chapterList, chapterMetadata, filmChapters } from "./chapters";
import { recordedTake, recordResult } from "../test-fixtures";

const chapter = (id: string, t: number, label: string): Action => ({
  id,
  kind: "chapter",
  t,
  point: { x: 0, y: 0 },
  label,
});

const log: ActionLog = {
  viewport: { width: 1920, height: 1080 },
  dpr: 1,
  t0: 0,
  durationMs: 60_000,
  actions: [
    chapter("a", 2000, "Sign in"),
    chapter("b", 20_000, "Build"),
    chapter("c", 25_000, "Too short"),
    chapter("d", 45_000, "Ship"),
  ],
  cursorTrack: [{ t: 0, x: 960, y: 540 }],
};

function film(edit?: { type: string; params: Record<string, unknown> }) {
  const g = new GraphBuilder();
  let last = recordedTake(g);
  if (edit) {
    g.node("edit", edit.type, 4, 0, edit.params);
    g.link(last, "edit");
    last = "edit";
  }
  g.node("out", "output.render", 5, 0);
  g.link(last, "out", "clip", "timeline");
  const spec = compileWorkflow(g.build({ id: "x", name: "x" }), {
    results: { record: recordResult(log) },
  }).specs.out;
  return { spec, chapters: filmChapters(spec) };
}

const titleAndStartSeconds = (chapters: ReturnType<typeof filmChapters>) =>
  chapters.map((c) => [c.title, Math.round(c.startMs / 100) / 10]);

describe("filmChapters", () => {
  it("starts at zero and merges chapters shorter than ten seconds", () => {
    const { spec, chapters } = film();
    expect(titleAndStartSeconds(chapters)).toEqual([
      ["Sign in", 0],
      ["Build", 20],
      ["Ship", 45],
    ]);
    expect(chapters.at(-1)!.endMs).toBe(Math.round((spec.durationInFrames / spec.fps) * 1000));
    expect(chapters[0].endMs).toBe(chapters[1].startMs);
  });

  it("adds an opening chapter when the first one starts late", () => {
    const { chapters } = film({ type: "timeline.delay", params: { ms: 15_000 } });
    expect(titleAndStartSeconds(chapters)).toEqual([
      ["Intro", 0],
      ["Sign in", 17],
      ["Build", 35],
      ["Ship", 60],
    ]);
  });

  it("follows a clip sped up on the timeline, dropping a last chapter under ten seconds", () => {
    const { chapters } = film({ type: "timeline.speed", params: { rate: 2 } });
    expect(titleAndStartSeconds(chapters)).toEqual([
      ["Sign in", 0],
      ["Build", 10],
    ]);
  });

  it("leaves out chapters trimmed away", () => {
    const { chapters } = film({
      type: "timeline.trim",
      params: { startMs: 10_000, durationMs: 0 },
    });
    expect(titleAndStartSeconds(chapters)).toEqual([
      ["Intro", 0],
      ["Build", 10],
      ["Ship", 35],
    ]);
  });

  it("has nothing for a film without chapters", () => {
    const g = new GraphBuilder();
    g.node("t", "fx.core.title", 0, 0, { clipDuration: 90 });
    g.node("out", "output.render", 1, 0);
    g.link("t", "out", "clip", "timeline");
    expect(filmChapters(compileWorkflow(g.build({ id: "x", name: "x" })).specs.out)).toEqual([]);
  });
});

describe("chapter files", () => {
  const chapters = [
    { title: "Sign in", startMs: 0, endMs: 20_000 },
    { title: "Build = ship; #1", startMs: 20_000, endMs: 3_725_000 },
  ];

  it("writes timestamps for a video description", () => {
    expect(chapterList(chapters)).toBe("0:00 Sign in\n0:20 Build = ship; #1\n");
  });

  it("writes escaped chapter metadata", () => {
    expect(chapterMetadata(chapters)).toBe(
      [
        ";FFMETADATA1",
        "[CHAPTER]",
        "TIMEBASE=1/1000",
        "START=0",
        "END=20000",
        "title=Sign in",
        "[CHAPTER]",
        "TIMEBASE=1/1000",
        "START=20000",
        "END=3725000",
        "title=Build \\= ship\\; \\#1",
        "",
      ].join("\n"),
    );
  });
});
