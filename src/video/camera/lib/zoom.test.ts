import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { BASE_POSE, posesNearlyEqual } from "./camera";
import type { ClickEvent, ClickLog } from "./click-log";
import { sees } from "./test-fixtures";
import {
  buildCameraTrack,
  cursorAim,
  narrowPose,
  clusterize,
  frameFor,
  S_MAX,
  sampleTrack,
  stickify,
  zoomAt,
} from "./zoom";

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

describe("stickify", () => {
  it("never merges different explicit cluster ids", () => {
    const groups = clusterize(clusteredLog.clicks, VP);
    const stuck = stickify(groups, VP);
    assert.equal(stuck.length, 4);
    assert.deepEqual(
      stuck.map((g) => g[0].cluster),
      ["language", "domain", "category", "payoff"],
    );
  });
});

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
    assert.equal(groups.length, 2, "far-apart targets stayed in one cluster");
    assert.equal(groups[0].length, 1);
    assert.equal(groups[1].length, 2);
  });

  it("keeps a cluster together when its targets are close", () => {
    const groups = clusterize(
      [click(1000, 476, 159, { cluster: "c0" }), click(3000, 557, 236, { cluster: "c0" })],
      VP,
    );
    assert.equal(groups.length, 1);
  });
});

describe("buildCameraTrack", () => {
  it("starts at base and returns to base before the cut", () => {
    const keys = buildCameraTrack(smokeLog);
    assert.equal(keys[0].pose.scale, BASE_POSE.scale);
    const last = keys[keys.length - 1];
    assert.ok(Math.abs(last.t - smokeLog.durationMs / 1000) < 0.05);
    assert.equal(last.pose.scale, BASE_POSE.scale);
    assert.equal(last.pose.cx, BASE_POSE.cx);
  });

  it("holds still at base for a beat before the cut", () => {
    const keys = buildCameraTrack(smokeLog);
    const end = smokeLog.durationMs / 1000;
    const a = sampleTrack(keys, end - 0.3);
    const b = sampleTrack(keys, end);
    assert.equal(a.scale, BASE_POSE.scale);
    assert.equal(b.scale, BASE_POSE.scale);
  });

  it("routes through base instead of panning across the screen", () => {
    const keys = buildCameraTrack({
      name: "wide",
      viewport: VP,
      durationMs: 16000,
      clicks: [
        click(3000, 300, 200, { tDepartMs: 2000, cluster: "one", ...box(240, 170, 200, 60) }),
        click(9000, 1700, 980, { tDepartMs: 8000, cluster: "one", ...box(1600, 950, 200, 60) }),
      ],
    });
    const between = keys.filter((k) => k.t > 3 && k.t < 9);
    assert.ok(
      between.some((k) => Math.abs(k.pose.scale - BASE_POSE.scale) < 1e-6),
      "camera panned across the screen instead of pulling back to base",
    );
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
    );
    const between = keys.filter((k) => k.t > 13.5 && k.t < 18.2);
    assert.ok(
      between.some((k) => Math.abs(k.pose.scale - BASE_POSE.scale) < 1e-6),
      "camera swept between clusters instead of pulling back to base",
    );
  });

  it("scales move durations so they land right on screen at speed", () => {
    const fast = buildCameraTrack(smokeLog, 1.25);
    const real = buildCameraTrack(smokeLog, 1);
    const firstMove = (keys: typeof fast) => {
      const i = keys.findIndex((k, n) => n > 0 && k.pose.scale > 1.05);
      return keys[i].t - keys[i - 1].t;
    };
    const ratio = firstMove(fast) / firstMove(real);
    assert.ok(
      Math.abs(ratio - 1.25) < 0.02,
      `expected 1.25x log-time duration at 1.25x speed, got ${ratio}`,
    );
    const onScreenMs = (firstMove(fast) / 1.25) * 1000;
    assert.ok(
      onScreenMs >= 550 && onScreenMs <= 1250,
      `on-screen zoom-in ${onScreenMs.toFixed(0)}ms outside the reference window`,
    );
  });

  it("holds still after the last click of a cluster", () => {
    const keys = buildCameraTrack(smokeLog);
    const holdStart = 4338 / 1000 + 0.9;
    const holdEnd = 4338 / 1000 + 1.25;
    const a = sampleTrack(keys, holdStart);
    const b = sampleTrack(keys, holdEnd);
    assert.ok(Math.abs(a.scale - b.scale) < 0.01, `hold drifted ${a.scale} → ${b.scale}`);
    assert.ok(Math.abs(a.cx - b.cx) < 0.01);
  });

  it("starts the first punch near cursor departure, not after the click", () => {
    const keys = buildCameraTrack(smokeLog);
    const onset = keys.find((k) => k.t > 0.05 && Math.abs(k.pose.scale - 1) < 0.02);
    assert.ok(onset);
    assert.ok(onset.t < 2.0, `camera onset at ${onset.t}s, click is 2.603s`);
    assert.ok(onset.t > 1.4, `camera moved too early: ${onset.t}`);
  });

  it("returns a constant pose when there are no zoomable clicks", () => {
    const keys = buildCameraTrack({
      name: "empty",
      viewport: VP,
      durationMs: 4000,
      clicks: [click(500, 10, 10, { zoom: false })],
    });
    const a = sampleTrack(keys, 0.2);
    const b = sampleTrack(keys, 3.5);
    assert.equal(a.scale, 1);
    assert.equal(b.scale, 1);
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
      assert.ok(Math.abs(a.translateX - b.translateX) < 0.03, `translateX jump at frame ${f}`);
      assert.ok(Math.abs(a.translateY - b.translateY) < 0.03, `translateY jump at frame ${f}`);
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

  const scaleAt = (log: ClickLog, tS: number) => sampleTrack(buildCameraTrack(log), tS).scale;

  it("stays zoomed until the last keystroke", () => {
    for (const t of [3.5, 4.5, 5.5, 5.9]) {
      assert.ok(
        scaleAt(typingLog, t) > BASE_POSE.scale + 0.05,
        `expected to still be zoomed at ${t}s, got ${scaleAt(typingLog, t)}`,
      );
    }
  });

  it("does trail out once typing has finished", () => {
    assert.ok(
      scaleAt(typingLog, 9) < BASE_POSE.scale + 0.05,
      "camera should be back at base well after the last keystroke",
    );
  });

  it("leaves a plain click's hold alone", () => {
    assert.ok(
      scaleAt(clickOnlyLog, 5.5) < BASE_POSE.scale + 0.05,
      "a click with no typing must not inherit the extended hold",
    );
  });

  it("ends at base either way", () => {
    for (const log of [typingLog, clickOnlyLog]) {
      const keys = buildCameraTrack(log);
      assert.ok(posesNearlyEqual(keys[keys.length - 1].pose, BASE_POSE));
    }
  });
});

describe("frameFor", () => {
  it("does not dead-center the subject", () => {
    const pose = frameFor(click(0, 1600, 800, box(1500, 760, 200, 80)), VP, 1.6);
    assert.ok(pose.scale >= 1.22 && pose.scale <= 1.6);
    assert.notEqual(Math.round(pose.cx * 100), Math.round((1600 / 1920) * 100));
  });

  it("keeps the click in shot when the framed region cannot fit", () => {
    const pose = frameFor(click(0, 1866, 1058, box(1160, 0, 760, 1080)), VP, S_MAX);
    assert.ok(sees(pose, VP, 1866, 1058), "the clicked button must be on screen");
  });

  it("does NOT surrender the zoom just because the region is oversized", () => {
    const pose = frameFor(click(0, 1721, 655, box(1280, 0, 640, 1080)), VP, S_MAX);
    assert.notDeepEqual(pose, BASE_POSE, "should still be zoomed in");
    assert.ok(pose.scale > 1.1, `expected a real zoom, got ${pose.scale}`);
    assert.ok(sees(pose, VP, 1721, 655));
  });

  it("keeps a click visible when it sits at the edge of a big framed region", () => {
    const pose = frameFor(click(0, 1500, 880, box(300, 250, 1200, 640)), VP, S_MAX);
    assert.ok(sees(pose, VP, 1500, 880), `click fell outside the frame: ${JSON.stringify(pose)}`);
  });

  it("still frames ordinary controls off-centre", () => {
    const pose = frameFor(click(0, 476, 223, box(273, 190, 406, 66)), VP, 1.54);
    assert.ok(pose.scale > 1.2, `expected a real zoom, got ${pose.scale}`);
    assert.ok(sees(pose, VP, 476, 223));
    assert.notDeepEqual(pose, BASE_POSE);
  });

  it("frames the whole target, centred, in a narrow frame", () => {
    const wide = click(1000, 400, 200, box(300, 180, 800, 40));
    const pose = frameFor(wide, VP, S_MAX * 1.69, true);
    const visible = 1 / pose.scale;
    assert.ok(visible * VP.width >= 800, "the target fits the visible width");
    assert.ok(Math.abs(pose.cx - (0.8 * 700 + 0.2 * 400) / VP.width) < 1e-9);
    const small = click(1000, 960, 500, box(940, 490, 40, 20));
    assert.ok(Math.abs(frameFor(small, VP, S_MAX * 1.69, true).scale - S_MAX * 1.69) < 1e-9);
  });

  it("never inverts its clamps at maximum zoom", () => {
    const pose = frameFor(click(0, 1919, 1079), VP, S_MAX);
    assert.ok(Number.isFinite(pose.cx) && Number.isFinite(pose.cy));
    assert.ok(pose.cx >= 1 / (2 * pose.scale) - 1e-9);
    assert.ok(pose.cy <= 1 - 1 / (2 * pose.scale) + 1e-9);
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
    assert.ok(aim);
    assert.ok(Math.abs(aim.cx - 1650 / 1920) < 1e-9);
    assert.ok(Math.abs(aim.cy - 350 / 1080) < 1e-9);
    const later = cursorAim(log, 4);
    assert.ok(later);
    assert.ok(Math.abs(later.cx - 200 / 1920) < 1e-9);
    assert.equal(cursorAim({ ...log, cursorTrack: [] }, 1), null);
  });

  it("never rests fully zoomed out, and rests on the cursor", () => {
    const rest = narrowPose(BASE_POSE, 1.69, { cx: 0.8, cy: 0.3 });
    assert.ok(rest.scale > 1.3);
    assert.equal(rest.cx, 0.8);
  });

  it("aims at the action itself when fully zoomed in", () => {
    const pose = narrowPose({ scale: S_MAX * 1.69, cx: 0.2, cy: 0.7 }, 1.69, { cx: 0.8, cy: 0.3 });
    assert.ok(Math.abs(pose.cx - 0.2) < 1e-9);
    assert.ok(Math.abs(pose.cy - 0.7) < 1e-9);
    assert.ok(Math.abs(pose.scale - S_MAX * 1.69) < 1e-9);
  });

  it("changes nothing in the recording's own shape", () => {
    const pose = { scale: 1.3, cx: 0.4, cy: 0.5 };
    assert.deepEqual(narrowPose(pose, 1, { cx: 0.9, cy: 0.9 }), pose);
  });
});
