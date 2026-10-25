import { describe, expect, it } from "vitest";
import {
  buildCameraTrack,
  cursorAim,
  narrowPose,
  clusterize,
  frameFor,
  S_MAX,
  sampleTrack,
  stickify,
} from "#video/camera/lib/camera-track.ts";
import { BASE_POSE, posesNearlyEqual } from "#video/camera/lib/camera.ts";
import type { ClickEvent, ClickLog } from "#video/camera/lib/click-log.ts";
import { isInFrame } from "#video/camera/lib/test-fixtures.ts";
import { zoomAt } from "#video/camera/lib/zoom.ts";

const VP = { width: 1920, height: 1080 };

const click = (tMs: number, x: number, y: number, more: Partial<ClickEvent> = {}): ClickEvent => ({
  tMs,
  x,
  y,
  ...more,
});

const box = (x: number, y: number, w: number, h: number) => ({ rect: { x, y, w, h } });

const smokeLog: ClickLog = {
  name: "smoke",
  viewport: VP,
  durationMs: 11561,
  clicks: [
    click(2603, 533, 302, { tDepartMs: 1800, cluster: "priority", ...box(491, 289, 83, 25) }),
    click(4338, 571, 417, { tDepartMs: 3600, cluster: "priority", ...box(498, 399, 146, 36) }),
    click(6854, 558, 612, { tDepartMs: 6000, cluster: "done", ...box(458, 589, 200, 46) }),
  ],
};

const clusteredLog: ClickLog = {
  name: "clustered",
  viewport: VP,
  durationMs: 23325,
  clicks: [
    click(8073, 502, 466, { cluster: "language" }),
    click(9342, 559, 586, { cluster: "language" }),
    click(11220, 502, 630, { cluster: "domain" }),
    click(12401, 503, 713, { cluster: "domain" }),
    click(14229, 502, 710, { cluster: "category" }),
    click(18041, 1122, 505, { cluster: "payoff" }),
  ],
};

describe("clusterize", () => {
  it("splits one cluster id when its targets are far apart", () => {
    const groups = clusterize(
      [
        click(1000, 702, 314, { cluster: "c1" }),
        click(4000, 1308, 991, { cluster: "c1" }),
        click(6000, 1400, 1010, { cluster: "c1" }),
      ],
      VP,
    );
    expect(groups.length, "far-apart targets stayed in one cluster").toBe(2);
    expect(groups[0].length).toBe(1);
    expect(groups[1].length).toBe(2);
  });

  it("keeps a cluster together when its targets are close", () => {
    const groups = clusterize(
      [click(1000, 476, 159, { cluster: "c0" }), click(3000, 557, 236, { cluster: "c0" })],
      VP,
    );
    expect(groups.length).toBe(1);
  });

  it("groups clicks without a cluster by time and distance, even beside clusters", () => {
    const groups = clusterize(
      [
        click(1000, 500, 400, { cluster: "a" }),
        click(2000, 520, 420),
        click(3000, 1500, 900),
        click(4000, 1510, 910),
        click(7000, 1520, 920),
      ],
      VP,
    );
    expect(groups.map((g) => g.length)).toEqual([1, 1, 2, 1]);
  });
});

describe("stickify", () => {
  it("never merges different explicit cluster ids", () => {
    const groups = clusterize(clusteredLog.clicks, VP);
    const stuck = stickify(groups, VP);
    expect(stuck.length).toBe(4);
    expect(stuck.map((g) => g[0].cluster)).toEqual(["language", "domain", "category", "payoff"]);
  });
});

