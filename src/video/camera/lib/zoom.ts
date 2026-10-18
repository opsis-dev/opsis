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
import { rectCentre, type Viewport } from "../../../shared/schema/actions";
import { stableJson } from "../../../shared/hash";

export const S_MAX = 1.74;

const ZOOM = { min: 1.18, firstShotMax: 1.54 } as const;

const SUBJECT = {
  padPx: 80,
  menuRoomPx: 110,
  tallTargetPx: 90,
  tallMenuRoomPx: 24,
  clickPull: 0.2,
} as const;

const FRAMING = {
  targetShare: 0.55,
  narrowTargetShare: 0.86,
  offCentre: { x: 0.26, y: 0.26 * 0.65 },
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

type CameraTrack = [CameraKey, ...CameraKey[]];

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

type Aim = Pick<CameraPose, "cx" | "cy">;

export const logTimeS = (frame: number, fps: number, speed: number): number =>
  (frame / fps) * speed;

const narrowFactor = (tuning: CameraTuning = {}): number => tuning.narrow ?? 1;

export const isNarrow = (tuning: CameraTuning = {}): boolean => narrowFactor(tuning) > 1;

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

const zoomCeiling = (tuning: CameraTuning): number => (tuning.sMax ?? S_MAX) * narrowFactor(tuning);

export function narrowPose(
  pose: CameraPose,
  tuning: CameraTuning,
  aimAt: () => Aim | null,
): CameraPose {
  if (!isNarrow(tuning)) return pose;
  const rest = narrowFactor(tuning) ** NARROW_AIM.restExp;
  const zoomed = clamp01((pose.scale - rest) / (zoomCeiling(tuning) - rest));
  const restAim = aimAt() ?? { cx: BASE_POSE.cx, cy: BASE_POSE.cy };
  return {
    scale: Math.max(pose.scale, rest),
    cx: lerp(restAim.cx, pose.cx, zoomed),
    cy: lerp(lerp(BASE_POSE.cy, restAim.cy, NARROW_AIM.restAimY), pose.cy, zoomed),
  };
}

const zoomable = (clicks: ClickEvent[]): ClickEvent[] => clicks.filter((c) => c.zoom !== false);

const apartFrac = (a: ClickEvent, b: ClickEvent, vp: Viewport): number =>
  Math.hypot(b.x - a.x, b.y - a.y) / Math.hypot(vp.width, vp.height);

const gapS = (a: ClickEvent, b: ClickEvent): number => (b.tMs - a.tMs) / 1000;

export function clusterize(clicks: ClickEvent[], vp: Viewport): ClickEvent[][] {
  const list = zoomable(clicks);
  const startsGroup = (prev: ClickEvent, c: ClickEvent): boolean =>
    apartFrac(prev, c, vp) > GROUPING.distFrac ||
    prev.cluster !== c.cluster ||
    (c.cluster === undefined && gapS(prev, c) > GROUPING.gapS);
  const groups: ClickEvent[][] = [];
  list.forEach((c, i) => {
    if (i === 0 || startsGroup(list[i - 1], c)) groups.push([c]);
    else groups[groups.length - 1].push(c);
  });
  return groups;
}

export function stickify(groups: ClickEvent[][], vp: Viewport): ClickEvent[][] {
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
    const close = gapS(last, first) <= STICKY.gapS && apartFrac(last, first, vp) <= STICKY.distFrac;
    if (!otherCluster && close) out[out.length - 1] = prev.concat(next);
    else out.push(next);
  }
  return out;
}

type Subject = { x: number; y: number; w: number; h: number };

function subjectOf(c: ClickEvent, width: number): Subject {
  const k = designScaleFor(width);
  const rect = c.rect ?? defaultTargetRect(c, width);
  const menuRoom =
    (rect.h >= SUBJECT.tallTargetPx * k ? SUBJECT.tallMenuRoomPx : SUBJECT.menuRoomPx) * k;
  const centre = rectCentre(rect);
  return {
    x: lerp(centre.x, c.x, SUBJECT.clickPull),
    y: lerp(centre.y + menuRoom / 2, c.y, SUBJECT.clickPull),
    w: rect.w + 2 * SUBJECT.padPx * k,
    h: rect.h + menuRoom + 2 * SUBJECT.padPx * k,
  };
}

