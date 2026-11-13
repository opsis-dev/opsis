import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { random } from "@opsis/runtime/random.ts";
import { expect, it } from "vitest";
import { toClickLog } from "#video/camera/action-log.ts";
import { buildCameraTrack, type CameraTuning } from "#video/camera/lib/camera-track.ts";
import { poseAt, zoomAt } from "#video/camera/lib/zoom.ts";
import type { Action, ActionKind, ActionLog, CameraEasing } from "#video/generated/spec.ts";

const FIXTURE = path.resolve(
  import.meta.dirname,
  "../../../../../fixtures/scene/camera-track.json",
);

const CASES = 48;
const FPS = 30;
const KINDS: ActionKind[] = [
  "click",
  "type",
  "hover",
  "focus",
  "drag",
  "key",
  "scroll",
  "navigate",
];
const EASINGS: CameraEasing[] = ["smooth", "gentle", "snappy", "linear"];

function draw(seed: string, low: number, high: number): number {
  return low + random(seed) * (high - low);
}

function pick<T>(seed: string, list: T[]): T {
  return list[Math.min(list.length - 1, Math.floor(random(seed) * list.length))];
}

function sometimes<T>(seed: string, share: number, value: () => T): T | undefined {
  return random(seed) < share ? value() : undefined;
}

function action(c: number, i: number, t: number, width: number, height: number): Action {
  const key = `${c}-${i}`;
  const kind = pick(`${key}-kind`, KINDS);
  const x = draw(`${key}-x`, 0, width);
  const y = draw(`${key}-y`, 0, height);
  const w = draw(`${key}-w`, 20, width * 0.6);
  const h = draw(`${key}-h`, 16, height * 0.4);
  return {
    id: `a${i}`,
    kind,
    t,
    tEnd: sometimes(`${key}-end`, 0.6, () => t + draw(`${key}-endt`, 100, 2400)),
    tDepart: sometimes(`${key}-dep`, 0.7, () => Math.max(0, t - draw(`${key}-dept`, 80, 900))),
    tDown: sometimes(`${key}-down`, 0.4, () => Math.max(0, t - draw(`${key}-downt`, 10, 300))),
    point: kind === "key" ? undefined : { x, y },
    rect: sometimes(`${key}-rect`, 0.75, () => ({ x: x - w / 2, y: y - h / 2, w, h })),
    drop: sometimes(`${key}-drop`, 0.5, () => ({
      x: draw(`${key}-dx`, 0, width),
      y: draw(`${key}-dy`, 0, height),
      w: draw(`${key}-dw`, 20, 300),
      h: draw(`${key}-dh`, 20, 200),
    })),
    cluster: sometimes(`${key}-cl`, 0.3, () => pick(`${key}-clv`, ["", "a", "b"])),
    label: `Label ${i}`,
    zoom: sometimes(`${key}-zoom`, 0.15, () => random(`${key}-zoomv`) < 0.5),
    zoomScale: sometimes(`${key}-zs`, 0.1, () => draw(`${key}-zsv`, 0.9, 2.4)),
    text:
      kind === "key" || kind === "type" ? pick(`${key}-text`, ["", "Enter", "hello"]) : undefined,
  };
}

function actionLog(c: number): ActionLog {
  const width = pick(`${c}-vw`, [1280, 1440, 1920, 2560, 900]);
  const height = pick(`${c}-vh`, [720, 900, 1080, 1440, 1600]);
  const count = Math.floor(draw(`${c}-n`, 0, 14));
  let t = 0;
  const actions: Action[] = [];
  for (let i = 0; i < count; i++) {
    t += draw(`${c}-${i}-gap`, 50, 3600);
    actions.push(action(c, i, t, width, height));
  }
  const durationMs = t + draw(`${c}-tail`, 0, 4000);
  const cursorTrack =
    random(`${c}-track`) < 0.6
      ? Array.from({ length: 60 }, (_, i) => ({
          t: (durationMs / 60) * i,
          x: draw(`${c}-cx${i}`, 0, width),
          y: draw(`${c}-cy${i}`, 0, height),
        }))
      : undefined;
  return { viewport: { width, height }, dpr: 2, t0: 0, durationMs, actions, cursorTrack };
}

