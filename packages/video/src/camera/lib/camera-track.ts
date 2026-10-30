import {
  BASE_POSE,
  interpolatePose,
  moveDuration,
  posesNearlyEqual,
  poseTravelPx,
  type CameraPose,
} from "#video/camera/lib/camera.ts";
import type { ClickEvent, ClickLog } from "#video/camera/lib/click-log.ts";
import { designScaleFor } from "#video/camera/lib/design-width.ts";
import { distance, lerpPoint, pointRect, rectCenter } from "#video/common/geometry.ts";
import { clamp, clamp01, lerp, partitionPoint, sampleKeyed } from "#video/common/math.ts";
import type { CameraTuning as SpecTuning, Rect, Viewport } from "#video/generated/spec.ts";

export const S_MAX = 1.74;

const ZOOM = { min: 1.18, firstShotMax: 1.54 } as const;

const SUBJECT = {
  padPx: 80,
  menuRoomPx: 110,
  tallTargetPx: 90,
  tallMenuRoomPx: 24,
  clickPull: 0.2,
} as const;

const VERTICAL_OFF_CENTER_SHARE = 0.65;

const FRAMING = {
  targetShare: 0.55,
  narrowTargetShare: 0.86,
  offCenter: { x: 0.26, y: 0.26 * VERTICAL_OFF_CENTER_SHARE },
  clickEdgeMargin: 0.04,
  clickEdgeMarginMaxShare: 0.4,
} as const;

const GROUPING = { gapS: 2.2, distFrac: 0.28 } as const;

const STICKY = { gapS: 2.8, distFrac: 0.22 } as const;

const TIMING = {
  leadFallbackS: 0.75,
  holdMinS: 1.3,
  typeTailS: 0.4,
  cameraLeadS: 0.035,
  baseBeatS: 0.6,
} as const;

const TRACK = { maxPanPx: 500, minS: 0.5, keyEpsilonS: 1e-4, keyMergeS: 0.2 } as const;

const NARROW_AIM = { windowS: 1.2, restExp: 0.8, restAimY: 0.5 } as const;

type CameraKey = { t: number; pose: CameraPose };

export type CameraTrack = [CameraKey, ...CameraKey[]];

export type CameraTuning = SpecTuning & { narrow?: number };

type Tuned = {
  minScale: number;
  firstShotMax: number;
  padPx: number;
  targetShare: number;
  offCentre: number;
  joinGapS: number;
  stickyGapS: number;
  maxPanPx: number;
};

function tuned(t: CameraTuning): Tuned {
  return {
    minScale: t.minScale ?? ZOOM.min,
    firstShotMax: t.firstMaxScale ?? ZOOM.firstShotMax,
    padPx: t.roomPx ?? SUBJECT.padPx,
    targetShare: t.targetShare ?? FRAMING.targetShare,
    offCentre: t.offCentre ?? FRAMING.offCenter.x,
    joinGapS: t.joinWithinS ?? GROUPING.gapS,
    stickyGapS: t.stayZoomedWithinS ?? STICKY.gapS,
    maxPanPx: t.longestPanPx ?? TRACK.maxPanPx,
  };
}

export const NO_TUNING: CameraTuning = {};

type Aim = Pick<CameraPose, "cx" | "cy">;

function narrowFactor(tuning: CameraTuning): number {
  return tuning.narrow ?? 1;
}

export function isNarrow(tuning: CameraTuning): boolean {
  return narrowFactor(tuning) > 1;
}

export function cursorAim(log: ClickLog, tS: number): Aim | null {
  const track = log.cursorTrack;
  if (!track?.length) return null;
  const from = (tS - NARROW_AIM.windowS) * 1000;
  const to = (tS + NARROW_AIM.windowS) * 1000;
  const lo = partitionPoint(track, (s) => s.t < from);
  let x = 0;
  let y = 0;
  let n = 0;
  for (let i = lo; i < track.length && track[i].t <= to; i++) {
    x += track[i].x;
    y += track[i].y;
    n++;
  }
  if (!n) {
    const near = track[Math.min(lo, track.length - 1)];
    x = near.x;
    y = near.y;
    n = 1;
  }
  return { cx: x / n / log.viewport.width, cy: y / n / log.viewport.height };
}

function zoomCeiling(tuning: CameraTuning): number {
  return (tuning.sMax ?? S_MAX) * narrowFactor(tuning);
}

