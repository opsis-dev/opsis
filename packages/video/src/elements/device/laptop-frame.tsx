import type { CSSProperties, ReactNode } from "react";
import {
  Easing,
  getRemotionEnvironment,
  Img,
  interpolate,
  OffthreadVideo,
  staticFile,
} from "remotion";
import { themeColors } from "#video/common/theme.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import {
  mixOklch,
  resolveFont,
  type WarmTheme,
  useWarmTheme,
  withAlpha,
} from "#video/lib/warm-ui/index.ts";
import { WarmUIProvider } from "#video/lib/warm-ui/theme.ts";

export type LaptopFrameEntrance = "rise" | "open" | "none";
export type LaptopFrameFinale = "none" | "zoom-to-screen";
export type NotchPhase = "idle" | "loading" | "done";

type LaptopFrameProps = Omit<ElementSettings["laptop-frame"], "theme"> & {
  children?: ReactNode;
  theme: Partial<WarmTheme>;
  mode: "light" | "dark";
};

const FONT_FAMILY = 'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif';

export const LID_WIDTH = 820;
export const LID_HEIGHT = 500;

export const BEZEL_WIDTH = 14;

export const DECK_WIDTH = 900;
export const DECK_HEIGHT = 22;

export const LID_RADIUS = 22;

export const REST_TILT = 18;
export const PERSPECTIVE = 1600;

export const ENTRANCE_FRAMES = 30;

export const CAMERA_EASE = Easing.bezier(0.2, 0.6, 0.35, 1);

export const COLUMN_HEIGHT = LID_HEIGHT + DECK_HEIGHT;

export const NOTCH_HEIGHT = 30;
export const NOTCH_IDLE_WIDTH = 116;
export const NOTCH_LOADING_WIDTH = 150;
export const NOTCH_DONE_WIDTH = 320;

export const NOTCH_LOADING_FOR = 48;

export type ZoomTiming = { start: number; end: number };

export interface EntrancePose {
  opacity: number;
  translateY: number;
  scale: number;
}

export function entrancePose(frame: number, entrance: LaptopFrameEntrance): EntrancePose {
  const clamp = {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  } as const;
  const easeOut = Easing.out(Easing.cubic);

  if (entrance === "none") {
    return { opacity: 1, translateY: 0, scale: 1 };
  }

  if (entrance === "open") {
    return {
      opacity: interpolate(frame, [0, 10], [0, 1], clamp),
      translateY: interpolate(frame, [0, 24], [24, 0], {
        ...clamp,
        easing: easeOut,
      }),
      scale: 1,
    };
  }

  return {
    opacity: interpolate(frame, [0, 12], [0, 1], clamp),
    translateY: interpolate(frame, [0, 26], [70, 0], {
      ...clamp,
      easing: easeOut,
    }),
    scale: 1,
  };
}

export function lidTilt(
  frame: number,
  entrance: LaptopFrameEntrance,
  finale: LaptopFrameFinale,
  restTilt: number,
  zoom: ZoomTiming,
): number {
  const opened =
    entrance === "open"
      ? interpolate(frame, [0, ENTRANCE_FRAMES], [82, restTilt], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
          easing: Easing.out(Easing.cubic),
        })
      : restTilt;

  if (finale !== "zoom-to-screen" || frame < zoom.start) return opened;

  return interpolate(frame, [zoom.start, zoom.end], [restTilt, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: CAMERA_EASE,
  });
}

export interface CameraPose {
  scale: number;
  translateY: number;
}

export function cameraPose(
  frame: number,
  finale: LaptopFrameFinale,
  baseScale: number,
  compWidth: number,
  compHeight: number,
  zoom: ZoomTiming,
): CameraPose {
  if (finale !== "zoom-to-screen") return { scale: 1, translateY: 0 };

  const p = interpolate(frame, [zoom.start, zoom.end], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: CAMERA_EASE,
  });

  const innerW = (LID_WIDTH - 2 * BEZEL_WIDTH) * baseScale;
  const innerH = (LID_HEIGHT - 2 * BEZEL_WIDTH) * baseScale;
  const cover =
    compWidth >= compHeight
      ? Math.max(compWidth / innerW, compHeight / innerH)
      : Math.min(compWidth / innerW, compHeight / innerH);
  const scale = 1 + (cover - 1) * p;

  const d = (COLUMN_HEIGHT / 2 - LID_HEIGHT / 2) * baseScale;
  const translateY = d * cover * p;

  return { scale, translateY };
}

export function floatOffset(
  frame: number,
  fps: number,
  amplitude: number,
  startFrame: number,
  periodSeconds: number,
): number {
  const local = Math.max(0, frame - startFrame);
  const value = Math.sin((local / (fps * periodSeconds)) * Math.PI * 2) * amplitude;
  return value === 0 ? 0 : -value;
}

