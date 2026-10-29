import type { CSSProperties } from "react";
import {
  AbsoluteFill,
  Easing,
  getRemotionEnvironment,
  Img,
  interpolate,
  OffthreadVideo,
  staticFile,
  useVideoConfig,
} from "remotion";
import { themeColors } from "#video/common/theme.ts";
import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { type WarmTheme, useWarmTheme } from "#video/lib/warm-ui/index.ts";
import { WarmUIProvider } from "#video/lib/warm-ui/theme.ts";

export type ScreenRecordingFit = "cover" | "contain";
export type ScreenRecordingEntrance = "fade" | "none";

export interface ScreenCrop {
  top?: number;
  right?: number;
  bottom?: number;
  left?: number;
}

export interface CameraMove {
  at: number;
  duration?: number;
  zoom?: number;
  x?: number;
  y?: number;
}

type ScreenRecordingProps = Omit<ElementSettings["screen-recording"], "theme"> & {
  theme: Partial<WarmTheme>;
  mode: "light" | "dark";
};

export const CAMERA_EASE = Easing.bezier(0.2, 0.6, 0.35, 1);

export const CAMERA_MOVE_FRAMES = 25;

export const TAIL_FRAMES = 18;

export const ENTRANCE_FRAMES = 18;

const ENTRANCE_START = 4;

const ENTRANCE_BLUR = 8;

export interface CameraPose {
  zoom: number;
  x: number;
  y: number;
}

export const RESTING_POSE: CameraPose = { zoom: 1, x: 0.5, y: 0.5 };

const lerp = (a: number, b: number, p: number) => a + (b - a) * p;

