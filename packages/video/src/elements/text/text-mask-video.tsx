import { useId } from "react";
import { AbsoluteFill, Img, OffthreadVideo, useCurrentFrame, useVideoConfig } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import type { ElementSettings, MaskReveal } from "#video/generated/spec.ts";
import { scaleFont } from "#video/lib/layout/layout.ts";
import { useEnterExit } from "#video/lib/motion/motion-primitive.ts";

export type TextMaskMedia = "video" | "image" | "gradient";

const GLYPH_WIDTH = 0.72;

const REVEAL_INSET: Record<Exclude<MaskReveal, "none">, (hidden: number) => string> = {
  left: (hidden) => `inset(0 ${hidden}% 0 0)`,
  right: (hidden) => `inset(0 0 0 ${hidden}%)`,
  up: (hidden) => `inset(0 0 ${hidden}% 0)`,
  down: (hidden) => `inset(${hidden}% 0 0 0)`,
};

export function TextMaskVideo({
  text,
  src,
  media,
  gradient,
  drift,
  zoom,
  reveal,
  fontSize: fontSizeProp,
  fontFamily,
  fontWeight,
  letterSpacing,
  lineHeight,
  durationInFrames,
  driftSpeed,
  fitMax,
  startFrom,
}: ElementSettings["text-mask-video"] & { media: TextMaskMedia }) {
  const frame = useCurrentFrame();
  const { width: videoWidth, height: videoHeight, fps } = useVideoConfig();
  const clipId = `mask-${useId().replace(/[^a-zA-Z0-9]/g, "")}`;

  const kind: TextMaskMedia =
    !src || media === "gradient" ? "gradient" : media === "image" ? "image" : "video";
  const lines = text.split(/\r?\n/);
  const longest = Math.max(1, ...lines.map((line) => Array.from(line).length));
  const fittedSize = Math.min(
    scaleFont(fitMax, videoWidth),
    (videoWidth * 0.9) / longest / GLYPH_WIDTH - letterSpacing / GLYPH_WIDTH,
    (videoHeight * 0.85) / (lines.length * lineHeight * 1.25),
  );
  const fontSize = fontSizeProp ?? Math.max(1, fittedSize);
  const width = Math.round(
    Math.max(fontSize * 6.2, longest * (fontSize * GLYPH_WIDTH + letterSpacing)),
  );
  const height = Math.round(fontSize * lineHeight * lines.length * 1.25);

  const { motion, opacity, exitProgress } = useEnterExit({ durationInFrames });
  const revealed = reveal === "none" ? 1 : motion * (1 - exitProgress);
  const hidden = Math.max(0, Math.min(100, (1 - revealed) * 100));

  const seconds = (frame / fps) * driftSpeed;
  const pan = Math.sin(seconds * 0.62) * drift;
  const scale = 1 + zoom * (0.5 + 0.5 * Math.sin(seconds * 0.44 + 1.1));

  const cx = width / 2;
  const blockHeight = (lines.length - 1) * fontSize * lineHeight;
  const firstY = height / 2 - blockHeight / 2;

  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <div
        style={{
          position: "relative",
          width,
          height,
          opacity,
        }}
      >
        <svg width={0} height={0} style={{ position: "absolute" }} aria-hidden>
          <defs>
            <clipPath id={clipId} clipPathUnits="userSpaceOnUse">
              {lines.map((line, index) => (
                <text
                  key={`${line}-${index}`}
                  x={cx}
                  y={firstY + index * fontSize * lineHeight}
                  textAnchor="middle"
                  dominantBaseline="central"
                  fontSize={fontSize}
                  fontWeight={fontWeight}
                  letterSpacing={letterSpacing}
                  fontFamily={fontOr(fontFamily, "heading")}
                >
                  {line}
                </text>
              ))}
            </clipPath>
          </defs>
        </svg>

        <div
          style={{
            position: "absolute",
            inset: 0,
            clipPath: `url(#${clipId})`,
            WebkitClipPath: `url(#${clipId})`,
          }}
        >
          <div
            style={{
              position: "absolute",
              inset: 0,
              ...(reveal === "none"
                ? null
                : {
                    clipPath: REVEAL_INSET[reveal](hidden),
                    WebkitClipPath: REVEAL_INSET[reveal](hidden),
                  }),
            }}
          >
            <div
              style={{
                position: "absolute",
                inset: "-8%",
                transform: `translateX(${(-pan).toFixed(2)}px) scale(${scale.toFixed(4)})`,
                transformOrigin: "center center",
              }}
            >
              {kind === "video" && src ? (
                <OffthreadVideo
                  src={src}
                  startFrom={Math.round(startFrom * fps)}
                  muted
                  pauseWhenBuffering
                  style={{
                    width: "100%",
                    height: "100%",
                    objectFit: "cover",
                  }}
                />
              ) : kind === "image" && src ? (
                <Img src={src} style={{ width: "100%", height: "100%", objectFit: "cover" }} />
              ) : (
                <div style={{ width: "100%", height: "100%", background: gradient }} />
              )}
            </div>
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
}
