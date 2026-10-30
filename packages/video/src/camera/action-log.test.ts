import { describe, expect, it } from "vitest";
import { resolveAnnotations, toClickLog } from "#video/camera/action-log.ts";
import {
  buildCameraTrack,
  type CameraTuning,
  clusterize,
  sampleTrack,
  NO_TUNING,
  tunedFrom,
} from "#video/camera/lib/camera-track.ts";
import { BASE_POSE } from "#video/camera/lib/camera.ts";
import { isInFrame } from "#video/camera/lib/test-fixtures.ts";
import { rectCenter } from "#video/common/geometry.ts";
import type { ActionLog, Rect } from "#video/generated/spec.ts";

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

  it("carries a double click's second press and a right click's button", () => {
    const clicks = toClickLog(
      {
        ...log,
        actions: [
          {
            id: "d",
            kind: "click",
            t: 1300,
            tDown: 1000,
            tSecondDown: 1170,
            point: { x: 500, y: 400 },
          },
          {
            id: "r",
            kind: "click",
            t: 2100,
            tDown: 2000,
            button: "right",
            point: { x: 520, y: 420 },
          },
        ],
      },
      "demo",
    ).clicks;
    expect(clicks[0]).toMatchObject({ tDownMs: 1000, tSecondDownMs: 1170, button: undefined });
    expect(clicks[1]).toMatchObject({ tDownMs: 2000, tSecondDownMs: undefined, button: "right" });
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
    expect(clusterize(clicks, log.viewport, tunedFrom(NO_TUNING)).map((g) => g.length)).toEqual([
      1, 2,
    ]);
  });

  it("drives a camera track that leaves and returns to base", () => {
    const cl = toClickLog(log, "demo");
    const track = buildCameraTrack(cl, 1, {});
    expect(sampleTrack(track, 0, NO_TUNING)).toEqual(BASE_POSE);
    expect(sampleTrack(track, 2.6, NO_TUNING).scale).toBeGreaterThan(1.1);
    expect(sampleTrack(track, 9, NO_TUNING).scale).toBeCloseTo(1, 5);
  });

  it("honours the Auto zoom tuning", () => {
    const cl = toClickLog(log, "demo");
    const loose = Math.max(...buildCameraTrack(cl, 1, {}).map((k) => k.pose.scale));
    const tight = Math.max(...buildCameraTrack(cl, 1, { sMax: 1.2 }).map((k) => k.pose.scale));
    expect(tight).toBeLessThanOrEqual(1.2);
    expect(loose).toBeGreaterThan(tight);
  });

  it("gives the target more room, waits longer and moves slower when tuned to", () => {
    const cl = toClickLog(log, "demo");
    function most(tuning: CameraTuning) {
      return Math.max(...buildCameraTrack(cl, 1, tuning).map((k) => k.pose.scale));
    }
    expect(most({ roomPx: 300, firstMaxScale: 2.5 })).toBeLessThan(most({ firstMaxScale: 2.5 }));
    function last(tuning: CameraTuning) {
      const keys = buildCameraTrack(cl, 1, tuning);
      return keys[keys.length - 1].t;
    }
    expect(last({ pace: 2 })).toBeGreaterThanOrEqual(last({}));
    const keys = buildCameraTrack(cl, 1, {});
    const moving = keys.findIndex((k, i) => i > 0 && k.pose.scale !== keys[i - 1].pose.scale);
    const quarter = keys[moving - 1].t + (keys[moving].t - keys[moving - 1].t) / 4;
    function halfway(easing: CameraTuning["easing"]) {
      return sampleTrack(keys, quarter, { easing }).scale;
    }
    expect(halfway("linear")).not.toBe(halfway("snappy"));
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

  function dragLog(drop: Rect): ActionLog {
    return {
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
    };
  }

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
      const pose = sampleTrack(track, t, NO_TUNING);
      expect(pose.scale).toBeGreaterThan(1.05);
      for (const r of [pick, drop]) {
        expect(isInFrame(pose, log.viewport, r.x, r.y)).toBe(true);
        expect(isInFrame(pose, log.viewport, r.x + r.w, r.y + r.h)).toBe(true);
      }
    }
  });
});