function moveProgress(frame: number, at: number, duration: number): number {
  if (duration <= 0) return 1;
  return interpolate(frame, [at, at + duration], [0, 1], {
    easing: CAMERA_EASE,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
}

export function cameraPoseAt(frame: number, moves?: CameraMove[]): CameraPose {
  if (!moves || moves.length === 0) return RESTING_POSE;
  const sorted = moves
    .map((move, index) => ({ move, index }))
    .sort((a, b) => a.move.at - b.move.at || a.index - b.index)
    .filter((e, i, arr) => i === arr.length - 1 || arr[i + 1].move.at !== e.move.at)
    .map((e) => e.move);

  let on = RESTING_POSE;
  let target = RESTING_POSE;
  let current: {
    from: CameraPose;
    to: CameraPose;
    at: number;
    dur: number;
  } | null = null;

  for (const move of sorted) {
    if (move.at > frame) break;
    if (current) {
      const p = moveProgress(move.at, current.at, current.dur);
      on = {
        zoom: lerp(current.from.zoom, current.to.zoom, p),
        x: lerp(current.from.x, current.to.x, p),
        y: lerp(current.from.y, current.to.y, p),
      };
    }
    target = {
      zoom: move.zoom ?? target.zoom,
      x: move.x ?? target.x,
      y: move.y ?? target.y,
    };
    current = {
      from: on,
      to: target,
      at: move.at,
      dur: move.duration ?? CAMERA_MOVE_FRAMES,
    };
  }

  if (!current) return on;
  const p = moveProgress(frame, current.at, current.dur);
  return {
    zoom: lerp(current.from.zoom, current.to.zoom, p),
    x: lerp(current.from.x, current.to.x, p),
    y: lerp(current.from.y, current.to.y, p),
  };
}

export function clampFocus(value: number, zoom: number): number {
  const lo = 0.5 / zoom;
  const hi = 1 - lo;
  if (lo >= hi) return 0.5;
  return Math.min(Math.max(value, lo), hi);
}

export function screenRecordingFrames(camera?: CameraMove[]): number {
  if (!camera || camera.length === 0) return 0;
  const end = Math.max(...camera.map((m) => m.at + (m.duration ?? CAMERA_MOVE_FRAMES)));
  return Math.round(end + TAIL_FRAMES);
}

export interface ShotLayout {
  left: number;
  top: number;
  width: number;
  height: number;
  mediaWidth: number;
  mediaHeight: number;
  mediaLeft: number;
  mediaTop: number;
}

export function shotLayout(
  width: number,
  height: number,
  crop: ScreenCrop | undefined,
  fit: ScreenRecordingFit,
  sourceAspect: number | undefined,
): ShotLayout {
  const left = crop?.left ?? 0;
  const top = crop?.top ?? 0;
  const keepX = Math.max(1 - left - (crop?.right ?? 0), 0.01);
  const keepY = Math.max(1 - top - (crop?.bottom ?? 0), 0.01);
  const aspect = sourceAspect && sourceAspect > 0 ? sourceAspect : width / height;

  const regionW = aspect * keepX;
  const regionH = keepY;
  const k =
    fit === "contain"
      ? Math.min(width / regionW, height / regionH)
      : Math.max(width / regionW, height / regionH);

  const w = regionW * k;
  const h = regionH * k;
  return {
    left: (width - w) / 2,
    top: (height - h) / 2,
    width: w,
    height: h,
    mediaWidth: aspect * k,
    mediaHeight: k,
    mediaLeft: -left * aspect * k,
    mediaTop: -top * k,
  };
}

const isVideo = (src: string) => /\.(mp4|webm|mov|m4v)(\?|$)/i.test(src);

const pct = (value: number, of: number) => `${(value / of) * 100}%`;

function resolveSrc(src: string): string {
  const isLocal = src.startsWith("/") && !src.startsWith("//");
  if (!isLocal || getRemotionEnvironment().isPlayer) return src;
  const base = staticFile("_").slice(0, -2);
  if (base && src.startsWith(`${base}/`)) return src;
  try {
    return staticFile(src.replace(/^\/+/, ""));
  } catch {
    return src;
  }
}

function ScreenRecordingView({
  src,
  cropTop,
  cropRight,
  cropBottom,
  cropLeft,
  sourceAspect,
  fit,
  backdropColor,
  camera,
  radius,
  trimBefore,
  audio,
  entrance,
  theme,
  mode,
  speed,
}: ScreenRecordingProps) {
  const frame = useAuthoredFrame() * speed;
  const t = useWarmTheme(theme, mode);
  const { width, height } = useVideoConfig();
  const isRendering = getRemotionEnvironment().isRendering;

  const shot = shotLayout(
    width,
    height,
    { top: cropTop, right: cropRight, bottom: cropBottom, left: cropLeft },
    fit,
    sourceAspect,
  );
  const pose = cameraPoseAt(frame, camera);
  const focusX = clampFocus(pose.x, pose.zoom);
  const focusY = clampFocus(pose.y, pose.zoom);
  const tx = (0.5 - focusX * pose.zoom) * 100;
  const ty = (0.5 - focusY * pose.zoom) * 100;

  const arrival =
    entrance === "none"
      ? 1
      : interpolate(frame, [ENTRANCE_START, ENTRANCE_START + ENTRANCE_FRAMES], [0, 1], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
        });

  const mediaStyle: CSSProperties = {
    position: "absolute",
    left: pct(shot.mediaLeft, shot.width),
    top: pct(shot.mediaTop, shot.height),
    width: pct(shot.mediaWidth, shot.width),
    height: pct(shot.mediaHeight, shot.height),
    maxWidth: "none",
    objectFit: "cover",
    opacity: arrival,
    ...(arrival < 1 ? { filter: `blur(${(1 - arrival) * ENTRANCE_BLUR}px)` } : null),
  };
  const resolved = resolveSrc(src);

  return (
    <AbsoluteFill
      style={{
        backgroundColor: backdropColor || t.background,
        borderRadius: radius,
        overflow: "hidden",
      }}
    >
      <div
        style={{
          position: "absolute",
          inset: 0,
          transformOrigin: "0 0",
          transform: `translate(${tx}%, ${ty}%) scale(${pose.zoom})`,
          ...(isRendering ? {} : { willChange: "transform" as const }),
        }}
      >
        <div
          style={{
            position: "absolute",
            left: pct(shot.left, width),
            top: pct(shot.top, height),
            width: pct(shot.width, width),
            height: pct(shot.height, height),
            overflow: "hidden",
          }}
        >
          {isVideo(src) ? (
            <OffthreadVideo
              src={resolved}
              muted={!audio}
              trimBefore={trimBefore}
              playbackRate={speed}
              style={mediaStyle}
            />
          ) : (
            <Img src={resolved} style={mediaStyle} />
          )}
        </div>
      </div>
    </AbsoluteFill>
  );
}

export function ScreenRecording({ theme, ...settings }: ElementSettings["screen-recording"]) {
  return (
    <WarmUIProvider mode={theme.mode}>
      <ScreenRecordingView {...settings} theme={themeColors(theme)} mode={theme.mode} />
    </WarmUIProvider>
  );
}
