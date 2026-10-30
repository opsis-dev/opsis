import type { CSSProperties } from "react";
import { Img, interpolate, OffthreadVideo } from "remotion";
import { CLAMPED } from "#video/core/anim.ts";
import type { MediaKind } from "#video/generated/spec.ts";

export function ScreenMedia({
  src,
  kind,
  frame,
}: {
  src: string;
  kind: MediaKind | undefined;
  frame: number;
}) {
  const t = interpolate(frame, [4, 22], [0, 1], CLAMPED);
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
  return kind === "video" ? (
    <OffthreadVideo src={src} muted style={style} />
  ) : (
    <Img src={src} style={style} />
  );
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
