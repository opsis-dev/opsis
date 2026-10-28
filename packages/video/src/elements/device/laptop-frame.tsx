import type { CSSProperties, ReactNode } from "react";
import type React from "react";
import {
  Easing,
  getRemotionEnvironment,
  Img,
  interpolate,
  OffthreadVideo,
  staticFile,
} from "remotion";
import { type ElementProps } from "#video/common/spec.ts";
import { isTheme, themeColors } from "#video/common/theme.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
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

export interface LaptopFrameProps {
  children?: ReactNode;
  screenSrc?: string;
  entrance?: LaptopFrameEntrance;
  finale?: LaptopFrameFinale;
  bezelColor?: string;
  screenColor?: string;
  theme?: Partial<WarmTheme>;
  mode?: "light" | "dark";
  fontFamily?: string;
  indicatorColor?: string;
  restTilt?: number;
  radius?: number;
  shadow?: string;
  scale?: number;
  showNotch?: boolean;
  notchLabel?: string;
  batteryLevel?: number;
  floatLoop?: boolean;
  floatAmplitude?: number;
  speed?: number;
  className?: string;
}

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

export const PUSH_START = 140;
export const PUSH_END = 185;

export const CAMERA_EASE = Easing.bezier(0.2, 0.6, 0.35, 1);

export const COLUMN_HEIGHT = LID_HEIGHT + DECK_HEIGHT;

export const FLOAT_PERIOD_SECONDS = 5;

export const NOTCH_HEIGHT = 30;
export const NOTCH_IDLE_WIDTH = 116;
export const NOTCH_LOADING_WIDTH = 150;
export const NOTCH_DONE_WIDTH = 320;

