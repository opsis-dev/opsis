import {
  BASE_BEAT_S,
  CAMERA_LEAD_S,
  defaultTargetRect,
  DESIGN_WIDTH,
  END_TAIL_S,
  HOLD_MIN_S,
  TYPE_TAIL_S,
  LEAD_FALLBACK_S,
  type ClickEvent,
  type ClickLog,
} from "./click-log";
import {
  BASE_POSE,
  interpolatePose,
  MAX_PAN_PX,
  mix,
  moveDuration,
  poseToCss,
  posesNearlyEqual,
  poseTravelPx,
  type CameraPose,
} from "./camera";
import { clamp } from "../../../shared/math";

export type { ClickEvent, ClickLog } from "./click-log";
export { EMPTY_LOG } from "./click-log";

export type ZoomState = {
  scale: number;
  translateX: number;
  translateY: number;
};

export type CameraKey = { t: number; pose: CameraPose };

export type CameraTuning = {
  sMax?: number;
  holdMinS?: number;
  narrow?: number;
};

export type ZoomOptions = {
  chromeFrac?: number;
  speed?: number;
  fit?: number;
  drift?: number;
  tuning?: CameraTuning;
  boost?: number;
};

export function boostPose(pose: CameraPose, boost = 1): CameraPose {
  if (boost <= 1 || pose.scale <= 1) return pose;
  return { ...pose, scale: pose.scale ** (1 + Math.log(boost) / Math.log(S_MAX)) };
}

const AIM_WINDOW_S = 1.2;

export function cursorAim(log: ClickLog, tS: number): { cx: number; cy: number } | null {
  const track = log.cursorTrack;
  if (!track?.length) return null;
  const from = (tS - AIM_WINDOW_S) * 1000;
  const to = (tS + AIM_WINDOW_S) * 1000;
  let lo = 0;
  let hi = track.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (track[mid].t < from) lo = mid + 1;
    else hi = mid;
  }
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
  boost: number,
  aim: { cx: number; cy: number } | null,
): CameraPose {
  if (boost <= 1) return pose;
  const rest = boost ** 0.8;
  const zoomed = clamp((pose.scale - rest) / (S_MAX * boost - rest), 0, 1);
  const restAim = aim ?? { cx: BASE_POSE.cx, cy: BASE_POSE.cy };
  return {
    scale: Math.max(pose.scale, rest),
    cx: mix(restAim.cx, pose.cx, zoomed),
    cy: mix(mix(BASE_POSE.cy, restAim.cy, 0.5), pose.cy, zoomed),
  };
}

const S_MIN = 1.18;
const S_MAX = 1.74;
const S_MAX_SOFT = 1.54;
const PAD = 80;
const MENU_ROOM = 110;
const FRAME_FRAC = 0.55;
const NARROW_FRAC = 0.86;
const FRAME_OFFSET = 0.26;
const CLICK_MARGIN = 0.04;
const CLICK_WEIGHT = 0.2;

const GROUP_TIME_GAP = 2.2;
const GROUP_DIST_FRAC = 0.28;

const STICKY_TIME_GAP = 2.8;
const STICKY_DIST_FRAC = 0.22;

const zoomable = (clicks: ClickEvent[]): ClickEvent[] => clicks.filter((c) => c.zoom !== false);