export interface NotchState {
  phase: NotchPhase;
  width: number;
  idle: number;
  loading: number;
  done: number;
}

export function notchState(
  frame: number,
  timing: { loadingStart: number; doneStart: number },
): NotchState {
  const { loadingStart, doneStart } = timing;
  const clamp = {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  } as const;

  const phase: NotchPhase = frame < loadingStart ? "idle" : frame < doneStart ? "loading" : "done";

  const width = interpolate(
    frame,
    [loadingStart - 8, loadingStart, doneStart - 4, doneStart + 12],
    [NOTCH_IDLE_WIDTH, NOTCH_LOADING_WIDTH, NOTCH_LOADING_WIDTH, NOTCH_DONE_WIDTH],
    { ...clamp, easing: Easing.inOut(Easing.cubic) },
  );

  const idle = interpolate(frame, [loadingStart - 6, loadingStart], [1, 0], clamp);
  const loading = interpolate(
    frame,
    [loadingStart - 6, loadingStart, doneStart - 6, doneStart],
    [0, 1, 1, 0],
    clamp,
  );
  const done = interpolate(frame, [doneStart - 4, doneStart + 6], [0, 1], clamp);

  return { phase, width, idle, loading, done };
}

const isVideo = (src: string) => /\.(mp4|webm|mov|m4v)(\?|$)/i.test(src);

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