function scaleFor(
  c: ClickEvent,
  subject: Subject,
  vp: Viewport,
  sMax: number,
  narrow: boolean,
): number {
  if (c.zoomScale != null) return clamp(c.zoomScale, ZOOM.min, sMax);
  const share = narrow ? FRAMING.narrowTargetShare : FRAMING.targetShare;
  const byWidth = (share * vp.width) / subject.w;
  if (narrow) return clamp(byWidth, ZOOM.min, sMax);
  const byHeight = (FRAMING.targetShare * vp.height) / subject.h;
  return clamp(Math.min(byWidth, byHeight), ZOOM.min, sMax);
}

const inView = (v: number, half: number): number => clamp(v, half, 1 - half);

function offCentre(focus: number, click: number, offset: number, scale: number): number {
  const half = 1 / (2 * scale);
  const side = focus >= 0.5 ? 1 : -1;
  const aimed = inView(focus - (offset * side) / scale, half);
  const margin = Math.min(FRAMING.clickEdgeMargin, half * FRAMING.clickEdgeMarginMaxShare);
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
    cx: offCentre(focusX, clamp01(c.x / vp.width), FRAMING.offCentre.x, scale),
    cy: offCentre(focusY, clamp01(c.y / vp.height), FRAMING.offCentre.y, scale),
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
    const tPan = Math.max(curT, departS(shot.group[j]) - TIMING.cameraLeadS);
    if (poseTravelPx(from, to, vp) > TRACK.maxPanPx) {
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

  const farApart = poseTravelPx(pose, next.poses[0], plan.vp) > TRACK.maxPanPx;
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

export function buildCameraTrack(log: ClickLog, speed = 1, tuning: CameraTuning = {}): CameraTrack {
  const narrow = isNarrow(tuning);
  const sMax = zoomCeiling(tuning);
  const firstShotMax = Math.min(ZOOM.firstShotMax, sMax);
  const vp = log.viewport;
  const endS = log.durationMs / 1000;
  const groups = stickify(clusterize(log.clicks, vp), vp);
  const start: CameraKey = { t: 0, pose: BASE_POSE };
  if (groups.length === 0) return [start, { t: Math.max(endS, TRACK.minS), pose: BASE_POSE }];

  const plan: TrackPlan = {
    vp,
    speed,
    holdMin: (tuning.holdMinS ?? TIMING.holdMinS) * speed,
    typeTail: TIMING.typeTailS * speed,
    baseBeat: TIMING.baseBeatS * speed,
    endS,
  };
  const shots: Shot[] = groups.map((group, i) => ({
    group,
    poses: group.map((c) =>
      frameFor(c, vp, i === 0 && c.zoomScale == null && !narrow ? firstShotMax : sMax, narrow),
    ),
    tIn: Math.max(0, departS(group[0]) - TIMING.cameraLeadS),
  }));

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

export function sampleTrack(keys: CameraTrack, t: number): CameraPose {
  return sampleKeyed(
    keys,
    t,
    (k) => k.t,
    (k) => k.pose,
    (a, b, p) => interpolatePose(a.pose, b.pose, p),
  );
}

const trackCache = new WeakMap<ClickLog, { key: string; keys: CameraTrack }>();

function cachedTrack(log: ClickLog, speed: number, tuning: CameraTuning): CameraTrack {
  const key = stableJson({ speed, tuning });
  const hit = trackCache.get(log);
  if (hit?.key === key) return hit.keys;
  const keys = buildCameraTrack(log, speed, tuning);
  trackCache.set(log, { key, keys });
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
  const pose = narrowPose(poseAt(frame, fps, log, opts), opts.tuning ?? {}, () =>
    cursorAim(log, logTimeS(frame, fps, opts.speed ?? 1)),
  );
  return poseToCss(pose, opts.chromeFrac ?? 0, opts.fit ?? 1);
}