export function narrowPose(pose: CameraPose, tuning: CameraTuning, aim: Aim | null): CameraPose {
  if (!isNarrow(tuning)) return pose;
  const rest = narrowFactor(tuning) ** NARROW_AIM.restExp;
  const zoomed = clamp01((pose.scale - rest) / (zoomCeiling(tuning) - rest));
  const restAim = aim ?? { cx: BASE_POSE.cx, cy: BASE_POSE.cy };
  return {
    scale: Math.max(pose.scale, rest),
    cx: lerp(restAim.cx, pose.cx, zoomed),
    cy: lerp(lerp(BASE_POSE.cy, restAim.cy, NARROW_AIM.restAimY), pose.cy, zoomed),
  };
}

function zoomable(clicks: ClickEvent[]): ClickEvent[] {
  return clicks.filter((c) => c.zoom !== false);
}

function apartFrac(a: ClickEvent, b: ClickEvent, vp: Viewport): number {
  return distance(a, b) / Math.hypot(vp.width, vp.height);
}

function gapS(a: ClickEvent, b: ClickEvent): number {
  return (b.tMs - a.tMs) / 1000;
}

export const tunedFrom = tuned;

export function clusterize(clicks: ClickEvent[], vp: Viewport, t: Tuned): ClickEvent[][] {
  const list = zoomable(clicks);
  function startsGroup(prev: ClickEvent, c: ClickEvent): boolean {
    return (
      apartFrac(prev, c, vp) > GROUPING.distFrac ||
      prev.cluster !== c.cluster ||
      (c.cluster === undefined && gapS(prev, c) > t.joinGapS)
    );
  }
  const groups: ClickEvent[][] = [];
  list.forEach((c, i) => {
    if (i === 0 || startsGroup(list[i - 1], c)) groups.push([c]);
    else groups[groups.length - 1].push(c);
  });
  return groups;
}

export function stickify(groups: ClickEvent[][], vp: Viewport, t: Tuned): ClickEvent[][] {
  if (groups.length <= 1) return groups;
  const explicit = groups.some((g) => g.some((c) => c.cluster !== undefined));
  const out: ClickEvent[][] = [groups[0]];
  for (const next of groups.slice(1)) {
    const prev = out[out.length - 1];
    const last = prev[prev.length - 1];
    const first = next[0];
    const otherCluster =
      explicit &&
      last.cluster !== undefined &&
      first.cluster !== undefined &&
      last.cluster !== first.cluster;
    const close =
      gapS(last, first) <= t.stickyGapS && apartFrac(last, first, vp) <= STICKY.distFrac;
    if (!otherCluster && close) out[out.length - 1] = prev.concat(next);
    else out.push(next);
  }
  return out;
}

function subjectOf(c: ClickEvent, width: number, padPx: number): Rect {
  const k = designScaleFor(width);
  const rect = c.rect ?? pointRect(c, k);
  const menuRoom =
    (rect.h >= SUBJECT.tallTargetPx * k ? SUBJECT.tallMenuRoomPx : SUBJECT.menuRoomPx) * k;
  const center = rectCenter(rect);
  return {
    ...lerpPoint({ x: center.x, y: center.y + menuRoom / 2 }, c, SUBJECT.clickPull),
    w: rect.w + 2 * padPx * k,
    h: rect.h + menuRoom + 2 * padPx * k,
  };
}

function scaleFor(
  c: ClickEvent,
  subject: Rect,
  vp: Viewport,
  sMax: number,
  narrow: boolean,
  t: Tuned,
): number {
  const least = Math.min(t.minScale, sMax);
  if (c.zoomScale != null) return clamp(c.zoomScale, least, sMax);
  const share = narrow ? FRAMING.narrowTargetShare : t.targetShare;
  const byWidth = (share * vp.width) / subject.w;
  if (narrow) return clamp(byWidth, least, sMax);
  const byHeight = (t.targetShare * vp.height) / subject.h;
  return clamp(Math.min(byWidth, byHeight), least, sMax);
}

function inView(v: number, half: number): number {
  return clamp(v, half, 1 - half);
}

function keepingClick(aimed: number, click: number, half: number): number {
  const margin = Math.min(FRAMING.clickEdgeMargin, half * FRAMING.clickEdgeMarginMaxShare);
  return inView(clamp(aimed, click - half + margin, click + half - margin), half);
}