describe("buildCameraTrack", () => {
  it("starts at base and returns to base before the cut", () => {
    const keys = buildCameraTrack(smokeLog, 1, {});
    expect(keys[0].pose.scale).toBe(BASE_POSE.scale);
    const last = keys[keys.length - 1];
    expect(last.t).toBeCloseTo(smokeLog.durationMs / 1000, 1);
    expect(last.pose.scale).toBe(BASE_POSE.scale);
    expect(last.pose.cx).toBe(BASE_POSE.cx);
  });

  it("holds still at base for a beat before the cut", () => {
    const keys = buildCameraTrack(smokeLog, 1, {});
    const end = smokeLog.durationMs / 1000;
    const a = sampleTrack(keys, end - 0.3);
    const b = sampleTrack(keys, end);
    expect(a.scale).toBe(BASE_POSE.scale);
    expect(b.scale).toBe(BASE_POSE.scale);
  });

  it("routes through base instead of panning across the screen", () => {
    const keys = buildCameraTrack(
      {
        name: "wide",
        viewport: VP,
        durationMs: 16000,
        clicks: [
          click(3000, 300, 200, { tDepartMs: 2000, cluster: "one", ...box(240, 170, 200, 60) }),
          click(9000, 1700, 980, { tDepartMs: 8000, cluster: "one", ...box(1600, 950, 200, 60) }),
        ],
      },
      1,
      {},
    );
    const between = keys.filter((k) => k.t > 3 && k.t < 9);
    expect(
      between.some((k) => Math.abs(k.pose.scale - BASE_POSE.scale) < 1e-6),
      "camera panned across the screen instead of pulling back to base",
    ).toBe(true);
  });

  it("routes through base between distant clusters even with no room to trail", () => {
    const keys = buildCameraTrack(
      {
        name: "far",
        viewport: VP,
        durationMs: 24000,
        clicks: [
          click(16700, 1870, 1050, {
            tDepartMs: 13350,
            cluster: "save",
            ...box(1816, 1014, 112, 72),
          }),
          click(21470, 320, 148, {
            tDepartMs: 18180,
            cluster: "payoff",
            ...box(249, 112, 151, 72),
          }),
        ],
      },
      1.25,
      {},
    );
    const between = keys.filter((k) => k.t > 13.5 && k.t < 18.2);
    expect(
      between.some((k) => Math.abs(k.pose.scale - BASE_POSE.scale) < 1e-6),
      "camera swept between clusters instead of pulling back to base",
    ).toBe(true);
  });

  it("scales move durations so they land right on screen at speed", () => {
    const fast = buildCameraTrack(smokeLog, 1.25, {});
    const real = buildCameraTrack(smokeLog, 1, {});
    const firstMove = (keys: typeof fast) => {
      const i = keys.findIndex((k, n) => n > 0 && k.pose.scale > 1.05);
      return keys[i].t - keys[i - 1].t;
    };
    const ratio = firstMove(fast) / firstMove(real);
    expect(Math.abs(ratio - 1.25), `ratio ${ratio}`).toBeLessThan(0.02);
    const onScreenMs = (firstMove(fast) / 1.25) * 1000;
    expect(onScreenMs).toBeGreaterThanOrEqual(550);
    expect(onScreenMs).toBeLessThanOrEqual(1250);
  });

  it("holds still after the last click of a cluster", () => {
    const keys = buildCameraTrack(smokeLog, 1, {});
    const holdStart = 4338 / 1000 + 0.9;
    const holdEnd = 4338 / 1000 + 1.25;
    const a = sampleTrack(keys, holdStart);
    const b = sampleTrack(keys, holdEnd);
    expect(Math.abs(a.scale - b.scale), "hold drifted").toBeLessThan(0.01);
    expect(Math.abs(a.cx - b.cx)).toBeLessThan(0.01);
  });

  it("starts the first punch near cursor departure, not after the click", () => {
    const keys = buildCameraTrack(smokeLog, 1, {});
    const onset = keys.find((k) => k.t > 0.05 && Math.abs(k.pose.scale - 1) < 0.02)?.t;
    expect(onset, "click is 2.603s").toBeLessThan(2.0);
    expect(onset, "camera moved too early").toBeGreaterThan(1.4);
  });

  it("returns a constant pose when there are no zoomable clicks", () => {
    const keys = buildCameraTrack(
      {
        name: "empty",
        viewport: VP,
        durationMs: 4000,
        clicks: [click(500, 10, 10, { zoom: false })],
      },
      1,
      {},
    );
    const a = sampleTrack(keys, 0.2);
    const b = sampleTrack(keys, 3.5);
    expect(a.scale).toBe(1);
    expect(b.scale).toBe(1);
  });
});

