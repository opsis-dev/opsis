import {
  BASE_BEAT_S,
  CAMERA_LEAD_S,
  defaultTargetRect,
  DESIGN_WIDTH,
  HOLD_MIN_S,
  TYPE_TAIL_S,
  LEAD_FALLBACK_S,
  type ClickEvent,
  type ClickLog,
} from "./click-log";
import {
  BASE_POSE,
  interpolatePose,
  moveDuration,
  partitionPoint,
  poseToCss,
  posesNearlyEqual,
  poseTravelPx,
  type CameraPose,
} from "./camera";
import { clamp, lerp } from "../../../shared/math";

export const S_MAX = 1.74;
const S_MIN = 1.18;
const S_MAX_SOFT = 1.54;
const PAD = 80;
const MENU_ROOM = 110;
const TALL_TARGET = 90;
const TALL_MENU_ROOM = 24;
const FRAME_FRAC = 0.55;
const NARROW_FRAC = 0.86;
const FRAME_OFFSET = 0.26;
const FRAME_OFFSET_Y = FRAME_OFFSET * 0.65;
const CLICK_MARGIN = 0.04;
const CLICK_MARGIN_MAX = 0.4;
const CLICK_WEIGHT = 0.2;

const GROUP_TIME_GAP = 2.2;
const GROUP_DIST_FRAC = 0.28;

const STICKY_TIME_GAP = 2.8;
const STICKY_DIST_FRAC = 0.22;

const MAX_PAN_PX = 500;
const MIN_TRACK_S = 0.5;
const KEY_EPSILON_S = 1e-4;
const KEY_MERGE_S = 0.2;

const AIM_WINDOW_S = 1.2;
const NARROW_REST_EXP = 0.8;
const NARROW_REST_AIM_Y = 0.5;

type ZoomState = {
  scale: number;
  translateX: number;
  translateY: number;
};

type CameraKey = { t: number; pose: CameraPose };

export type CameraTuning = {
  sMax?: number;
  holdMinS?: number;
  narrow?: number;
};

type ZoomOptions = {
  chromeFrac?: number;
  speed?: number;
  fit?: number;
  tuning?: CameraTuning;
};

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
  const zoomed = clamp((pose.scale - rest) / (S_MAX * narrow - rest), 0, 1);
  const restAim = aim ?? { cx: BASE_POSE.cx, cy: BASE_POSE.cy };
  return {
    scale: Math.max(pose.scale, rest),
    cx: lerp(restAim.cx, pose.cx, zoomed),
    cy: lerp(lerp(BASE_POSE.cy, restAim.cy, NARROW_REST_AIM_Y), pose.cy, zoomed),
  };
}

const zoomable = (clicks: ClickEvent[]): ClickEvent[] => clicks.filter((c) => c.zoom !== false);

const hasCluster = (c: ClickEvent): boolean => c.cluster != null && c.cluster !== "";

export function clusterize(
  clicks: ClickEvent[],
  vp: { width: number; height: number },
): ClickEvent[][] {
  const list = zoomable(clicks);
  if (list.length === 0) return [];

  const diag = Math.hypot(vp.width, vp.height);
  const tooFar = (a: ClickEvent, b: ClickEvent) =>
    Math.hypot(b.x - a.x, b.y - a.y) / diag > GROUP_DIST_FRAC;

  if (list.some(hasCluster)) {
    const groups: ClickEvent[][] = [];
    let cur: ClickEvent[] = [];
    let curId: string | undefined;
    for (const c of list) {
      const id = c.cluster ?? `__auto_${groups.length}`;
      if (cur.length === 0) {
        cur = [c];
        curId = id;
        continue;
      }
      if (id === curId && !tooFar(cur[cur.length - 1], c)) {
        cur.push(c);
      } else {
        groups.push(cur);
        cur = [c];
        curId = id;
      }
    }
    if (cur.length) groups.push(cur);
    return groups;
  }

  const groups: ClickEvent[][] = [[list[0]]];
  for (let i = 1; i < list.length; i++) {
    const prev = list[i - 1];
    const cur = list[i];
    const dt = (cur.tMs - prev.tMs) / 1000;
    if (dt > GROUP_TIME_GAP || tooFar(prev, cur)) groups.push([cur]);
    else groups[groups.length - 1].push(cur);
  }
  return groups;
}