export const NOTCH_TIMING = {
  loadingStart: 72,
  doneStart: 120,
} as const;

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
  finale: LaptopFrameFinale = "none",
  restTilt: number = REST_TILT,
): number {
  const opened =
    entrance === "open"
      ? interpolate(frame, [0, ENTRANCE_FRAMES], [82, restTilt], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
          easing: Easing.out(Easing.cubic),
        })
      : restTilt;

  if (finale !== "zoom-to-screen" || frame < PUSH_START) return opened;

  return interpolate(frame, [PUSH_START, PUSH_END], [restTilt, 0], {
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
): CameraPose {
  if (finale !== "zoom-to-screen") return { scale: 1, translateY: 0 };

  const p = interpolate(frame, [PUSH_START, PUSH_END], [0, 1], {
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
  startFrame = 0,
  periodSeconds: number = FLOAT_PERIOD_SECONDS,
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
  timing: { loadingStart: number; doneStart: number } = NOTCH_TIMING,
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

function PlaceholderScreen({ frame, t }: { frame: number; t: WarmTheme }) {
  const rise = (delay: number) => {
    const r = interpolate(frame, [delay, delay + 14], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
      easing: Easing.out(Easing.cubic),
    });
    return { opacity: r, transform: `translateY(${(1 - r) * 14}px)` };
  };

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        background: `linear-gradient(165deg, ${t.card} 0%, ${mixOklch(
          t.card,
          t.primary,
          0.06,
        )} 100%)`,
        color: t.foreground,
        display: "flex",
        flexDirection: "column",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "22px 34px",
          ...rise(6),
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 9 }}>
          <div
            style={{
              width: 24,
              height: 24,
              borderRadius: 7,
              background: `linear-gradient(135deg, ${t.primary}, ${mixOklch(
                t.primary,
                t.foreground,
                0.18,
              )})`,
            }}
          />
          <span style={{ fontSize: 17, fontWeight: 700, letterSpacing: "-0.02em" }}>Acme</span>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 20 }}>
          {["Product", "Pricing", "Docs"].map((l) => (
            <span key={l} style={{ fontSize: 13.5, color: t.mutedForeground }}>
              {l}
            </span>
          ))}
          <div
            style={{
              padding: "7px 14px",
              borderRadius: 8,
              backgroundColor: t.foreground,
              color: t.background,
              fontSize: 13,
              fontWeight: 600,
            }}
          >
            Sign in
          </div>
        </div>
      </div>

      <div
        style={{
          flex: 1,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          textAlign: "center",
          padding: "0 40px",
          gap: 18,
        }}
      >
        <div
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: 7,
            padding: "5px 12px",
            borderRadius: 999,
            border: `1px solid ${t.border}`,
            backgroundColor: t.card,
            fontSize: 12.5,
            fontWeight: 600,
            color: t.primary,
            ...rise(12),
          }}
        >
          <span
            style={{
              width: 6,
              height: 6,
              borderRadius: 999,
              backgroundColor: t.primary,
            }}
          />
          Now in public beta
        </div>
        <div
          style={{
            fontSize: 46,
            fontWeight: 700,
            letterSpacing: "-0.03em",
            lineHeight: 1.05,
            maxWidth: 560,
            ...rise(16),
          }}
        >
          Ship demos that sell.
        </div>
        <div
          style={{
            fontSize: 16.5,
            lineHeight: 1.5,
            color: t.mutedForeground,
            maxWidth: 440,
            ...rise(20),
          }}
        >
          Turn your product into a polished launch video in minutes — no editor, no render farm.
        </div>
        <div style={{ display: "flex", gap: 10, marginTop: 6, ...rise(24) }}>
          <div
            style={{
              padding: "11px 20px",
              borderRadius: 10,
              background: `linear-gradient(135deg, ${t.primary}, ${mixOklch(
                t.primary,
                t.foreground,
                0.18,
              )})`,
              color: t.primaryForeground,
              fontSize: 14.5,
              fontWeight: 600,
              boxShadow: `0 10px 24px ${withAlpha(t.primary, 0.32)}`,
            }}
          >
            Get started
          </div>
          <div
            style={{
              padding: "11px 20px",
              borderRadius: 10,
              border: `1px solid ${t.border}`,
              backgroundColor: t.card,
              fontSize: 14.5,
              fontWeight: 600,
            }}
          >
            Watch demo
          </div>
        </div>
      </div>
    </div>
  );
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
  batteryLevel,
  t,
  indicatorColor,
}: {
  frame: number;
  label: string;
  batteryLevel: number;
  t: WarmTheme;
  indicatorColor: string;
}) {
  const state = notchState(frame);
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
            fontSize: 12,
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
  entrance = "rise",
  finale = "none",
  bezelColor,
  screenColor,
  theme,
  mode,
  fontFamily,
  indicatorColor = "#30D158",
  restTilt = REST_TILT,
  radius = LID_RADIUS,
  shadow,
  scale = 0.95,
  showNotch = true,
  notchLabel = "AirPods Connected",
  batteryLevel = 85,
  floatLoop = true,
  floatAmplitude = 4,
  speed = 1,
  className,
}: LaptopFrameProps) {
  const frame = useAuthoredFrame() * speed;
  const t = useWarmTheme(theme, mode);
  const face = resolveFont(fontFamily ?? t.fontFamily) ?? FONT_FAMILY;
  const shell = useWarmTheme(theme, "dark");
  const lid = bezelColor ?? mixOklch(shell.background, shell.foreground, 0.09);
  const glass = screenColor ?? t.card;
  const drop = shadow ?? `0 40px 80px ${withAlpha(shell.background, 0.45)}`;
  const { fps, width, height } = useAuthoredConfig();
  const isRendering = getRemotionEnvironment().isRendering;

  const pose = entrancePose(frame, entrance);
  const tilt = lidTilt(frame, entrance, finale, restTilt);
  const camera = cameraPose(frame, finale, scale, width, height);
  const bob =
    floatLoop && !(finale === "zoom-to-screen" && frame >= PUSH_START)
      ? floatOffset(frame, fps, floatAmplitude, ENTRANCE_FRAMES)
      : 0;
  const notchFade =
    finale === "zoom-to-screen"
      ? interpolate(frame, [PUSH_START, PUSH_START + 10], [1, 0], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
        })
      : 1;

  const screenContent =
    children ??
    (screenSrc ? (
      <ScreenMedia src={screenSrc} frame={frame} />
    ) : (
      <PlaceholderScreen frame={frame} t={t} />
    ));

  return (
    <div
      className={className}
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
                      batteryLevel={batteryLevel}
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

export function LaptopFrame(props: ElementProps & { children?: React.ReactNode }) {
  const { theme } = props;
  if (!isTheme(theme))
    return (
      <LaptopFrameView {...(props as unknown as React.ComponentProps<typeof LaptopFrameView>)} />
    );
  return (
    <WarmUIProvider mode={theme.mode}>
      <LaptopFrameView
        {...(props as unknown as React.ComponentProps<typeof LaptopFrameView>)}
        theme={themeColors(theme)}
        mode={theme.mode}
      />
    </WarmUIProvider>
  );
}
