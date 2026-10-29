import type React from "react";
import { AbsoluteFill } from "remotion";
import type { ElementSettings, PhotoShape } from "#video/generated/spec.ts";
import { HAND_FAMILY, Handwrite } from "#video/parts/handwrite.tsx";

const BASE_WIDTH = 652;
const BASE_PAD = 16;
const BASE_MEDIA_WIDTH = 620;
const BASE_CAPTION_SIZE = 36;

const PHOTO_HEIGHT: Record<PhotoShape, number> = {
  "16:9": 349,
  "4:3": 465,
  "1:1": 620,
  "3:4": 827,
};

export type PolaroidGeometry = {
  pad: number;
  mediaWidth: number;
  mediaHeight: number;
  captionHeight: number;
  height: number;
};

export function polaroidGeometry(
  width: number,
  shape: PhotoShape,
  captionStrip: number,
): PolaroidGeometry {
  const scale = width / BASE_WIDTH;
  const pad = BASE_PAD * scale;
  const mediaWidth = BASE_MEDIA_WIDTH * scale;
  const mediaHeight = PHOTO_HEIGHT[shape] * scale;
  const captionHeight = captionStrip * scale;
  return {
    pad,
    mediaWidth,
    mediaHeight,
    captionHeight,
    height: pad + mediaHeight + captionHeight,
  };
}

export function Polaroid({
  children,
  caption,
  captionAt,
  width,
  frameColor,
  captionColor,
  captionSize,
  step,
  photoShape,
  captionStrip,
  cornerRadius,
  captionWeight,
  writeSpeed,
  captionFont,
  photoColor,
}: ElementSettings["polaroid"] & { children?: React.ReactNode }) {
  const { pad, mediaWidth, mediaHeight, captionHeight, height } = polaroidGeometry(
    width,
    photoShape,
    captionStrip,
  );
  const resolvedCaptionSize = Math.min(
    captionSize ?? BASE_CAPTION_SIZE * (width / BASE_WIDTH),
    captionHeight * 0.62,
    mediaWidth / Math.max(1, Array.from(caption).length * 0.52),
  );

  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <div
        style={{
          width,
          height,
          boxSizing: "border-box",
          background: frameColor,
          borderRadius: cornerRadius,
          boxShadow: "0 8px 20px rgba(38,36,44,0.18), 0 2px 5px rgba(38,36,44,0.12)",
          paddingTop: pad,
        }}
      >
        <div
          style={{
            position: "relative",
            width: mediaWidth,
            height: mediaHeight,
            margin: "0 auto",
            background: photoColor,
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
              perStep={writeSpeed}
              weight={captionWeight}
              fontFamily={captionFont || HAND_FAMILY}
              step={step}
            />
          ) : null}
        </div>
      </div>
    </AbsoluteFill>
  );
}
