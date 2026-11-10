import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { spring } from "@opsis/runtime/spring.ts";
import type { ReactNode } from "react";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { getSafeAreaPadding, scaleFont } from "#video/lib/layout/layout.ts";
import { ScreenFit } from "#video/lib/layout/screen-fit.tsx";
import { EASING, STAGGER } from "#video/lib/motion/motion-tokens.ts";
import { springSmooth } from "#video/lib/motion/springs.ts";

type Look = ElementSettings["device-mockup-zoom"];

function clampProgress(frame: number, input: [number, number], output: [number, number]) {
  return interpolate(frame, input, output, { ...CLAMPED, easing: EASING.enter });
}

type DeviceShellProps = {
  children: ReactNode;
  look: Look;
  progress: number;
  width: number;
};

function DeviceShell({ children, look, progress, width }: DeviceShellProps) {
  const { device, accentColor, shadow } = look;
  const isPhone = device === "phone";
  const isLaptop = device === "laptop";
  const isBrowser = device === "browser";

  const radius = isPhone ? scaleFont(56, width) : scaleFont(18, width);
  const chromePad = isPhone
    ? scaleFont(9, width)
    : isBrowser
      ? scaleFont(10, width)
      : scaleFont(7, width);
  const screenRadius = isPhone
    ? radius - chromePad * 0.7
    : isBrowser
      ? scaleFont(12, width)
      : scaleFont(3, width);
  const buttonWidth = scaleFont(3, width);
  const buttonColor = look.buttonColor;

  return (
    <div
      style={{
        width: "100%",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
      }}
    >
      <div
        style={{
          position: "relative",
          width: isLaptop ? "92%" : "100%",
          aspectRatio: isPhone ? "0.4615" : "1.6",
          borderRadius: radius,
          background: `linear-gradient(145deg, ${look.deviceEdgeColor}, ${look.deviceColor})`,
          padding: chromePad,
          boxShadow: `0 ${scaleFont(34, width)}px ${scaleFont(90, width)}px rgba(0,0,0,${shadow}), 0 0 ${scaleFont(96, width) * progress}px color-mix(in srgb, ${accentColor} ${look.glow * 100}%, transparent), inset 0 0 0 1px rgba(255,255,255,0.1)`,
          display: "flex",
          flexDirection: "column",
          gap: isBrowser ? scaleFont(8, width) : 0,
        }}
      >
        {isPhone ? (
          <>
            <div
              style={{
                position: "absolute",
                left: -buttonWidth,
                top: "16%",
                width: buttonWidth,
                height: scaleFont(9, width),
                borderRadius: `${buttonWidth}px 0 0 ${buttonWidth}px`,
                background: buttonColor,
              }}
            />
            <div
              style={{
                position: "absolute",
                left: -buttonWidth,
                top: "26%",
                width: buttonWidth,
                height: scaleFont(16, width),
                borderRadius: `${buttonWidth}px 0 0 ${buttonWidth}px`,
                background: buttonColor,
              }}
            />
            <div
              style={{
                position: "absolute",
                left: -buttonWidth,
                top: "37%",
                width: buttonWidth,
                height: scaleFont(16, width),
                borderRadius: `${buttonWidth}px 0 0 ${buttonWidth}px`,
                background: buttonColor,
              }}
            />
            <div
              style={{
                position: "absolute",
                right: -buttonWidth,
                top: "22%",
                width: buttonWidth,
                height: scaleFont(22, width),
                borderRadius: `0 ${buttonWidth}px ${buttonWidth}px 0`,
                background: buttonColor,
              }}
            />
          </>
        ) : isBrowser ? (
          <div
            style={{
              height: scaleFont(32, width),
              borderRadius: scaleFont(14, width),
              background: "rgba(255,255,255,0.06)",
              display: "flex",
              alignItems: "center",
              gap: scaleFont(8, width),
              padding: `0 ${scaleFont(12, width)}px`,
            }}
          >
            {[look.thirdAccent, accentColor, look.secondAccent].map((color, index) => (
              <span
                key={index}
                style={{
                  width: scaleFont(10, width),
                  height: scaleFont(10, width),
                  borderRadius: 99,
                  background: color,
                }}
              />
            ))}
            <div
              style={{
                flex: 1,
                height: scaleFont(16, width),
                borderRadius: 999,
                background: "rgba(255,255,255,0.08)",
                marginLeft: scaleFont(8, width),
              }}
            />
          </div>
        ) : (
          <div
            style={{
              height: scaleFont(15, width),
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <div
              style={{
                width: scaleFont(5, width),
                height: scaleFont(5, width),
                borderRadius: "50%",
                background: look.cameraColor,
                boxShadow: "inset 0 0 0 1px rgba(255,255,255,0.18)",
              }}
            />
          </div>
        )}

        <div
          style={{
            flex: 1,
            minHeight: 0,
            position: "relative",
            borderRadius: screenRadius,
            overflow: "hidden",
            background: look.bezelColor,
            border: "1px solid rgba(255,255,255,0.08)",
          }}
        >
          {children}
          {isPhone ? (
            <div
              style={{
                position: "absolute",
                top: "2.6%",
                left: "50%",
                transform: "translateX(-50%)",
                width: "34%",
                height: scaleFont(9, width),
                borderRadius: 999,
                background: look.islandColor,
                display: "flex",
                alignItems: "center",
                justifyContent: "flex-end",
                padding: `0 ${scaleFont(6, width)}px`,
                pointerEvents: "none",
              }}
            >
              <div
                style={{
                  width: scaleFont(4, width),
                  height: scaleFont(4, width),
                  borderRadius: "50%",
                  background: look.lensColor,
                  boxShadow: "inset 0 0 0 1px rgba(255,255,255,0.06)",
                }}
              />
            </div>
          ) : null}
        </div>
      </div>

      {isLaptop ? (
        <>
          <div
            style={{
              width: "104%",
              height: 2,
              background:
                "linear-gradient(90deg, transparent, rgba(0,0,0,0.55) 12%, rgba(0,0,0,0.55) 88%, transparent)",
            }}
          />
          <div
            style={{
              width: "104%",
              height: scaleFont(30, width),
              borderRadius: `0 0 ${scaleFont(16, width)}px ${scaleFont(16, width)}px`,
              background: look.baseGradient,
              boxShadow: `0 ${scaleFont(24, width)}px ${scaleFont(50, width)}px rgba(0,0,0,${(shadow * 0.38) / 0.42}), inset 0 1px 0 rgba(255,255,255,0.16)`,
              display: "flex",
              justifyContent: "center",
            }}
          >
            <div
              style={{
                marginTop: scaleFont(6, width),
                width: "16%",
                height: scaleFont(4, width),
                borderRadius: 3,
                background: "rgba(255,255,255,0.1)",
              }}
            />
          </div>
          <div
            style={{
              width: "82%",
              height: scaleFont(18, width),
              borderRadius: "50%",
              background: `rgba(0,0,0,${(shadow * 0.28) / 0.42})`,
              filter: `blur(${scaleFont(12, width)}px)`,
              marginTop: scaleFont(10, width),
            }}
          />
        </>
      ) : null}
    </div>
  );
}

function DeviceMockupZoomView({ children, look }: { children?: ReactNode; look: Look }) {
  const { device, backgroundColor, accentColor, pullBackFrom, scale } = look;
  const frame = useFrame();
  const { width, height, fps } = useFilm();
  const safe = getSafeAreaPadding({ width, height });
  const portrait = height > width;

  const camera = spring({
    fps,
    frame,
    config: springSmooth,
    delay: 8,
    durationInFrames: 58,
  });
  const deviceProgress = clampProgress(frame, [STAGGER.normal, 64], [0, 1]);
  const accentProgress = clampProgress(frame, [20, 72], [0, 1]);
  const cameraScale = interpolate(camera, [0, 1], [pullBackFrom, 0.95], {
    ...CLAMPED,
    output: "perceptual-scale",
  });
  const cameraY = interpolate(camera, [0, 1], [scaleFont(20, width), 0], CLAMPED);
  const mainShare =
    device === "phone" ? (portrait ? 58 : 32) : portrait ? 96 : device === "laptop" ? 86 : 82;
  const mainMaxWidth = `${mainShare * scale}%`;

  return (
    <AbsoluteFill style={{ background: backgroundColor, overflow: "hidden" }}>
      <AbsoluteFill
        style={{
          background: `radial-gradient(circle at 72% 42%, color-mix(in srgb, ${accentColor} 19%, transparent) 0, transparent 32%), radial-gradient(circle at 16% 82%, color-mix(in srgb, ${look.secondAccent} 12.5%, transparent) 0, transparent 26%), linear-gradient(135deg, ${backgroundColor} 0%, color-mix(in srgb, ${backgroundColor} 93%, ${look.backgroundTint}) 54%, color-mix(in srgb, ${backgroundColor} 97%, black) 100%)`,
        }}
      />
      {look.showGrid ? (
        <AbsoluteFill
          style={{
            opacity: 0.16,
            backgroundImage:
              "linear-gradient(rgba(255,255,255,0.2) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.2) 1px, transparent 1px)",
            backgroundSize: `${scaleFont(56, width)}px ${scaleFont(56, width)}px`,
            maskImage: "radial-gradient(circle at 68% 44%, black, transparent 72%)",
          }}
        />
      ) : null}

      <div
        style={{
          position: "relative",
          zIndex: 1,
          width,
          height,
          paddingLeft: safe.paddingLeft,
          paddingRight: safe.paddingRight,
          paddingTop: safe.paddingTop,
          paddingBottom: safe.paddingBottom,
          display: "grid",
          alignItems: "center",
        }}
      >
        <div
          style={{
            width: "100%",
            maxWidth: mainMaxWidth,
            justifySelf: "center",
            opacity: deviceProgress,
            scale: cameraScale,
            translate: `0px ${cameraY}px`,
          }}
        >
          <DeviceShell look={look} progress={accentProgress} width={width}>
            {children}
          </DeviceShell>
        </div>
      </div>
    </AbsoluteFill>
  );
}

export function DeviceMockupZoom({ children, ...look }: Look & { children?: ReactNode }) {
  return (
    <DeviceMockupZoomView look={look}>
      {children ? <ScreenFit>{children}</ScreenFit> : undefined}
    </DeviceMockupZoomView>
  );
}
