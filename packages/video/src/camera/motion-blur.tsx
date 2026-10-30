import type { ReactNode } from "react";
import { AbsoluteFill, Freeze, useCurrentFrame } from "remotion";
import { sampleOpacity, shutterOffsets } from "#video/camera/lib/shutter.ts";

export function MotionBlur({
  shutterAngle,
  samples,
  children,
}: {
  shutterAngle: number;
  samples: number;
  children: (muted: boolean) => ReactNode;
}) {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill>
      {shutterOffsets(shutterAngle, samples).map((offset, i) => (
        <AbsoluteFill key={offset} style={{ opacity: sampleOpacity(i) }}>
          <Freeze frame={Math.max(0, frame - offset)}>{children(i > 0)}</Freeze>
        </AbsoluteFill>
      ))}
    </AbsoluteFill>
  );
}
