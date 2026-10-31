import { Freeze, useFrame } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import type { ReactNode } from "react";
import { sampleOpacity, shutterOffsets } from "#video/camera/lib/shutter.ts";

export function MotionBlur({
  shutterAngle,
  samples,
  children,
}: {
  shutterAngle: number;
  samples: number;
  children: ReactNode;
}) {
  const frame = useFrame();
  return (
    <AbsoluteFill>
      {shutterOffsets(shutterAngle, samples).map((offset, i) => (
        <AbsoluteFill key={offset} style={{ opacity: sampleOpacity(i) }}>
          <Freeze frame={Math.max(0, frame - offset)}>{children}</Freeze>
        </AbsoluteFill>
      ))}
    </AbsoluteFill>
  );
}
