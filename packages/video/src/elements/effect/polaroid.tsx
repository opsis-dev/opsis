import type React from "react";
import { Handwrite } from "#video/parts/handwrite.tsx";
import { DEFAULT_STEP } from "#video/lib/motion/stop-motion.ts";

const BASE_WIDTH = 652;
const BASE_PAD = 16;
const BASE_MEDIA_WIDTH = 620;
const BASE_MEDIA_HEIGHT = 349;
const BASE_CAPTION_HEIGHT = 62;
const BASE_CAPTION_SIZE = 36;

export type PolaroidGeometry = {
  pad: number;
  mediaWidth: number;
  mediaHeight: number;
  captionHeight: number;
  height: number;
};

export function polaroidGeometry(width: number): PolaroidGeometry {
  const scale = width / BASE_WIDTH;
  const pad = BASE_PAD * scale;
  const mediaWidth = BASE_MEDIA_WIDTH * scale;
  const mediaHeight = BASE_MEDIA_HEIGHT * scale;
  const captionHeight = BASE_CAPTION_HEIGHT * scale;
  return {
    pad,
    mediaWidth,
    mediaHeight,
    captionHeight,
    height: pad + mediaHeight + captionHeight,
  };
}

export interface PolaroidProps {
  children: React.ReactNode;
  caption?: string;
  captionAt?: number;
  width?: number;
  frameColor?: string;
  captionColor?: string;
  captionSize?: number;
  step?: number;
}

export function Polaroid({
  children,
  caption,
  captionAt = 0,
  width = BASE_WIDTH,
  frameColor = "#fdfcf8",
  captionColor = "#26242c",
  captionSize,
  step = DEFAULT_STEP,
}: PolaroidProps) {
  const { pad, mediaWidth, mediaHeight, captionHeight, height } =
    polaroidGeometry(width);
  const resolvedCaptionSize = Math.min(
    captionSize ?? BASE_CAPTION_SIZE * (width / BASE_WIDTH),
    captionHeight * 0.62,
    mediaWidth / Math.max(1, Array.from(caption ?? "").length * 0.52),
  );

  return (
    <div
      style={{
        width,
        height,
        boxSizing: "border-box",
        background: frameColor,
        borderRadius: 4,
        boxShadow:
          "0 8px 20px rgba(38,36,44,0.18), 0 2px 5px rgba(38,36,44,0.12)",
        paddingTop: pad,
      }}
    >
      <div
        style={{
          position: "relative",
          width: mediaWidth,
          height: mediaHeight,
          margin: "0 auto",
          background: "#100f14",
          overflow: "hidden",
        }}
      >
        {children}
      </div>
      <div style={{ position: "relative", height: captionHeight }}>
        {caption ? (
          <Handwrite
            text={caption}
            fontSize={resolvedCaptionSize}
            color={captionColor}
            delay={captionAt}
            perStep={1.4}
            weight={600}
            step={step}
          />
        ) : null}
      </div>
    </div>
  );
}