export function stickify(
  groups: ClickEvent[][],
  vp: { width: number; height: number },
): ClickEvent[][] {
  if (groups.length <= 1) return groups;
  const hasExplicit = groups.some((g) => g.some(hasCluster));
  const diag = Math.hypot(vp.width, vp.height);
  const out: ClickEvent[][] = [groups[0].slice()];
  for (let i = 1; i < groups.length; i++) {
    const prev = out[out.length - 1];
    const next = groups[i];
    const last = prev[prev.length - 1];
    const first = next[0];
    const prevId = last.cluster;
    const nextId = first.cluster;
    if (hasExplicit && prevId && nextId && prevId !== nextId) {
      out.push(next.slice());
      continue;
    }
    const dt = (first.tMs - last.tMs) / 1000;
    const dist = Math.hypot(first.x - last.x, first.y - last.y) / diag;
    if (dt <= STICKY_TIME_GAP && dist <= STICKY_DIST_FRAC) {
      out[out.length - 1] = prev.concat(next);
    } else {
      out.push(next.slice());
    }
  }
  return out;
}

export function frameFor(
  c: ClickEvent,
  vp: { width: number; height: number },
  sMax: number,
  narrow = false,
): CameraPose {
  const W = vp.width;
  const H = vp.height;
  const k = W > 0 ? W / DESIGN_WIDTH : 1;
  const rect = c.rect ?? defaultTargetRect(c, W);

  const menuRoom = (rect.h >= TALL_TARGET * k ? TALL_MENU_ROOM : MENU_ROOM) * k;
  const extW = rect.w;
  const extH = rect.h + menuRoom;
  const rectCx = rect.x + rect.w / 2;
  const rectCy = rect.y + rect.h / 2 + menuRoom / 2;
  const focusX = lerp(rectCx, c.x, CLICK_WEIGHT);
  const focusY = lerp(rectCy, c.y, CLICK_WEIGHT);

  const sByW = ((narrow ? NARROW_FRAC : FRAME_FRAC) * W) / (extW + 2 * PAD * k);
  const sByH = (FRAME_FRAC * H) / (extH + 2 * PAD * k);
  const S =
    c.zoomScale != null
      ? clamp(c.zoomScale, S_MIN, sMax)
      : clamp(narrow ? sByW : Math.min(sByW, sByH), S_MIN, sMax);

  const half = 1 / (2 * S);
  const inFrame = (v: number) => clamp(v, half, 1 - half);
  const contentCx = clamp(focusX / W, 0, 1);
  const contentCy = clamp(focusY / H, 0, 1);
  if (narrow) return { scale: S, cx: inFrame(contentCx), cy: inFrame(contentCy) };

  const sideX = contentCx >= 0.5 ? 1 : -1;
  const sideY = contentCy >= 0.5 ? 1 : -1;
  const cx = inFrame(contentCx - (FRAME_OFFSET * sideX) / S);
  const cy = inFrame(contentCy - (FRAME_OFFSET_Y * sideY) / S);

  const clickX = clamp(c.x / W, 0, 1);
  const clickY = clamp(c.y / H, 0, 1);
  const margin = Math.min(CLICK_MARGIN, half * CLICK_MARGIN_MAX);
  return {
    scale: S,
    cx: inFrame(clamp(cx, clickX - half + margin, clickX + half - margin)),
    cy: inFrame(clamp(cy, clickY - half + margin, clickY + half - margin)),
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
  vp: { width: number; height: number };
  speed: number;
  holdMin: number;
  typeTail: number;
  baseBeat: number;
  endS: number;
};

const holdUntil = (c: ClickEvent, plan: TrackPlan): number =>
  c.typeEndMs != null
    ? Math.max(c.tMs / 1000 + plan.holdMin, c.typeEndMs / 1000 + plan.typeTail)
    : c.tMs / 1000 + plan.holdMin;

function zoomIn(keys: CameraKey[], from: CameraPose, shot: Shot, plan: TrackPlan): number {
  const to = shot.poses[0];
  const startT = Math.max(shot.tIn, keys[keys.length - 1].t);
  const endT = startT + moveDuration(from, to, "in", plan.vp, plan.speed);
  keys.push({ t: startT, pose: from }, { t: endT, pose: to });
  return endT;
}

function panWithin(keys: CameraKey[], shot: Shot, startT: number, plan: TrackPlan): number {
  const { vp, speed } = plan;
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
      curT = inStart + durNext;
    } else {
      const durPan = moveDuration(from, to, "pan", vp, speed);
      keys.push({ t: tPan, pose: from }, { t: tPan + durPan, pose: to });
      curT = tPan + durPan;
    }
  }
  return curT;
}

