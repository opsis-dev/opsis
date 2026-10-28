import { useId } from "react";
import type { CSSProperties } from "react";
import type React from "react";
import { AbsoluteFill, Img, OffthreadVideo, useCurrentFrame, useVideoConfig } from "remotion";
import { type ElementProps } from "#video/common/spec.ts";
import { scaleFont } from "#video/lib/layout/layout.ts";
import { useEnterExit, type EnterExitOptions } from "#video/lib/motion/motion-primitive.ts";

export type TextMaskMedia = "video" | "image" | "gradient";

export type TextMaskVideoProps = EnterExitOptions & {
  text: string;
  src?: string;
  media?: TextMaskMedia;
  gradient?: string;
  width?: number;
  height?: number;
  drift?: number;
  zoom?: number;
  reveal?: "left" | "right" | "up" | "down" | "none";
  fontSize?: number;
  fontFamily?: string;
  fontWeight?: number | string;
  letterSpacing?: number;
  lineHeight?: number;
  startFrom?: number;
  mediaFilter?: string;
  style?: CSSProperties;
  className?: string;
};

const GLYPH_WIDTH = 0.72;

const REVEAL_INSET: Record<
  Exclude<TextMaskVideoProps["reveal"] & string, "none">,
  (hidden: number) => string
> = {
  left: (hidden) => `inset(0 ${hidden}% 0 0)`,
  right: (hidden) => `inset(0 0 0 ${hidden}%)`,
  up: (hidden) => `inset(0 0 ${hidden}% 0)`,
  down: (hidden) => `inset(${hidden}% 0 0 0)`,
};

const TextMaskVideoView: React.FC<TextMaskVideoProps> = ({
  text,
  src,
  media,
  gradient = "linear-gradient(115deg, #e8b86d 0%, #f472b6 42%, #2dd4bf 100%)",
  width: widthProp,
  height: heightProp,
  drift = 26,
  zoom = 0.12,
  reveal = "left",
  fontSize: fontSizeProp,
  fontFamily,
  fontWeight = 800,
  letterSpacing = 0,
  lineHeight = 0.98,
  startFrom,
  mediaFilter,
  style,
  className,
  ...motionProps
}) => {
  const frame = useCurrentFrame();
  const { width: videoWidth, height: videoHeight, fps } = useVideoConfig();
  const clipId = `mask-${useId().replace(/[^a-zA-Z0-9]/g, "")}`;

  const kind: TextMaskMedia =
    !src || media === "gradient" ? "gradient" : media === "image" ? "image" : "video";
  const lines = text.split(/\r?\n/);
  const longest = Math.max(1, ...lines.map((line) => Array.from(line).length));
  const fittedSize = Math.min(
    scaleFont(150, videoWidth),
    (videoWidth * 0.9) / longest / GLYPH_WIDTH - letterSpacing / GLYPH_WIDTH,
    (videoHeight * 0.85) / (lines.length * lineHeight * 1.25),
  );
  const fontSize = fontSizeProp ?? Math.max(1, fittedSize);
  const width =
    widthProp ??
    Math.round(Math.max(fontSize * 6.2, longest * (fontSize * GLYPH_WIDTH + letterSpacing)));
  const height = heightProp ?? Math.round(fontSize * lineHeight * lines.length * 1.25);

  const { motion, opacity, exitProgress } = useEnterExit(motionProps);
  const revealed = reveal === "none" ? 1 : motion * (1 - exitProgress);
  const hidden = Math.max(0, Math.min(100, (1 - revealed) * 100));

  const seconds = frame / fps;
  const pan = Math.sin(seconds * 0.62) * drift;
  const scale = 1 + zoom * (0.5 + 0.5 * Math.sin(seconds * 0.44 + 1.1));

  const cx = width / 2;
  const blockHeight = (lines.length - 1) * fontSize * lineHeight;
  const firstY = height / 2 - blockHeight / 2;

  return (
    <div
      className={className}
      style={{
        position: "relative",
        width,
        height,
        opacity,
        ...style,
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
                {...(fontFamily ? { fontFamily } : {})}
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
              ...(mediaFilter ? { filter: mediaFilter } : null),
            }}
          >
            {kind === "video" && src ? (
              <OffthreadVideo
                src={src}
                startFrom={startFrom}
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
  );
};

export function TextMaskVideo(props: ElementProps & { children?: React.ReactNode }) {
  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <TextMaskVideoView
        {...(props as unknown as React.ComponentProps<typeof TextMaskVideoView>)}
      />
    </AbsoluteFill>
  );
}