function tuning(c: number): CameraTuning {
  return {
    sMax: sometimes(`${c}-smax`, 0.3, () => draw(`${c}-smaxv`, 1.2, 2.6)),
    holdMinS: sometimes(`${c}-hold`, 0.3, () => draw(`${c}-holdv`, 0.2, 3)),
    minScale: sometimes(`${c}-min`, 0.2, () => draw(`${c}-minv`, 0.8, 1.6)),
    firstMaxScale: sometimes(`${c}-first`, 0.2, () => draw(`${c}-firstv`, 1, 2)),
    roomPx: sometimes(`${c}-room`, 0.2, () => draw(`${c}-roomv`, 0, 200)),
    targetShare: sometimes(`${c}-share`, 0.2, () => draw(`${c}-sharev`, 0.2, 0.9)),
    offCentre: sometimes(`${c}-off`, 0.2, () => draw(`${c}-offv`, 0, 0.4)),
    joinWithinS: sometimes(`${c}-join`, 0.2, () => draw(`${c}-joinv`, 0.5, 5)),
    stayZoomedWithinS: sometimes(`${c}-stay`, 0.2, () => draw(`${c}-stayv`, 0.5, 5)),
    holdAfterTypingS: sometimes(`${c}-tail`, 0.2, () => draw(`${c}-tailv`, 0, 2)),
    restAtEndS: sometimes(`${c}-rest`, 0.2, () => draw(`${c}-restv`, 0, 2)),
    longestPanPx: sometimes(`${c}-pan`, 0.2, () => draw(`${c}-panv`, 100, 1200)),
    pace: sometimes(`${c}-pace`, 0.2, () => draw(`${c}-pacev`, 0.5, 2)),
    easing: sometimes(`${c}-ease`, 0.6, () => pick(`${c}-easev`, EASINGS)),
    narrow: sometimes(`${c}-narrow`, 0.3, () => draw(`${c}-narrowv`, 1, 2.4)),
  };
}

function recorded() {
  return {
    cases: Array.from({ length: CASES }, (_, c) => {
      const log = actionLog(c);
      const tuned = tuning(c);
      const speed = pick(`${c}-speed`, [1, 1.25, 0.75, 2]);
      const zoomActions = random(`${c}-za`) < 0.85;
      const clicks = toClickLog(log, `take ${c}`, [], { zoomActions });
      const frames = Math.ceil((log.durationMs / 1000 / speed) * FPS) + 10;
      const samples = Array.from({ length: Math.min(frames, 150) }, (_, i) => {
        const frame = (i * frames) / Math.min(frames, 150);
        return {
          frame,
          pose: poseAt(frame, FPS, clicks, { speed, tuning: tuned }),
          zoom: zoomAt(frame, FPS, clicks, { speed, tuning: tuned, chromeFrac: 0.04, fit: 0.9 }),
        };
      });
      return {
        log,
        tuning: tuned,
        speed,
        zoomActions,
        clicks,
        keys: buildCameraTrack(clicks, speed, tuned),
        samples,
      };
    }),
  };
}

it("records the film's camera track, for the native renderer's port to match", () => {
  const made = JSON.parse(JSON.stringify(recorded()));
  if (process.env.OPSIS_WRITE_GOLDEN === "1") {
    mkdirSync(path.dirname(FIXTURE), { recursive: true });
    writeFileSync(FIXTURE, `${JSON.stringify(made)}\n`);
  }
  expect(existsSync(FIXTURE), `${FIXTURE} is missing: record it with OPSIS_WRITE_GOLDEN=1`).toBe(
    true,
  );
  expect(JSON.parse(readFileSync(FIXTURE, "utf8"))).toEqual(made);
});
