import { describe, expect, it } from "vitest";
import { rectCenter, type ActionLog, type Rect } from "../../shared/schema/actions";
import { resolveAnnotations, toClickLog } from "./action-log";
import { BASE_POSE } from "./lib/camera";
import { buildCameraTrack, clusterize, sampleTrack } from "./lib/camera-track";
import { isInFrame } from "./lib/test-fixtures";

const log: ActionLog = {
  viewport: { width: 1920, height: 1080 },
  dpr: 2,
  t0: 0,
  durationMs: 9000,
  actions: [
    { id: "nav", kind: "navigate", t: 0 },
    {
      id: "a",
      kind: "click",
      t: 2000,
      tDepart: 1400,
      tDown: 1950,
      point: { x: 1500, y: 300 },
      rect: { x: 1440, y: 280, w: 120, h: 40 },
      label: "Save",
    },
    { id: "k", kind: "key", t: 4000, text: "Meta+K", label: "Palette" },
    {
      id: "b",
      kind: "type",
      t: 5000,
      tEnd: 6500,
      point: { x: 400, y: 800 },
      rect: { x: 200, y: 780, w: 400, h: 40 },
      label: "Search",
    },
  ],
};

describe("ActionLog → ClickLog", () => {
  it("turns off zoom on the take's own actions only, keeping zooms placed by camera nodes", () => {
    const zoomHere = {
      id: "z:a",
      kind: "focus" as const,
      t: 3000,
      point: { x: 960, y: 540 },
      zoomScale: 2,
      cluster: "z:zoom",
    };
    const cl = toClickLog(log, "demo", [zoomHere], { zoomActions: false });
    expect(cl.clicks.map((c) => [c.tMs, c.zoom])).toEqual([
      [2000, false],
      [3000, undefined],
      [5000, false],
    ]);
  });

  it("keeps pointer beats as clicks and chords as keys", () => {
    const cl = toClickLog(log, "demo");
    expect(cl.clicks.map((c) => c.label)).toEqual(["Save", "Search"]);
    expect(cl.clicks[0]).toMatchObject({
      tMs: 2000,
      tDepartMs: 1400,
      tDownMs: 1950,
      x: 1500,
      y: 300,
    });
    expect(cl.clicks[1].typeEndMs).toBe(6500);
    expect(cl.keys).toEqual([{ tMs: 4000, chord: "Meta+K", label: "Palette" }]);
  });

  it("treats an empty zoom group as no group", () => {
    const clicks = toClickLog(
      {
        ...log,
        actions: [
          { id: "a", kind: "click", t: 1000, point: { x: 500, y: 400 }, cluster: "a" },
          { id: "b", kind: "click", t: 2000, point: { x: 520, y: 420 }, cluster: "" },
          { id: "c", kind: "click", t: 2500, point: { x: 530, y: 430 } },
        ],
      },
      "groups",
    ).clicks;
    expect(clicks.map((c) => c.cluster)).toEqual(["a", undefined, undefined]);
    expect(clusterize(clicks, log.viewport).map((g) => g.length)).toEqual([1, 2]);
  });

  it("drives a camera track that leaves and returns to base", () => {
    const cl = toClickLog(log, "demo");
    const track = buildCameraTrack(cl, 1, {});
    expect(sampleTrack(track, 0)).toEqual(BASE_POSE);
    expect(sampleTrack(track, 2.6).scale).toBeGreaterThan(1.1);
    expect(sampleTrack(track, 9).scale).toBeCloseTo(1, 5);
  });

  it("honours the Auto zoom tuning", () => {
    const cl = toClickLog(log, "demo");
    const loose = Math.max(...buildCameraTrack(cl, 1, {}).map((k) => k.pose.scale));
    const tight = Math.max(...buildCameraTrack(cl, 1, { sMax: 1.2 }).map((k) => k.pose.scale));
    expect(tight).toBeLessThanOrEqual(1.2);
    expect(loose).toBeGreaterThan(tight);
  });

  it("pins annotations to matching beats", () => {
    const ann = resolveAnnotations(log, [
      {
        kind: "highlight",
        match: "save",
        text: "",
        color: "#fff",
        padding: 8,
        leadMs: 200,
        holdMs: 1000,
      },
    ]);
    expect(ann).toHaveLength(1);
    expect(ann[0].fromS).toBeCloseTo(1.2);
    expect(ann[0].toS).toBeCloseTo(3);
  });

  const dragLog = (drop: Rect): ActionLog => ({
    ...log,
    actions: [
      {
        id: "d",
        kind: "drag",
        t: 4000,
        tDepart: 2400,
        tDown: 3000,
        point: rectCenter(drop),
        rect: { x: 300, y: 300, w: 200, h: 80 },
        drop,
        label: "Card",
      },
    ],
  });

  it("frames a short drag as one event covering both ends", () => {
    const [event, ...rest] = toClickLog(dragLog({ x: 700, y: 320, w: 200, h: 200 }), "d").clicks;
    expect(rest).toEqual([]);
    expect(event).toMatchObject({
      tMs: 4000,
      tDepartMs: 2400,
      tDownMs: 3000,
      tUpMs: 4000,
      rect: { x: 300, y: 300, w: 600, h: 220 },
      x: 600,
      y: 410,
    });
  });

  it("splits a long drag into a pick-up and a drop", () => {
    const clicks = toClickLog(dragLog({ x: 1500, y: 800, w: 200, h: 200 }), "d").clicks;
    expect(clicks).toHaveLength(2);
    expect(clicks[0]).toMatchObject({
      tMs: 3000,
      tDownMs: 3000,
      tUpMs: 4000,
      rect: { x: 300, y: 300, w: 200, h: 80 },
    });
    expect(clicks[1]).toMatchObject({
      tMs: 4000,
      tDepartMs: 3000,
      rect: { x: 1500, y: 800, w: 200, h: 200 },
    });
    expect(clicks[1].tDownMs).toBeUndefined();
  });

  it("keeps both ends of a short drag in frame while dragging", () => {
    const pick = { x: 300, y: 300, w: 200, h: 80 };
    const drop = { x: 700, y: 320, w: 200, h: 200 };
    const track = buildCameraTrack(toClickLog(dragLog(drop), "d"), 1, {});
    for (const t of [3, 3.5, 4]) {
      const pose = sampleTrack(track, t);
      expect(pose.scale).toBeGreaterThan(1.05);
      for (const r of [pick, drop]) {
        expect(isInFrame(pose, log.viewport, r.x, r.y)).toBe(true);
        expect(isInFrame(pose, log.viewport, r.x + r.w, r.y + r.h)).toBe(true);
      }
    }
  });
});