describe("zoomAt", () => {
  it("has no origin discontinuity larger than 3% on a single frame during a hold", () => {
    const fps = 30;
    const start = Math.round(4.4 * fps);
    const end = Math.round(5.4 * fps);
    for (let f = start; f < end; f++) {
      const a = zoomAt(f, fps, smokeLog);
      const b = zoomAt(f + 1, fps, smokeLog);
      expect(Math.abs(a.translateX - b.translateX), `translateX jump at frame ${f}`).toBeLessThan(
        0.03,
      );
      expect(Math.abs(a.translateY - b.translateY), `translateY jump at frame ${f}`).toBeLessThan(
        0.03,
      );
    }
  });
});

describe("typing beats", () => {
  const typingLog: ClickLog = {
    name: "typing",
    viewport: VP,
    durationMs: 12000,
    clicks: [
      click(2000, 1328, 149, {
        tDepartMs: 1200,
        typeEndMs: 6000,
        cluster: "write",
        ...box(1005, 105, 646, 72),
      }),
    ],
  };

  const clickOnlyLog: ClickLog = {
    ...typingLog,
    clicks: [{ ...typingLog.clicks[0], typeEndMs: undefined }],
  };

  const scaleAt = (log: ClickLog, tS: number) =>
    sampleTrack(buildCameraTrack(log, 1, {}), tS).scale;

  it("stays zoomed until the last keystroke", () => {
    for (const t of [3.5, 4.5, 5.5, 5.9]) {
      expect(
        scaleAt(typingLog, t) > BASE_POSE.scale + 0.05,
        `expected to still be zoomed at ${t}s, got ${scaleAt(typingLog, t)}`,
      ).toBe(true);
    }
  });

  it("does trail out once typing has finished", () => {
    expect(
      scaleAt(typingLog, 9) < BASE_POSE.scale + 0.05,
      "camera should be back at base well after the last keystroke",
    ).toBe(true);
  });

  it("leaves a plain click's hold alone", () => {
    expect(
      scaleAt(clickOnlyLog, 5.5) < BASE_POSE.scale + 0.05,
      "a click with no typing must not inherit the extended hold",
    ).toBe(true);
  });

  it("ends at base either way", () => {
    for (const log of [typingLog, clickOnlyLog]) {
      const keys = buildCameraTrack(log, 1, {});
      expect(posesNearlyEqual(keys[keys.length - 1].pose, BASE_POSE)).toBe(true);
    }
  });
});