function ScreenMedia({ src, frame }: { src: string; frame: number }) {
  const t = interpolate(frame, [4, 22], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const style: CSSProperties = {
    position: "absolute",
    inset: 0,
    width: "100%",
    height: "100%",
    maxWidth: "none",
    objectFit: "cover",
    opacity: t,
    filter: `blur(${(1 - t) * 8}px)`,
  };
  const resolved = resolveSrc(src);
  return isVideo(src) ? (
    <OffthreadVideo src={resolved} muted style={style} />
  ) : (
    <Img src={resolved} style={style} />
  );
}

function Dot({ size, color, style }: { size: number; color: string; style?: CSSProperties }) {
  return (
    <div
      style={{
        width: size,
        height: size,
        borderRadius: 999,
        backgroundColor: color,
        ...style,
      }}
    />
  );
}

function Notch({
  frame,
  label,
  labelSize,
  batteryLevel,
  t,
  indicatorColor,
  doneAt,
}: {
  frame: number;
  label: string;
  labelSize: number;
  batteryLevel: number;
  t: WarmTheme;
  indicatorColor: string;
  doneAt: number;
}) {
  const state = notchState(frame, { loadingStart: doneAt - NOTCH_LOADING_FOR, doneStart: doneAt });
  const layer: CSSProperties = {
    position: "absolute",
    inset: 0,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
  };

  return (
    <div
      style={{
        position: "absolute",
        top: 9,
        left: "50%",
        translate: "-50% 0px",
        width: state.width,
        height: NOTCH_HEIGHT,
        borderRadius: 999,
        backgroundColor: t.background,
        overflow: "hidden",
        zIndex: 10,
      }}
    >
      <div style={{ ...layer, gap: 6, opacity: state.idle }}>
        <Dot
          size={9}
          color={mixOklch(t.background, t.foreground, 0.12)}
          style={{
            border: `1px solid ${mixOklch(t.background, t.foreground, 0.2)}`,
          }}
        />
        <Dot size={5} color={mixOklch(t.background, t.foreground, 0.06)} />
      </div>

      <div style={{ ...layer, gap: 4, opacity: state.loading }}>
        {[0, 1, 2].map((i) => (
          <Dot
            key={i}
            size={5}
            color={t.foreground}
            style={{
              opacity: 0.4 + 0.6 * (0.5 + 0.5 * Math.sin(frame / 4 - i * 0.9)),
            }}
          />
        ))}
      </div>

      <div style={{ ...layer, gap: 8, opacity: state.done }}>
        <span
          style={{
            fontSize: labelSize,
            fontWeight: 500,
            letterSpacing: "-0.01em",
            color: t.foreground,
            whiteSpace: "nowrap",
          }}
        >
          {label}
        </span>
        <div
          style={{
            width: 22,
            height: 11,
            borderRadius: 3,
            border: `1.5px solid ${indicatorColor}`,
            padding: 1.5,
            display: "flex",
            alignItems: "center",
          }}
        >
          <div
            style={{
              width: `${Math.max(0, Math.min(100, batteryLevel))}%`,
              height: "100%",
              borderRadius: 1,
              backgroundColor: indicatorColor,
            }}
          />
        </div>
      </div>
    </div>
  );
}

function LaptopFrameView({
  children,
  screenSrc,
  entrance,
  finale,
  bezelColor,
  screenColor,
  theme,
  mode,
  fontFamily,
  indicatorColor,
  restTilt,
  radius,
  shadow,
  scale,
  showNotch,
  notchLabel,
  batteryLevel,
  floatLoop,
  floatAmplitude,
  speed,
  notchAt,
  zoomAt,
  zoomLength,
  floatPeriod,
  notchLabelSize,
}: LaptopFrameProps) {
  const frame = useAuthoredFrame() * speed;
  const t = useWarmTheme(theme, mode);
  const face = resolveFont(fontFamily || t.fontFamily) ?? FONT_FAMILY;
  const shell = useWarmTheme(theme, "dark");
  const lid = bezelColor || mixOklch(shell.background, shell.foreground, 0.09);
  const glass = screenColor || t.card;
  const drop = shadow;
  const zoom = { start: zoomAt, end: zoomAt + zoomLength };
  const { fps, width, height } = useAuthoredConfig();
  const isRendering = getRemotionEnvironment().isRendering;

  const pose = entrancePose(frame, entrance);
  const tilt = lidTilt(frame, entrance, finale, restTilt, zoom);
  const camera = cameraPose(frame, finale, scale, width, height, zoom);
  const bob =
    floatLoop && !(finale === "zoom-to-screen" && frame >= zoom.start)
      ? floatOffset(frame, fps, floatAmplitude, ENTRANCE_FRAMES, floatPeriod)
      : 0;
  const notchFade =
    finale === "zoom-to-screen"
      ? interpolate(frame, [zoom.start, zoom.start + 10], [1, 0], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
        })
      : 1;

  const screenContent =
    children ?? (screenSrc ? <ScreenMedia src={screenSrc} frame={frame} /> : null);

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontFamily: face,
      }}
    >
      <div
        style={{
          transform: `translateY(${camera.translateY}px) scale(${camera.scale})`,
          transformOrigin: "center center",
          textRendering: "geometricPrecision",
          ...(isRendering ? {} : { willChange: "transform" as const }),
        }}
      >
        <div
          style={{
            opacity: pose.opacity,
            translate: `0px ${pose.translateY + bob}px`,
            scale: `${pose.scale * scale}`,
          }}
        >
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              perspective: PERSPECTIVE,
            }}
          >
            <div
              style={{
                width: LID_WIDTH,
                height: LID_HEIGHT,
                borderRadius: `${radius}px ${radius}px 6px 6px`,
                backgroundColor: lid,
                padding: BEZEL_WIDTH,
                transformOrigin: "center bottom",
                transform: `rotateX(${tilt}deg)`,
              }}
            >
              <div
                style={{
                  position: "relative",
                  width: "100%",
                  height: "100%",
                  borderRadius: `${Math.max(4, radius - BEZEL_WIDTH)}px ${Math.max(
                    4,
                    radius - BEZEL_WIDTH,
                  )}px 4px 4px`,
                  backgroundColor: glass,
                  overflow: "hidden",
                }}
              >
                {screenContent}
                {showNotch && notchFade > 0 && (
                  <div style={{ opacity: notchFade }}>
                    <Notch
                      t={shell}
                      indicatorColor={indicatorColor}
                      frame={frame}
                      label={notchLabel}
                      labelSize={notchLabelSize}
                      batteryLevel={batteryLevel}
                      doneAt={notchAt}
                    />
                  </div>
                )}
              </div>
            </div>

            <div
              style={{
                position: "relative",
                width: DECK_WIDTH,
                height: DECK_HEIGHT,
                borderRadius: "6px 6px 40px 40px",
                background: `linear-gradient(to bottom, ${withAlpha(
                  shell.foreground,
                  0.16,
                )}, ${withAlpha(shell.background, 0)} 34%, ${withAlpha(
                  shell.background,
                  0.34,
                )}), ${lid}`,
                ...(shadow === "" ? {} : { boxShadow: drop }),
              }}
            >
              <div
                style={{
                  position: "absolute",
                  top: 0,
                  left: "50%",
                  translate: "-50% 0px",
                  width: 132,
                  height: 8,
                  borderRadius: "0 0 6px 6px",
                  backgroundColor: withAlpha(shell.background, 0.28),
                }}
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export function LaptopFrame({
  theme,
  ...settings
}: ElementSettings["laptop-frame"] & { children?: ReactNode }) {
  return (
    <WarmUIProvider mode={theme.mode}>
      <LaptopFrameView {...settings} theme={themeColors(theme)} mode={theme.mode} />
    </WarmUIProvider>
  );
}
