import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { Img } from "@opsis/runtime/parts.tsx";
import type { CSSProperties } from "react";
import { CLAMPED } from "#video/core/anim.ts";
import { FootageFrame } from "#video/core/footage.tsx";
import type { MediaKind } from "#video/generated/spec.ts";

export function ScreenMedia({
  src,
  kind,
  frame,
  footage,
}: {
  src: string;
  kind: MediaKind | undefined;
  frame: number;
  footage?: string;
}) {
  const now = useFrame();
  const { fps } = useFilm();
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
  if (kind !== "video") return <Img src={src} style={style} />;
  return footage ? (
    <FootageFrame url={footage} timeS={now / fps} fit="cover" style={style} />
  ) : null;
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
