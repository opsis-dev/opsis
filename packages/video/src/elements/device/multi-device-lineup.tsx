import type { ReactNode } from "react";
import { interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { AbsoluteFill } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { ScreenFit } from "#video/lib/layout/screen-fit.tsx";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

type MultiDeviceLineupProps = ElementSettings["multi-device-lineup"] & {
  children?: ReactNode;
  delayInFrames?: number;
  exitAtInFrames?: number;
  exitInFrames?: number;
};

const clamp = {
  extrapolateLeft: "clamp",
  extrapolateRight: "clamp",
} as const;

const HARDWARE = {
  phone: { screenW: 148, screenH: 320, bezel: 8, radius: 26 },
  tablet: { screenW: 236, screenH: 316, bezel: 11, radius: 20 },
  laptop: { screenW: 396, screenH: 248, bezel: 9, radius: 12 },
} as const;

type DeviceKind = keyof typeof HARDWARE;

function MultiDeviceLineupView({
  phoneLabel,
  tabletLabel,
  laptopLabel,
  children,
  scale,
  delayInFrames = 4,
  staggerInFrames,
  exitAtInFrames,
  exitInFrames = 16,
  bezelColor,
  screenColor,
  labelColor,
  baseGradient,
  islandColor,
  cameraColor,
  edgeColor,
  shadow,
  gap,
  labelSize,
  labelUppercase,
  fontFamily,
}: MultiDeviceLineupProps) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const exit =
    exitAtInFrames === undefined
      ? 0
      : interpolate(frame, [exitAtInFrames, exitAtInFrames + exitInFrames], [0, 1], {
          easing: EASING.exit,
          ...clamp,
        });

  const devices: { kind: DeviceKind; label: string }[] = [
    { kind: "phone", label: phoneLabel },
    { kind: "tablet", label: tabletLabel },
    { kind: "laptop", label: laptopLabel },
  ];

  const chrome = (kind: DeviceKind) => {
    const spec = HARDWARE[kind];
    return {
      ...spec,
      width: spec.screenW + spec.bezel * 2,
      height: spec.screenH + spec.bezel * 2,
    };
  };

  return (
    <div
      style={{
        display: "flex",
        alignItems: "flex-end",
        justifyContent: "center",
        gap: gap * scale,
        opacity: 1 - exit,
        translate: `0 ${exit * 26 * scale}px`,
      }}
    >
      {devices.map((device, index) => {
        const spec = chrome(device.kind);
        const start = delayInFrames + index * staggerInFrames;
        const rise = spring({
          frame: frame - start,
          fps,
          config: {
            damping: 17,
            stiffness: 150,
            mass: 0.75,
            overshootClamping: true,
          },
        });
        const fade = interpolate(frame, [start, start + 10], [0, 1], {
          easing: EASING.enter,
          ...clamp,
        });
        const riseScale = interpolate(rise, [0, 1], [0.94, 1], {
          output: "perceptual-scale",
          ...clamp,
        });

        return (
          <div
            key={device.kind}
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              gap: 10 * scale,
              opacity: fade,
              translate: `0 ${(1 - rise) * 26 * scale}px`,
              scale: String(riseScale),
            }}
          >
            <div
              style={{
                position: "relative",
                width: spec.width * scale,
                height: spec.height * scale,
                borderRadius: spec.radius * scale,
                background: bezelColor,
                border: `${1 * scale}px solid ${edgeColor}`,
                boxShadow: `inset 0 ${1 * scale}px 0 ${edgeColor}, 0 ${
                  18 * scale
                }px ${44 * scale}px rgba(0,0,0,${shadow})`,
                padding: spec.bezel * scale,
              }}
            >
              <div
                style={{
                  width: spec.screenW * scale,
                  height: spec.screenH * scale,
                  borderRadius: Math.max(4, spec.radius - spec.bezel) * scale,
                  background: screenColor,
                  overflow: "hidden",
                  position: "relative",
                }}
              >
                <div
                  style={{
                    position: "absolute",
                    inset: 0,
                    display: "flex",
                    flexDirection: "column",
                  }}
                >
                  {children}
                </div>
              </div>

              {device.kind === "phone" ? (
                <div
                  style={{
                    position: "absolute",
                    top: (spec.bezel + 5) * scale,
                    left: "50%",
                    transform: "translateX(-50%)",
                    width: 44 * scale,
                    height: 9 * scale,
                    borderRadius: 999,
                    background: islandColor,
                  }}
                />
              ) : null}

              {device.kind === "tablet" ? (
                <div
                  style={{
                    position: "absolute",
                    top: (spec.bezel / 2) * scale,
                    left: "50%",
                    transform: "translateX(-50%)",
                    width: 4 * scale,
                    height: 4 * scale,
                    borderRadius: 999,
                    background: cameraColor,
                  }}
                />
              ) : null}
            </div>

            {device.kind === "laptop" ? (
              <div
                style={{
                  width: (spec.width + 52) * scale,
                  height: 9 * scale,
                  marginTop: -8 * scale,
                  borderRadius: `0 0 ${7 * scale}px ${7 * scale}px`,
                  background: baseGradient,
                  boxShadow: `0 ${8 * scale}px ${18 * scale}px rgba(0,0,0,${(shadow * 0.5) / 0.55})`,
                  position: "relative",
                }}
              >
                <div
                  style={{
                    position: "absolute",
                    top: 0,
                    left: "50%",
                    transform: "translateX(-50%)",
                    width: 74 * scale,
                    height: 3 * scale,
                    borderRadius: `0 0 ${3 * scale}px ${3 * scale}px`,
                    background: "rgba(0,0,0,0.55)",
                  }}
                />
              </div>
            ) : null}

            {device.label ? (
              <div
                style={{
                  color: labelColor,
                  fontSize: labelSize * scale,
                  fontWeight: 600,
                  letterSpacing: "0.08em",
                  textTransform: labelUppercase ? "uppercase" : "none",
                  fontFamily: fontOr(fontFamily, "main"),
                }}
              >
                {device.label}
              </div>
            ) : null}
          </div>
        );
      })}
    </div>
  );
}

export function MultiDeviceLineup({ children, ...settings }: MultiDeviceLineupProps) {
  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <MultiDeviceLineupView {...settings}>
        {children ? <ScreenFit>{children}</ScreenFit> : undefined}
      </MultiDeviceLineupView>
    </AbsoluteFill>
  );
}