export function clusterize(
  clicks: ClickEvent[],
  vp: { width: number; height: number },
): ClickEvent[][] {
  const list = zoomable(clicks);
  if (list.length === 0) return [];

  const diag = Math.hypot(vp.width, vp.height);
  const tooFar = (a: ClickEvent, b: ClickEvent) =>
    Math.hypot(b.x - a.x, b.y - a.y) / diag > GROUP_DIST_FRAC;

  const hasExplicit = list.some((c) => c.cluster != null && c.cluster !== "");
  if (hasExplicit) {
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
  const hasExplicit = groups.some((g) => g.some((c) => c.cluster != null && c.cluster !== ""));
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

  const menuRoom = (rect.h >= 90 * k ? 24 : MENU_ROOM) * k;
  const extW = rect.w;
  const extH = rect.h + menuRoom;
  const rectCx = rect.x + rect.w / 2;
  const rectCy = rect.y + rect.h / 2 + menuRoom / 2;
  const focusX = mix(rectCx, c.x, CLICK_WEIGHT);
  const focusY = mix(rectCy, c.y, CLICK_WEIGHT);

  const sByW = ((narrow ? NARROW_FRAC : FRAME_FRAC) * W) / (extW + 2 * PAD * k);
  const sByH = (FRAME_FRAC * H) / (extH + 2 * PAD * k);
  const S =
    c.zoomScale != null
      ? clamp(c.zoomScale, S_MIN, sMax)
      : clamp(narrow ? sByW : Math.min(sByW, sByH), S_MIN, sMax);

  const contentCx = clamp(focusX / W, 0, 1);
  const contentCy = clamp(focusY / H, 0, 1);
  if (narrow) {
    const h = 1 / (2 * S);
    return { scale: S, cx: clamp(contentCx, h, 1 - h), cy: clamp(contentCy, h, 1 - h) };
  }
  const sideX = contentCx >= 0.5 ? 1 : -1;
  const sideY = contentCy >= 0.5 ? 1 : -1;
  const half = 1 / (2 * S);
  let cx = clamp(contentCx - (FRAME_OFFSET * sideX) / S, half, 1 - half);
  let cy = clamp(contentCy - (FRAME_OFFSET * 0.65 * sideY) / S, half, 1 - half);

  const clickX = clamp(c.x / W, 0, 1);
  const clickY = clamp(c.y / H, 0, 1);
  const margin = Math.min(CLICK_MARGIN, half * 0.4);
  cx = clamp(cx, clickX - half + margin, clickX + half - margin);
  cy = clamp(cy, clickY - half + margin, clickY + half - margin);
  cx = clamp(cx, half, 1 - half);
  cy = clamp(cy, half, 1 - half);
  return { scale: S, cx, cy };
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
    if (k.t + 1e-4 < prev.t) continue;
    if (k.t - prev.t < 1e-4) {
      out[out.length - 1] = k;
      continue;
    }
    if (k.t - prev.t < 0.2 && posesNearlyEqual(prev.pose, k.pose)) {
      out[out.length - 1] = { t: k.t, pose: k.pose };
      continue;
    }
    out.push(k);
  }
  return out;
}

