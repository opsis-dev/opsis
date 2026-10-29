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
import { resolveFont, type WarmTheme, useWarmTheme } from "#video/lib/warm-ui/index.ts";
import { WarmUIProvider } from "#video/lib/warm-ui/theme.ts";

export type PhoneFrameVariant = "flat" | "tilt" | "showcase";
export type PhoneFrameEntrance = "rise" | "rotate-in" | "float";

type PhoneFrameProps = Omit<ElementSettings["phone-frame"], "theme"> & {
  children?: ReactNode;
  theme: Partial<WarmTheme>;
  mode: "light" | "dark";
};

const FONT_FAMILY = 'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif';

export const DEVICE_WIDTH = 320;
export const DEVICE_HEIGHT = 656;

export const BEZEL_WIDTH = 12;

export const ENTRANCE_FRAMES = 30;

export interface EntrancePose {
  opacity: number;
  translateY: number;
  rotateDeg: number;
  scale: number;
}

export function entrancePose(frame: number, entrance: PhoneFrameEntrance): EntrancePose {
  const clamp = {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  } as const;
  const easeOut = Easing.out(Easing.cubic);

  if (entrance === "rotate-in") {
    return {
      opacity: interpolate(frame, [0, 12], [0, 1], clamp),
      translateY: interpolate(frame, [0, 28], [60, 0], {
        ...clamp,
        easing: easeOut,
      }),
      rotateDeg: interpolate(frame, [0, 28], [-8, 0], {
        ...clamp,
        easing: easeOut,
      }),
      scale: interpolate(frame, [0, 28], [0.94, 1], {
        ...clamp,
        easing: easeOut,
      }),
    };
  }

  if (entrance === "float") {
    return {
      opacity: interpolate(frame, [0, 16], [0, 1], clamp),
      translateY: 0,
      rotateDeg: 0,
      scale: interpolate(frame, [0, 24], [0.96, 1], {
        ...clamp,
        easing: easeOut,
      }),
    };
  }

  return {
    opacity: interpolate(frame, [0, 12], [0, 1], clamp),
    translateY: interpolate(frame, [0, 26], [70, 0], {
      ...clamp,
      easing: easeOut,
    }),
    rotateDeg: 0,
    scale: 1,
  };
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

export const SIDE_BUTTONS: Array<{
  side: "left" | "right";
  top: number;
  length: number;
}> = [
  { side: "left", top: 118, length: 26 },
  { side: "left", top: 166, length: 46 },
  { side: "left", top: 222, length: 46 },
  { side: "right", top: 176, length: 68 },
];

export interface ShowcasePose {
  rotateX: number;
  rotateY: number;
  rotateZ: number;
  zoom: number;
  x: number;
  y: number;
}

const SHOWCASE_STOPS = [0, 0.1, 0.42, 0.62, 0.9, 1];
const SHOWCASE_KEYS: Record<keyof ShowcasePose, number[]> = {
  rotateX: [52, 36, 26, 12, 1.5, 0],
  rotateY: [28, 20, 10, 6, 1, 0],
  rotateZ: [30, 14, 2, 0, 0, 0],
  zoom: [2.6, 1.35, 2.75, 2.3, 1.04, 1],
  x: [60, 30, 0, -10, 0, 0],
  y: [-195, -40, 430, 380, 15, 0],
};

export function showcasePose(frame: number, durationInFrames: number): ShowcasePose {
  const p = Math.max(0, Math.min(1, frame / Math.max(1, durationInFrames - 1)));
  const easing = Easing.inOut(Easing.cubic);
  const at = (values: number[]) => interpolate(p, SHOWCASE_STOPS, values, { easing });
  return {
    rotateX: at(SHOWCASE_KEYS.rotateX),
    rotateY: at(SHOWCASE_KEYS.rotateY),
    rotateZ: at(SHOWCASE_KEYS.rotateZ),
    zoom: at(SHOWCASE_KEYS.zoom),
    x: at(SHOWCASE_KEYS.x),
    y: at(SHOWCASE_KEYS.y),
  };
}

const CLAMP = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;

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
  const t = interpolate(frame, [4, 22], [0, 1], CLAMP);
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

function PhoneFrameView({
  children,
  screenSrc,
  variant,
  entrance,
  bezelColor,
  screenColor,
  theme,
  mode,
  fontFamily,
  radius,
  screenRadius,
  shadow,
  scale,
  showDynamicIsland,
  floatLoop,
  floatAmplitude,
  tiltAngle,
  speed,
  floatPeriod,
}: PhoneFrameProps) {
  const frame = useAuthoredFrame() * speed;
  const { fps, durationInFrames: clipFrames } = useAuthoredConfig();
  const durationInFrames = clipFrames * speed;
  const t = useWarmTheme(theme, mode);
  const face = resolveFont(fontFamily || t.fontFamily) ?? FONT_FAMILY;
  const island = useWarmTheme(theme, "dark");
  const body = bezelColor || t.foreground;
  const glass = screenColor || t.card;
  const drop = shadow;

  const bodyRadius = radius;
  const innerRadius = screenRadius;

  const pose = entrancePose(frame, entrance);
  const floatStart = entrance === "float" ? 0 : ENTRANCE_FRAMES;
  const bob = floatLoop ? floatOffset(frame, fps, floatAmplitude, floatStart, floatPeriod) : 0;

  const crane = variant === "showcase" ? showcasePose(frame, durationInFrames) : null;

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
          opacity: pose.opacity,
          translate: `0px ${pose.translateY + bob}px`,
          rotate: `${pose.rotateDeg}deg`,
          scale: `${pose.scale * scale}`,
        }}
      >
        <div
          style={{
            transform: crane
              ? `translate(${crane.x}px, ${crane.y}px) scale(${crane.zoom})`
              : undefined,
          }}
        >
          <div
            style={{
              transform: `perspective(1400px) rotateX(${crane ? crane.rotateX : 0}deg) rotateY(${
                crane ? crane.rotateY : variant === "tilt" ? tiltAngle : 0
              }deg) rotateZ(${crane ? crane.rotateZ : 0}deg)`,
            }}
          >
            <div
              style={{
                position: "relative",
                width: DEVICE_WIDTH,
                height: DEVICE_HEIGHT,
                borderRadius: bodyRadius,
                backgroundColor: body,
                padding: BEZEL_WIDTH,
                ...(shadow === "" ? {} : { boxShadow: drop }),
              }}
            >
              {SIDE_BUTTONS.map((button) => (
                <div
                  key={`${button.side}-${button.top}`}
                  style={{
                    position: "absolute",
                    top: button.top,
                    [button.side]: -3,
                    width: 3,
                    height: button.length,
                    borderRadius: 2,
                    backgroundColor: body,
                  }}
                />
              ))}

              <div
                style={{
                  position: "relative",
                  width: "100%",
                  height: "100%",
                  borderRadius: innerRadius,
                  backgroundColor: glass,
                  overflow: "hidden",
                }}
              >
                {children ?? (screenSrc ? <ScreenMedia src={screenSrc} frame={frame} /> : null)}

                {showDynamicIsland && (
                  <div
                    style={{
                      position: "absolute",
                      top: 10,
                      left: "50%",
                      translate: "-50% 0px",
                      width: 90,
                      height: 26,
                      borderRadius: 999,
                      backgroundColor: island.background,
                      zIndex: 10,
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "flex-end",
                      paddingRight: 9,
                    }}
                  >
                    <div
                      style={{
                        width: 9,
                        height: 9,
                        borderRadius: 999,
                        backgroundColor: island.card,
                        border: `1px solid ${island.border}`,
                      }}
                    />
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export function PhoneFrame({
  theme,
  ...settings
}: ElementSettings["phone-frame"] & { children?: ReactNode }) {
  return (
    <WarmUIProvider mode={theme.mode}>
      <PhoneFrameView {...settings} theme={themeColors(theme)} mode={theme.mode} />
    </WarmUIProvider>
  );
}