function offCenter(focus: number, click: number, offset: number, scale: number): number {
  const half = 1 / (2 * scale);
  const side = focus >= 0.5 ? 1 : -1;
  return keepingClick(inView(focus - (offset * side) / scale, half), click, half);
}

export function frameFor(
  c: ClickEvent,
  vp: Viewport,
  sMax: number,
  narrow: boolean,
  t: Tuned,
): CameraPose {
  const subject = subjectOf(c, vp.width, t.padPx);
  const scale = scaleFor(c, subject, vp, sMax, narrow, t);
  const focusX = clamp01(subject.x / vp.width);
  const focusY = clamp01(subject.y / vp.height);
  if (narrow) {
    const half = 1 / (2 * scale);
    return {
      scale,
      cx: keepingClick(inView(focusX, half), clamp01(c.x / vp.width), half),
      cy: keepingClick(inView(focusY, half), clamp01(c.y / vp.height), half),
    };
  }
  return {
    scale,
    cx: offCenter(focusX, clamp01(c.x / vp.width), t.offCentre, scale),
    cy: offCenter(focusY, clamp01(c.y / vp.height), t.offCentre * VERTICAL_OFF_CENTER_SHARE, scale),
  };
}

function departS(c: ClickEvent): number {
  return (c.tDepartMs ?? c.tMs - TIMING.leadFallbackS * 1000) / 1000;
}

function normalizeKeys(keys: CameraTrack): CameraTrack {
  const [first, ...rest] = [...keys].sort((a, b) => a.t - b.t);
  const out: CameraTrack = [first];
  for (const k of rest) {
    const prev = out[out.length - 1];
    if (k.t + TRACK.keyEpsilonS < prev.t) continue;
    const replaces =
      k.t - prev.t < TRACK.keyEpsilonS ||
      (k.t - prev.t < TRACK.keyMergeS && posesNearlyEqual(prev.pose, k.pose));
    if (replaces) out[out.length - 1] = k;
    else out.push(k);
  }
  return out;
}

type Shot = { group: ClickEvent[]; poses: CameraPose[]; tIn: number };

type TrackPlan = {
  vp: Viewport;
  speed: number;
  maxPanPx: number;
  holdMin: number;
  typeTail: number;
  baseBeat: number;
  endS: number;
};

function lastT(keys: CameraKey[]): number {
  return keys[keys.length - 1].t;
}

function holdUntil(c: ClickEvent, plan: TrackPlan): number {
  return c.typeEndMs != null
    ? Math.max(c.tMs / 1000 + plan.holdMin, c.typeEndMs / 1000 + plan.typeTail)
    : c.tMs / 1000 + plan.holdMin;
}

function move(t: number, from: CameraPose, to: CameraPose, duration: number): CameraKey[] {
  return [
    { t, pose: from },
    { t: t + duration, pose: to },
  ];
}

function zoomIn(from: CameraPose, shot: Shot, afterT: number, plan: TrackPlan): CameraKey[] {
  const to = shot.poses[0];
  return move(
    Math.max(shot.tIn, afterT),
    from,
    to,
    moveDuration(from, to, "in", plan.vp, plan.speed),
  );
}

function routeViaBase(
  from: CameraPose,
  to: CameraPose,
  { tPan, holdEnd }: { tPan: number; holdEnd: number },
  { vp, speed }: TrackPlan,
): CameraKey[] {
  const durBack = moveDuration(from, BASE_POSE, "out", vp, speed);
  const backStart = clamp(tPan - durBack, holdEnd, tPan);
  return [
    ...move(backStart, from, BASE_POSE, durBack),
    ...move(
      Math.max(backStart + durBack, tPan),
      BASE_POSE,
      to,
      moveDuration(BASE_POSE, to, "in", vp, speed),
    ),
  ];
}

function panWithin(shot: Shot, startT: number, plan: TrackPlan): CameraKey[] {
  const { vp, speed } = plan;
  const keys: CameraKey[] = [];
  let curT = startT;
  for (let j = 1; j < shot.group.length; j++) {
    const from = shot.poses[j - 1];
    const to = shot.poses[j];
    const tPan = Math.max(curT, departS(shot.group[j]) - TIMING.cameraLeadS);
    const holdEnd = Math.max(curT, holdUntil(shot.group[j - 1], plan));
    keys.push(
      ...(poseTravelPx(from, to, vp) > plan.maxPanPx
        ? routeViaBase(from, to, { tPan, holdEnd }, plan)
        : move(tPan, from, to, moveDuration(from, to, "pan", vp, speed))),
    );
    curT = lastT(keys);
  }
  return keys;
}