describe("frameFor", () => {
  it("does not dead-center the subject", () => {
    const pose = frameFor(click(0, 1600, 800, box(1500, 760, 200, 80)), VP, 1.6);
    expect(pose.scale >= 1.22 && pose.scale <= 1.6).toBe(true);
    expect(Math.round(pose.cx * 100)).not.toBe(Math.round((1600 / 1920) * 100));
  });

  it("keeps the click in shot when the framed region cannot fit", () => {
    const pose = frameFor(click(0, 1866, 1058, box(1160, 0, 760, 1080)), VP, S_MAX);
    expect(isInFrame(pose, VP, 1866, 1058), "the clicked button must be on screen").toBe(true);
  });

  it("does NOT surrender the zoom just because the region is oversized", () => {
    const pose = frameFor(click(0, 1721, 655, box(1280, 0, 640, 1080)), VP, S_MAX);
    expect(pose, "should still be zoomed in").not.toEqual(BASE_POSE);
    expect(pose.scale, "expected a real zoom").toBeGreaterThan(1.1);
    expect(isInFrame(pose, VP, 1721, 655)).toBe(true);
  });

  it("keeps a click visible when it sits at the edge of a big framed region", () => {
    const pose = frameFor(click(0, 1500, 880, box(300, 250, 1200, 640)), VP, S_MAX);
    expect(
      isInFrame(pose, VP, 1500, 880),
      `click fell outside the frame: ${JSON.stringify(pose)}`,
    ).toBe(true);
  });

  it("still frames ordinary controls off-centre", () => {
    const pose = frameFor(click(0, 476, 223, box(273, 190, 406, 66)), VP, 1.54);
    expect(pose.scale, "expected a real zoom").toBeGreaterThan(1.2);
    expect(isInFrame(pose, VP, 476, 223)).toBe(true);
    expect(pose).not.toEqual(BASE_POSE);
  });

  it("frames the whole target, centred, in a narrow frame", () => {
    const wide = click(1000, 400, 200, box(300, 180, 800, 40));
    const pose = frameFor(wide, VP, S_MAX * 1.69, true);
    const visible = 1 / pose.scale;
    expect(visible * VP.width, "the target fits the visible width").toBeGreaterThanOrEqual(800);
    expect(pose.cx).toBeCloseTo((0.8 * 700 + 0.2 * 400) / VP.width, 9);
    const small = click(1000, 960, 500, box(940, 490, 40, 20));
    expect(frameFor(small, VP, S_MAX * 1.69, true).scale).toBeCloseTo(S_MAX * 1.69, 9);
  });

  it("never inverts its clamps at maximum zoom", () => {
    const pose = frameFor(click(0, 1919, 1079), VP, S_MAX);
    expect(Number.isFinite(pose.cx) && Number.isFinite(pose.cy)).toBe(true);
    expect(pose.cx).toBeGreaterThanOrEqual(1 / (2 * pose.scale) - 1e-9);
    expect(pose.cy).toBeLessThanOrEqual(1 - 1 / (2 * pose.scale) + 1e-9);
  });
});

describe("camera in narrower shapes", () => {
  const log: ClickLog = {
    name: "aim",
    viewport: VP,
    durationMs: 6000,
    clicks: [],
    cursorTrack: [
      { t: 0, x: 1600, y: 300 },
      { t: 500, x: 1700, y: 400 },
      { t: 4000, x: 200, y: 900 },
    ],
  };

  it("aims at where the cursor has been around that moment", () => {
    const aim = cursorAim(log, 0.3);
    expect(aim?.cx).toBeCloseTo(1650 / 1920, 9);
    expect(aim?.cy).toBeCloseTo(350 / 1080, 9);
    expect(cursorAim(log, 4)?.cx).toBeCloseTo(200 / 1920, 9);
    expect(cursorAim({ ...log, cursorTrack: [] }, 1)).toBe(null);
  });

  it("never rests fully zoomed out, and rests on the cursor", () => {
    const rest = narrowPose(BASE_POSE, { narrow: 1.69 }, { cx: 0.8, cy: 0.3 });
    expect(rest.scale).toBeGreaterThan(1.3);
    expect(rest.cx).toBe(0.8);
  });

  it("aims at the action itself when fully zoomed in", () => {
    const pose = narrowPose(
      { scale: S_MAX * 1.69, cx: 0.2, cy: 0.7 },
      { narrow: 1.69 },
      { cx: 0.8, cy: 0.3 },
    );
    expect(pose.cx).toBeCloseTo(0.2, 9);
    expect(pose.cy).toBeCloseTo(0.7, 9);
    expect(pose.scale).toBeCloseTo(S_MAX * 1.69, 9);
  });

  it("changes nothing in the recording's own shape", () => {
    const pose = { scale: 1.3, cx: 0.4, cy: 0.5 };
    expect(narrowPose(pose, { narrow: 1 }, { cx: 0.9, cy: 0.9 })).toEqual(pose);
  });

  it("aims at the action once zoomed to the node's own maximum", () => {
    const tuning = { narrow: 1.69, sMax: 1.3 };
    const full = { scale: 1.3 * 1.69, cx: 0.2, cy: 0.7 };
    const pose = narrowPose(full, tuning, { cx: 0.8, cy: 0.3 });
    expect(pose.cx).toBeCloseTo(0.2, 9);
    expect(pose.cy).toBeCloseTo(0.7, 9);
  });
});