export function buildCameraTrack(log: ClickLog, speed = 1, tuning: CameraTuning = {}): CameraKey[] {
  const narrow = (tuning.narrow ?? 1) > 1;
  const sMax = (tuning.sMax ?? S_MAX) * (narrow ? tuning.narrow! : 1);
  const sMaxSoft = Math.min(S_MAX_SOFT, sMax);
  const vp = log.viewport;
  const groups = stickify(clusterize(log.clicks, vp), vp);
  const keys: CameraKey[] = [{ t: 0, pose: BASE_POSE }];
  if (groups.length === 0) {
    keys.push({
      t: Math.max((log.durationMs || 8000) / 1000, 0.5),
      pose: BASE_POSE,
    });
    return keys;
  }

  const holdMin = (tuning.holdMinS ?? HOLD_MIN_S) * speed;
  const typeTail = TYPE_TAIL_S * speed;
  const baseBeat = BASE_BEAT_S * speed;

  const holdUntil = (c: ClickEvent): number =>
    c.typeEndMs != null
      ? Math.max(c.tMs / 1000 + holdMin, c.typeEndMs / 1000 + typeTail)
      : c.tMs / 1000 + holdMin;

  let prevPose = BASE_POSE;
  const meta = groups.map((group, i) => ({
    group,
    poses: group.map((c) =>
      frameFor(c, vp, i === 0 && c.zoomScale == null && !narrow ? sMaxSoft : sMax, narrow),
    ),
    tIn: Math.max(0, departS(group[0]) - CAMERA_LEAD_S),
  }));

  for (let i = 0; i < meta.length; i++) {
    const { group, poses, tIn } = meta[i];
    const last = group[group.length - 1];
    const inPose = poses[0];
    const startT = Math.max(tIn, keys[keys.length - 1].t);
    const durIn = moveDuration(prevPose, inPose, "in", vp, speed);
    keys.push({ t: startT, pose: prevPose });
    keys.push({ t: startT + durIn, pose: inPose });

    let curPose = inPose;
    let curT = startT + durIn;
    for (let j = 1; j < group.length; j++) {
      const nextPose = poses[j];
      const tPan = Math.max(curT, departS(group[j]) - CAMERA_LEAD_S);
      if (poseTravelPx(curPose, nextPose, vp) > MAX_PAN_PX) {
        const durBack = moveDuration(curPose, BASE_POSE, "out", vp, speed);
        const durNext = moveDuration(BASE_POSE, nextPose, "in", vp, speed);
        const prevHoldEnd = holdUntil(group[j - 1]);
        const backStart = clamp(tPan - durBack, Math.max(curT, prevHoldEnd), Math.max(curT, tPan));
        keys.push({ t: backStart, pose: curPose });
        keys.push({ t: backStart + durBack, pose: BASE_POSE });
        const inStart = Math.max(backStart + durBack, tPan);
        keys.push({ t: inStart, pose: BASE_POSE });
        keys.push({ t: inStart + durNext, pose: nextPose });
        curT = inStart + durNext;
      } else {
        const durPan = moveDuration(curPose, nextPose, "pan", vp, speed);
        keys.push({ t: tPan, pose: curPose });
        keys.push({ t: tPan + durPan, pose: nextPose });
        curT = tPan + durPan;
      }
      curPose = nextPose;
    }

    const holdEnd = Math.max(curT, holdUntil(last));
    const durOut = moveDuration(curPose, BASE_POSE, "out", vp, speed);
    const next = meta[i + 1];

    if (!next) {
      const clipEnd = Math.max(curT, (log.durationMs || last.tMs + END_TAIL_S * 1000) / 1000);
      const latest = clipEnd - durOut - baseBeat;
      const trailStart = Math.max(curT, Math.min(holdEnd, latest));
      if (trailStart + durOut <= clipEnd) {
        keys.push({ t: trailStart, pose: curPose });
        keys.push({ t: trailStart + durOut, pose: BASE_POSE });
        keys.push({ t: clipEnd, pose: BASE_POSE });
      } else {
        keys.push({ t: clipEnd, pose: curPose });
      }
      prevPose = BASE_POSE;
      continue;
    }

    const farApart = poseTravelPx(curPose, next.poses[0], vp) > MAX_PAN_PX;
    if (next.tIn >= holdEnd + durOut || farApart) {
      const trailStart = Math.max(curT, Math.min(holdEnd, next.tIn - durOut));
      keys.push({ t: trailStart, pose: curPose });
      keys.push({ t: trailStart + durOut, pose: BASE_POSE });
      prevPose = BASE_POSE;
    } else if (next.tIn > holdEnd) {
      keys.push({ t: next.tIn, pose: curPose });
      prevPose = curPose;
    } else {
      keys.push({ t: Math.max(curT, next.tIn), pose: curPose });
      prevPose = curPose;
    }
  }

  return normalizeKeys(keys);
}

export function sampleTrack(keys: CameraKey[], t: number): CameraPose {
  if (keys.length === 0) return BASE_POSE;
  if (t <= keys[0].t) return keys[0].pose;
  const last = keys[keys.length - 1];
  if (t >= last.t) return last.pose;
  let i = 0;
  while (i < keys.length - 1 && t > keys[i + 1].t) i++;
  const a = keys[i];
  const b = keys[i + 1];
  const dur = b.t - a.t;
  if (dur < 1e-6) return b.pose;
  return interpolatePose(a.pose, b.pose, (t - a.t) / dur);
}