function trailOut(
  shot: Shot,
  next: Shot | undefined,
  curT: number,
  plan: TrackPlan,
): { keys: CameraKey[]; pose: CameraPose } {
  const pose = shot.poses[shot.poses.length - 1];
  const holdEnd = Math.max(curT, holdUntil(shot.group[shot.group.length - 1], plan));
  const durOut = moveDuration(pose, BASE_POSE, "out", plan.vp, plan.speed);

  if (!next) {
    const clipEnd = Math.max(curT, plan.endS);
    const trailStart = Math.max(curT, Math.min(holdEnd, clipEnd - durOut - plan.baseBeat));
    const keys =
      trailStart + durOut <= clipEnd
        ? [...move(trailStart, pose, BASE_POSE, durOut), { t: clipEnd, pose: BASE_POSE }]
        : [{ t: clipEnd, pose }];
    return { keys, pose: BASE_POSE };
  }

  const farApart = poseTravelPx(pose, next.poses[0], plan.vp) > plan.maxPanPx;
  if (next.tIn >= holdEnd + durOut || farApart) {
    const trailStart = Math.max(curT, Math.min(holdEnd, next.tIn - durOut));
    return { keys: move(trailStart, pose, BASE_POSE, durOut), pose: BASE_POSE };
  }
  return { keys: [{ t: Math.max(curT, next.tIn), pose }], pose };
}

function planShots(log: ClickLog, tuning: CameraTuning): Shot[] {
  const narrow = isNarrow(tuning);
  const sMax = zoomCeiling(tuning);
  const t = tuned(tuning);
  const firstShotMax = Math.min(t.firstShotMax, sMax);
  const vp = log.viewport;
  function shotMax(i: number, c: ClickEvent): number {
    return i === 0 && c.zoomScale == null && !narrow ? firstShotMax : sMax;
  }
  return stickify(clusterize(log.clicks, vp, t), vp, t).map((group, i) => ({
    group,
    poses: group.map((c) => frameFor(c, vp, shotMax(i, c), narrow, t)),
    tIn: Math.max(0, departS(group[0]) - TIMING.cameraLeadS),
  }));
}

function trackPlan(log: ClickLog, speed: number, tuning: CameraTuning): TrackPlan {
  return {
    vp: log.viewport,
    speed: speed * (tuning.pace ?? 1),
    maxPanPx: tuned(tuning).maxPanPx,
    holdMin: (tuning.holdMinS ?? TIMING.holdMinS) * speed,
    typeTail: (tuning.holdAfterTypingS ?? TIMING.typeTailS) * speed,
    baseBeat: (tuning.restAtEndS ?? TIMING.baseBeatS) * speed,
    endS: log.durationMs / 1000,
  };
}

export function buildCameraTrack(log: ClickLog, speed: number, tuning: CameraTuning): CameraTrack {
  const shots = planShots(log, tuning);
  const start: CameraKey = { t: 0, pose: BASE_POSE };
  const plan = trackPlan(log, speed, tuning);
  if (shots.length === 0) return [start, { t: Math.max(plan.endS, TRACK.minS), pose: BASE_POSE }];
  const keys: CameraTrack = [start];
  let pose = BASE_POSE;
  shots.forEach((shot, i) => {
    const moves = zoomIn(pose, shot, lastT(keys), plan);
    moves.push(...panWithin(shot, lastT(moves), plan));
    const trail = trailOut(shot, shots[i + 1], lastT(moves), plan);
    keys.push(...moves, ...trail.keys);
    pose = trail.pose;
  });

  return normalizeKeys(keys);
}

export function sampleTrack(keys: CameraTrack, t: number, tuning: CameraTuning): CameraPose {
  const easing = tuning.easing ?? "smooth";
  return sampleKeyed(
    keys,
    t,
    (k) => k.t,
    (k) => k.pose,
    (a, b, p) => interpolatePose(a.pose, b.pose, p, easing),
  );
}
