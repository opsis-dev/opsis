import { Easing, interpolate } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export const gradientScaleCutTextLength = 36;

const SETTLE_FRAMES = 8;
const GIANT_Y = -28;
const GIANT_X_CORRECTION = -48;
const COMPACT_Y = 5;
const COMPACT_START_X_CORRECTION = 28;
const clamp = {
  extrapolateLeft: "clamp" as const,
  extrapolateRight: "clamp" as const,
};

interface TextLayerProps {
  text: string;
  fontFamily: string;
  fontSize: number;
  fontWeight: string;
  gradientStart: string;
  gradientEnd: string;
  ghostColor: string;
  maskImage: string;
  gradientSpread: number;
  letterSpacing: number;
}

function TextLayer({
  text,
  fontFamily,
  fontSize,
  fontWeight,
  gradientStart,
  gradientEnd,
  ghostColor,
  maskImage,
  gradientSpread,
  letterSpacing,
}: TextLayerProps) {
  const textStyle = {
    display: "block",
    fontFamily,
    fontSize,
    fontWeight,
    letterSpacing: `${letterSpacing}em`,
    lineHeight: 1.12,
    whiteSpace: "nowrap" as const,
  };

  return (
    <div style={{ position: "relative" }}>
      <span style={{ ...textStyle, color: ghostColor }}>{text}</span>
      <span
        style={{
          ...textStyle,
          position: "absolute",
          inset: 0,
          color: "transparent",
          backgroundImage: `linear-gradient(90deg, ${gradientStart} 0%, ${gradientStart} 4%, ${gradientEnd} ${gradientSpread}%, ${gradientEnd} 100%)`,
          backgroundClip: "text",
          maskImage,
          maskRepeat: "no-repeat",
          opacity: 1,
          WebkitBackgroundClip: "text",
          WebkitMaskImage: maskImage,
          WebkitMaskRepeat: "no-repeat",
        }}
      >
        {text}
      </span>
    </div>
  );
}

export function GradientScaleCutText({
  text,
  fontFamily,
  giantFontSize,
  compactFontSize,
  fontWeight,
  gradientStart,
  gradientEnd,
  ghostColor,
  backgroundColor,
  anchorOffsetX,
  giantTravel,
  settleTravel,
  cutFrame,
  revealSoftness,
  compactBlur,
  speed,
  revealEnd,
  compactWidth,
  gradientSpread,
  letterSpacing,
}: ElementSettings["gradient-scale-cut-text"]) {
  const face = fontOr(fontFamily, "main");
  const frame = useAuthoredFrame() * Math.max(speed, 0);
  const safeCutFrame = Math.max(cutFrame, 1);
  const safeGiantFontSize = Math.max(giantFontSize, 1);
  const safeCompactFontSize = Math.max(compactFontSize, 1);
  const safeSoftness = Math.max(revealSoftness, 0);
  const safeBlur = Math.max(compactBlur, 0);
  const revealProgress = interpolate(frame, [0, revealEnd], [0.02, 1], {
    ...clamp,
    easing: Easing.bezier(0.22, 0.74, 0.24, 1),
  });
  const revealEdge = revealProgress * (100 + safeSoftness / 2);
  const revealSolid = Math.max(0, revealEdge - safeSoftness / 2);
  const revealFade = revealEdge + safeSoftness / 2;
  const maskImage = `linear-gradient(90deg, #000 0%, #000 ${revealSolid}%, transparent ${revealFade}%, transparent ${revealFade}%)`;
  const giantProgress = interpolate(frame, [0, Math.max(safeCutFrame - 1, 1)], [0, 1], {
    ...clamp,
    easing: Easing.out(Easing.cubic),
  });
  const settleProgress = interpolate(frame, [safeCutFrame, safeCutFrame + SETTLE_FRAMES], [0, 1], {
    ...clamp,
    easing: Easing.out(Easing.cubic),
  });
  const moveProgress = interpolate(
    frame,
    [safeCutFrame + 1, safeCutFrame + 2, safeCutFrame + 4, safeCutFrame + SETTLE_FRAMES],
    [0, 0.58, 0.875, 1],
    clamp,
  );
  const showGiant = frame < safeCutFrame;
  const compactX =
    interpolate(moveProgress, [0, 1], [settleTravel, 0], clamp) +
    COMPACT_START_X_CORRECTION * (1 - moveProgress);
  const compactBlurValue = safeBlur * (1 - settleProgress);

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        overflow: "hidden",
        backgroundColor,
      }}
    >
      {showGiant ? (
        <div
          style={{
            position: "absolute",
            top: `calc(50% + ${GIANT_Y}px)`,
            left: `calc(50% + ${anchorOffsetX + GIANT_X_CORRECTION - giantTravel * giantProgress}px)`,
            opacity: 1,
            transform: "translate3d(0, -50%, 0)",
          }}
        >
          <TextLayer
            fontFamily={face}
            text={text}
            fontSize={safeGiantFontSize}
            fontWeight={fontWeight}
            gradientStart={gradientStart}
            gradientEnd={gradientEnd}
            ghostColor={ghostColor}
            maskImage={maskImage}
            gradientSpread={gradientSpread}
            letterSpacing={letterSpacing}
          />
        </div>
      ) : (
        <div
          style={{
            position: "absolute",
            top: `calc(50% + ${COMPACT_Y}px)`,
            left: `calc(50% + ${anchorOffsetX + compactX}px)`,
            filter: `blur(${compactBlurValue}px)`,
            opacity: 1,
            transform: `translate3d(-50%, -50%, 0) scaleX(${compactWidth / 100})`,
            transformOrigin: "center",
          }}
        >
          <TextLayer
            fontFamily={face}
            text={text}
            fontSize={safeCompactFontSize}
            fontWeight={fontWeight}
            gradientStart={gradientStart}
            gradientEnd={gradientEnd}
            ghostColor={ghostColor}
            maskImage={maskImage}
            gradientSpread={gradientSpread}
            letterSpacing={letterSpacing}
          />
        </div>
      )}
    </div>
  );
}