export const DRIFT_RATE = 0.02;
export const DRIFT_PEAK_FRAC = 0.03;
export const DRIFT_MIN_HOLD_S = 2.0;
export const DRIFT_MIN_SCALE = 0.02;

export function holdSpanAt(keys: CameraKey[], t: number): { startT: number; endT: number } | null {
  if (keys.length < 2 || t <= keys[0].t) return null;
  const last = keys[keys.length - 1];
  if (t >= last.t) return null;
  let i = 0;
  while (i < keys.length - 1 && t > keys[i + 1].t) i++;
  const a = keys[i];
  const b = keys[i + 1];
  return posesNearlyEqual(a.pose, b.pose) ? { startT: a.t, endT: b.t } : null;
}

export function driftScale(scale: number, u: number, holdScreenS: number, amount: number): number {
  const peak = Math.min(
    DRIFT_PEAK_FRAC * scale,
    (DRIFT_RATE * holdScreenS) / Math.PI,
    Math.max(0, S_MAX - scale),
  );
  return scale + (amount * peak * (1 - Math.cos(2 * Math.PI * u))) / 2;
}

export function driftPose(
  pose: CameraPose,
  keys: CameraKey[],
  t: number,
  speed: number,
  amount: number,
): CameraPose {
  if (!(amount > 0)) return pose;
  if (pose.scale - BASE_POSE.scale < DRIFT_MIN_SCALE) return pose;
  const span = holdSpanAt(keys, t);
  if (!span) return pose;
  const holdScreenS = (span.endT - span.startT) / (speed || 1);
  if (holdScreenS < DRIFT_MIN_HOLD_S) return pose;
  const u = (t - span.startT) / (span.endT - span.startT);
  return { ...pose, scale: driftScale(pose.scale, u, holdScreenS, amount) };
}

const trackCache = new WeakMap<ClickLog, Map<string, CameraKey[]>>();

function cachedTrack(log: ClickLog, speed: number, tuning: CameraTuning = {}): CameraKey[] {
  let bySpeed = trackCache.get(log);
  if (!bySpeed) {
    bySpeed = new Map();
    trackCache.set(log, bySpeed);
  }
  const key = `${speed}|${tuning.sMax ?? ""}|${tuning.holdMinS ?? ""}|${tuning.narrow ?? ""}`;
  let track = bySpeed.get(key);
  if (!track) {
    track = buildCameraTrack(log, speed, tuning);
    bySpeed.set(key, track);
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
  const t = (frame / fps) * speed - (log.offsetMs ?? 0) / 1000;
  return sampleTrack(cachedTrack(log, speed, tuning), t);
}

export function zoomAt(
  frame: number,
  fps: number,
  log: ClickLog,
  opts: ZoomOptions = {},
): ZoomState {
  const chromeFrac = clamp(opts.chromeFrac ?? 0, 0, 0.2);
  const speed = opts.speed && opts.speed > 0 ? opts.speed : 1;
  const fit = opts.fit && opts.fit > 0 ? opts.fit : 1;
  const offset = (log.offsetMs ?? 0) / 1000;
  const t = (frame / fps) * speed - offset;
  const boostBy = opts.boost ?? 1;
  const track = cachedTrack(
    log,
    speed,
    boostBy > 1 ? { ...opts.tuning, narrow: boostBy } : opts.tuning,
  );
  const pose = driftPose(sampleTrack(track, t), track, t, speed, opts.drift ?? 0);
  const boost = opts.boost ?? 1;
  return poseToCss(boost > 1 ? narrowPose(pose, boost, cursorAim(log, t)) : pose, chromeFrac, fit);
}