function trailOut(
  keys: CameraKey[],
  shot: Shot,
  next: Shot | undefined,
  curT: number,
  plan: TrackPlan,
): CameraPose {
  const pose = shot.poses[shot.poses.length - 1];
  const holdEnd = Math.max(curT, holdUntil(shot.group[shot.group.length - 1], plan));
  const durOut = moveDuration(pose, BASE_POSE, "out", plan.vp, plan.speed);

  if (!next) {
    const clipEnd = Math.max(curT, plan.endS);
    const trailStart = Math.max(curT, Math.min(holdEnd, clipEnd - durOut - plan.baseBeat));
    if (trailStart + durOut <= clipEnd)
      keys.push(
        { t: trailStart, pose },
        { t: trailStart + durOut, pose: BASE_POSE },
        { t: clipEnd, pose: BASE_POSE },
      );
    else keys.push({ t: clipEnd, pose });
    return BASE_POSE;
  }

  const farApart = poseTravelPx(pose, next.poses[0], plan.vp) > MAX_PAN_PX;
  if (next.tIn >= holdEnd + durOut || farApart) {
    const trailStart = Math.max(curT, Math.min(holdEnd, next.tIn - durOut));
    keys.push({ t: trailStart, pose }, { t: trailStart + durOut, pose: BASE_POSE });
    return BASE_POSE;
  }
  keys.push({ t: Math.max(curT, next.tIn), pose });
  return pose;
}

export function buildCameraTrack(log: ClickLog, speed = 1, tuning: CameraTuning = {}): CameraKey[] {
  const narrowBy = tuning.narrow ?? 1;
  const narrow = narrowBy > 1;
  const sMax = (tuning.sMax ?? S_MAX) * narrowBy;
  const sMaxSoft = Math.min(S_MAX_SOFT, sMax);
  const vp = log.viewport;
  const endS = log.durationMs / 1000;
  const groups = stickify(clusterize(log.clicks, vp), vp);
  const keys: CameraKey[] = [{ t: 0, pose: BASE_POSE }];
  if (groups.length === 0) {
    keys.push({ t: Math.max(endS, MIN_TRACK_S), pose: BASE_POSE });
    return keys;
  }

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

  let prevPose = BASE_POSE;
  shots.forEach((shot, i) => {
    const inEnd = zoomIn(keys, prevPose, shot, plan);
    const curT = panWithin(keys, shot, inEnd, plan);
    prevPose = trailOut(keys, shot, shots[i + 1], curT, plan);
  });

  return normalizeKeys(keys);
}

export function sampleTrack(keys: CameraKey[], t: number): CameraPose {
  if (keys.length === 0) return BASE_POSE;
  if (t <= keys[0].t) return keys[0].pose;
  const last = keys[keys.length - 1];
  if (t >= last.t) return last.pose;
  const i = partitionPoint(keys, (k) => k.t < t);
  const a = keys[i - 1];
  const b = keys[i];
  const dur = b.t - a.t;
  if (dur < 1e-6) return b.pose;
  return interpolatePose(a.pose, b.pose, (t - a.t) / dur);
}

const trackCache = new WeakMap<ClickLog, Map<string, CameraKey[]>>();

function cachedTrack(log: ClickLog, speed: number, tuning?: CameraTuning): CameraKey[] {
  let byTuning = trackCache.get(log);
  if (!byTuning) {
    byTuning = new Map();
    trackCache.set(log, byTuning);
  }
  const key = JSON.stringify([speed, tuning ?? {}]);
  let track = byTuning.get(key);
  if (!track) {
    track = buildCameraTrack(log, speed, tuning);
    byTuning.set(key, track);
  }
  return track;
}

export function poseAt(
  frame: number,
  fps: number,
  log: ClickLog,
  speed = 1,
  tuning?: CameraTuning,
): CameraPose {
  return sampleTrack(cachedTrack(log, speed, tuning), logTimeS(frame, fps, speed));
}

export function zoomAt(
  frame: number,
  fps: number,
  log: ClickLog,
  opts: ZoomOptions = {},
): ZoomState {
  const chromeFrac = clamp(opts.chromeFrac ?? 0, 0, 0.2);
  const speed = opts.speed ?? 1;
  const narrow = opts.tuning?.narrow ?? 1;
  const t = logTimeS(frame, fps, speed);
  const pose = poseAt(frame, fps, log, speed, opts.tuning);
  const aimed = narrow > 1 ? narrowPose(pose, narrow, cursorAim(log, t)) : pose;
  return poseToCss(aimed, chromeFrac, opts.fit ?? 1);
}
