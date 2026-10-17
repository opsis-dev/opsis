import { defaultTargetRect, type ClickEvent, type ClickLog } from "./click-log";
import {
  BASE_POSE,
  interpolatePose,
  moveDuration,
  poseToCss,
  posesNearlyEqual,
  poseTravelPx,
  type CameraPose,
  type ZoomState,
} from "./camera";
import { designScaleFor } from "./design-width";
import { clamp, clamp01, lerp, partitionPoint, sampleKeyed } from "../../../shared/math";
import { rectCentre } from "../../../shared/schema/actions";

export const S_MAX = 1.74;
const S_MIN = 1.18;
const S_MAX_SOFT = 1.54;

const TARGET_PAD_PX = 80;
const MENU_ROOM_PX = 110;
const TALL_TARGET_PX = 90;
const TALL_MENU_ROOM_PX = 24;
const TARGET_SHARE = 0.55;
const NARROW_TARGET_SHARE = 0.86;
const OFF_CENTRE_X = 0.26;
const OFF_CENTRE_Y = OFF_CENTRE_X * 0.65;
const CLICK_EDGE_MARGIN = 0.04;
const CLICK_EDGE_MARGIN_MAX_SHARE = 0.4;
const CLICK_PULL = 0.2;

const GROUP_TIME_GAP = 2.2;
const GROUP_DIST_FRAC = 0.28;

const STICKY_TIME_GAP = 2.8;
const STICKY_DIST_FRAC = 0.22;

const LEAD_FALLBACK_S = 0.75;
const HOLD_MIN_S = 1.3;
const TYPE_TAIL_S = 0.4;
const CAMERA_LEAD_S = 0.035;
const BASE_BEAT_S = 0.6;

const MAX_PAN_PX = 500;
const MIN_TRACK_S = 0.5;
const KEY_EPSILON_S = 1e-4;
const KEY_MERGE_S = 0.2;

const AIM_WINDOW_S = 1.2;
const NARROW_REST_EXP = 0.8;
const NARROW_REST_AIM_Y = 0.5;

export type CameraKey = { t: number; pose: CameraPose };

export type CameraTuning = {
  sMax?: number;
  holdMinS?: number;
  narrow?: number;
};

type TrackOptions = {
  speed?: number;
  tuning?: CameraTuning;
};

type ZoomOptions = TrackOptions & {
  chromeFrac?: number;
  fit?: number;
};

type Viewport = { width: number; height: number };

const logTimeS = (frame: number, fps: number, speed: number): number => (frame / fps) * speed;

export function cursorAim(log: ClickLog, tS: number): { cx: number; cy: number } | null {
  const track = log.cursorTrack;
  if (!track?.length) return null;
  const from = (tS - AIM_WINDOW_S) * 1000;
  const to = (tS + AIM_WINDOW_S) * 1000;
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

export function narrowPose(
  pose: CameraPose,
  narrow: number,
  aim: { cx: number; cy: number } | null,
): CameraPose {
  if (narrow <= 1) return pose;
  const rest = narrow ** NARROW_REST_EXP;
  const zoomed = clamp01((pose.scale - rest) / (S_MAX * narrow - rest));
  const restAim = aim ?? { cx: BASE_POSE.cx, cy: BASE_POSE.cy };
  return {
    scale: Math.max(pose.scale, rest),
    cx: lerp(restAim.cx, pose.cx, zoomed),
    cy: lerp(lerp(BASE_POSE.cy, restAim.cy, NARROW_REST_AIM_Y), pose.cy, zoomed),
  };
}

const zoomable = (clicks: ClickEvent[]): ClickEvent[] => clicks.filter((c) => c.zoom !== false);

const hasCluster = (c: ClickEvent): boolean => c.cluster != null && c.cluster !== "";

const clusterOf = (c: ClickEvent): string | undefined => (hasCluster(c) ? c.cluster : undefined);

const apartFrac = (a: ClickEvent, b: ClickEvent, vp: Viewport): number =>
  Math.hypot(b.x - a.x, b.y - a.y) / Math.hypot(vp.width, vp.height);

const gapS = (a: ClickEvent, b: ClickEvent): number => (b.tMs - a.tMs) / 1000;

export function clusterize(clicks: ClickEvent[], vp: Viewport): ClickEvent[][] {
  const list = zoomable(clicks);
  const startsGroup = (prev: ClickEvent, c: ClickEvent): boolean =>
    apartFrac(prev, c, vp) > GROUP_DIST_FRAC ||
    clusterOf(prev) !== clusterOf(c) ||
    (!hasCluster(c) && gapS(prev, c) > GROUP_TIME_GAP);
  const groups: ClickEvent[][] = [];
  list.forEach((c, i) => {
    if (i === 0 || startsGroup(list[i - 1], c)) groups.push([c]);
    else groups[groups.length - 1].push(c);
  });
  return groups;
}

export function stickify(groups: ClickEvent[][], vp: Viewport): ClickEvent[][] {
  if (groups.length <= 1) return groups;
  const explicit = groups.some((g) => g.some(hasCluster));
  const out: ClickEvent[][] = [groups[0]];
  for (const next of groups.slice(1)) {
    const prev = out[out.length - 1];
    const last = prev[prev.length - 1];
    const first = next[0];
    const otherCluster =
      explicit && hasCluster(last) && hasCluster(first) && last.cluster !== first.cluster;
    const close =
      gapS(last, first) <= STICKY_TIME_GAP && apartFrac(last, first, vp) <= STICKY_DIST_FRAC;
    if (!otherCluster && close) out[out.length - 1] = prev.concat(next);
    else out.push(next);
  }
  return out;
}

type Subject = { x: number; y: number; w: number; h: number };

function subjectOf(c: ClickEvent, width: number): Subject {
  const k = designScaleFor(width);
  const rect = c.rect ?? defaultTargetRect(c, width);
  const menuRoom = (rect.h >= TALL_TARGET_PX * k ? TALL_MENU_ROOM_PX : MENU_ROOM_PX) * k;
  const centre = rectCentre(rect);
  return {
    x: lerp(centre.x, c.x, CLICK_PULL),
    y: lerp(centre.y + menuRoom / 2, c.y, CLICK_PULL),
    w: rect.w + 2 * TARGET_PAD_PX * k,
    h: rect.h + menuRoom + 2 * TARGET_PAD_PX * k,
  };
}

function scaleFor(
  c: ClickEvent,
  subject: Subject,
  vp: Viewport,
  sMax: number,
  narrow: boolean,
): number {
  if (c.zoomScale != null) return clamp(c.zoomScale, S_MIN, sMax);
  const byWidth = ((narrow ? NARROW_TARGET_SHARE : TARGET_SHARE) * vp.width) / subject.w;
  if (narrow) return clamp(byWidth, S_MIN, sMax);
  const byHeight = (TARGET_SHARE * vp.height) / subject.h;
  return clamp(Math.min(byWidth, byHeight), S_MIN, sMax);
}

const inView = (v: number, half: number): number => clamp(v, half, 1 - half);

function offCentre(focus: number, click: number, offset: number, scale: number): number {
  const half = 1 / (2 * scale);
  const side = focus >= 0.5 ? 1 : -1;
  const aimed = inView(focus - (offset * side) / scale, half);
  const margin = Math.min(CLICK_EDGE_MARGIN, half * CLICK_EDGE_MARGIN_MAX_SHARE);
  return inView(clamp(aimed, click - half + margin, click + half - margin), half);
}

export function frameFor(c: ClickEvent, vp: Viewport, sMax: number, narrow = false): CameraPose {
  const subject = subjectOf(c, vp.width);
  const scale = scaleFor(c, subject, vp, sMax, narrow);
  const focusX = clamp01(subject.x / vp.width);
  const focusY = clamp01(subject.y / vp.height);
  if (narrow) {
    const half = 1 / (2 * scale);
    return { scale, cx: inView(focusX, half), cy: inView(focusY, half) };
  }
  return {
    scale,
    cx: offCentre(focusX, clamp01(c.x / vp.width), OFF_CENTRE_X, scale),
    cy: offCentre(focusY, clamp01(c.y / vp.height), OFF_CENTRE_Y, scale),
  };
}

function departS(c: ClickEvent): number {
  return (c.tDepartMs ?? c.tMs - LEAD_FALLBACK_S * 1000) / 1000;
}

function normalizeKeys(keys: CameraKey[]): CameraKey[] {
  const sorted = [...keys].sort((a, b) => a.t - b.t);
  const out: CameraKey[] = [];
  for (const k of sorted) {
    if (out.length === 0) {
      out.push(k);
      continue;
    }
    const prev = out[out.length - 1];
    if (k.t + KEY_EPSILON_S < prev.t) continue;
    if (k.t - prev.t < KEY_EPSILON_S) {
      out[out.length - 1] = k;
      continue;
    }
    if (k.t - prev.t < KEY_MERGE_S && posesNearlyEqual(prev.pose, k.pose)) {
      out[out.length - 1] = { t: k.t, pose: k.pose };
      continue;
    }
    out.push(k);
  }
  return out;
}

type Shot = { group: ClickEvent[]; poses: CameraPose[]; tIn: number };

type TrackPlan = {
  vp: Viewport;
  speed: number;
  holdMin: number;
  typeTail: number;
  baseBeat: number;
  endS: number;
};

const lastT = (keys: CameraKey[]): number => keys[keys.length - 1].t;

const holdUntil = (c: ClickEvent, plan: TrackPlan): number =>
  c.typeEndMs != null
    ? Math.max(c.tMs / 1000 + plan.holdMin, c.typeEndMs / 1000 + plan.typeTail)
    : c.tMs / 1000 + plan.holdMin;

function zoomIn(from: CameraPose, shot: Shot, afterT: number, plan: TrackPlan): CameraKey[] {
  const to = shot.poses[0];
  const startT = Math.max(shot.tIn, afterT);
  const endT = startT + moveDuration(from, to, "in", plan.vp, plan.speed);
  return [
    { t: startT, pose: from },
    { t: endT, pose: to },
  ];
}

function panWithin(shot: Shot, startT: number, plan: TrackPlan): CameraKey[] {
  const { vp, speed } = plan;
  const keys: CameraKey[] = [];
  let curT = startT;
  for (let j = 1; j < shot.group.length; j++) {
    const from = shot.poses[j - 1];
    const to = shot.poses[j];
    const tPan = Math.max(curT, departS(shot.group[j]) - CAMERA_LEAD_S);
    if (poseTravelPx(from, to, vp) > MAX_PAN_PX) {
      const durBack = moveDuration(from, BASE_POSE, "out", vp, speed);
      const durNext = moveDuration(BASE_POSE, to, "in", vp, speed);
      const holdEnd = Math.max(curT, holdUntil(shot.group[j - 1], plan));
      const backStart = clamp(tPan - durBack, holdEnd, tPan);
      const inStart = Math.max(backStart + durBack, tPan);
      keys.push(
        { t: backStart, pose: from },
        { t: backStart + durBack, pose: BASE_POSE },
        { t: inStart, pose: BASE_POSE },
        { t: inStart + durNext, pose: to },
      );
    } else {
      const durPan = moveDuration(from, to, "pan", vp, speed);
      keys.push({ t: tPan, pose: from }, { t: tPan + durPan, pose: to });
    }
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
        ? [
            { t: trailStart, pose },
            { t: trailStart + durOut, pose: BASE_POSE },
            { t: clipEnd, pose: BASE_POSE },
          ]
        : [{ t: clipEnd, pose }];
    return { keys, pose: BASE_POSE };
  }

  const farApart = poseTravelPx(pose, next.poses[0], plan.vp) > MAX_PAN_PX;
  if (next.tIn >= holdEnd + durOut || farApart) {
    const trailStart = Math.max(curT, Math.min(holdEnd, next.tIn - durOut));
    return {
      keys: [
        { t: trailStart, pose },
        { t: trailStart + durOut, pose: BASE_POSE },
      ],
      pose: BASE_POSE,
    };
  }
  return { keys: [{ t: Math.max(curT, next.tIn), pose }], pose };
}

export function buildCameraTrack(log: ClickLog, speed = 1, tuning: CameraTuning = {}): CameraKey[] {
  const narrowBy = tuning.narrow ?? 1;
  const narrow = narrowBy > 1;
  const sMax = (tuning.sMax ?? S_MAX) * narrowBy;
  const sMaxSoft = Math.min(S_MAX_SOFT, sMax);
  const vp = log.viewport;
  const endS = log.durationMs / 1000;
  const groups = stickify(clusterize(log.clicks, vp), vp);
  const start: CameraKey = { t: 0, pose: BASE_POSE };
  if (groups.length === 0) return [start, { t: Math.max(endS, MIN_TRACK_S), pose: BASE_POSE }];

  const plan: TrackPlan = {
    vp,
    speed,
    holdMin: (tuning.holdMinS ?? HOLD_MIN_S) * speed,
    typeTail: TYPE_TAIL_S * speed,
    baseBeat: BASE_BEAT_S * speed,
    endS,
  };
  const shots: Shot[] = groups.map((group, i) => ({
    group,
    poses: group.map((c) =>
      frameFor(c, vp, i === 0 && c.zoomScale == null && !narrow ? sMaxSoft : sMax, narrow),
    ),
    tIn: Math.max(0, departS(group[0]) - CAMERA_LEAD_S),
  }));

  const keys = [start];
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

export function sampleTrack(keys: CameraKey[], t: number): CameraPose {
  return (
    sampleKeyed(
      keys,
      t,
      (k) => k.t,
      (k) => k.pose,
      (a, b, p) => interpolatePose(a.pose, b.pose, p),
    ) ?? BASE_POSE
  );
}

type CachedTrack = { speed: number; tuning: CameraTuning; keys: CameraKey[] };

const trackCache = new WeakMap<ClickLog, CachedTrack>();

const sameTuning = (a: CameraTuning, b: CameraTuning): boolean =>
  a.sMax === b.sMax && a.holdMinS === b.holdMinS && a.narrow === b.narrow;

function cachedTrack(log: ClickLog, speed: number, tuning: CameraTuning): CameraKey[] {
  const hit = trackCache.get(log);
  if (hit && hit.speed === speed && sameTuning(hit.tuning, tuning)) return hit.keys;
  const keys = buildCameraTrack(log, speed, tuning);
  trackCache.set(log, { speed, tuning, keys });
  return keys;
}

export function poseAt(
  frame: number,
  fps: number,
  log: ClickLog,
  opts: TrackOptions = {},
): CameraPose {
  const speed = opts.speed ?? 1;
  return sampleTrack(cachedTrack(log, speed, opts.tuning ?? {}), logTimeS(frame, fps, speed));
}

export function zoomAt(
  frame: number,
  fps: number,
  log: ClickLog,
  opts: ZoomOptions = {},
): ZoomState {
  const narrow = opts.tuning?.narrow ?? 1;
  const pose = poseAt(frame, fps, log, opts);
  const aimed =
    narrow > 1
      ? narrowPose(pose, narrow, cursorAim(log, logTimeS(frame, fps, opts.speed ?? 1)))
      : pose;
  return poseToCss(aimed, opts.chromeFrac ?? 0, opts.fit ?? 1);
}
