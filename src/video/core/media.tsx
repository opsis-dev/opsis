import React from "react";
import { AbsoluteFill, Img, interpolate, useCurrentFrame, useVideoConfig } from "remotion";
import { Video } from "@remotion/media";
import { Backdrop } from "../camera/demo-clip";
import { RimLight, Stage, WindowFrame } from "../camera/window-frame";
import { WINDOW_FIT } from "../camera/lib/window";
import { isLightBackdrop } from "../camera/lib/backdrop";

type Fit = "contain" | "cover" | "fill";

export const ImageElement: React.FC<{
  src: string;
  fit?: Fit;
  background?: string;
  kenBurns?: boolean;
}> = ({ src, fit = "contain", background, kenBurns }) => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  if (!src) return null;
  const scale = kenBurns ? interpolate(frame, [0, durationInFrames], [1, 1.06]) : 1;
  return (
    <AbsoluteFill
      style={{ background: background && background !== "transparent" ? background : undefined }}
    >
      <Img
        src={src}
        style={{ width: "100%", height: "100%", objectFit: fit, transform: `scale(${scale})` }}
      />
    </AbsoluteFill>
  );
};

export const VideoElement: React.FC<{
  src: string;
  fit?: Fit;
  loop?: boolean;
  muted?: boolean;
  volume?: number;
  playbackRate?: number;
  trimBeforeMs?: number;
}> = ({
  src,
  fit = "cover",
  loop = false,
  muted = true,
  volume = 1,
  playbackRate = 1,
  trimBeforeMs = 0,
}) => {
  const { fps } = useVideoConfig();
  if (!src) return null;
  return (
    <AbsoluteFill>
      <Video
        src={src}
        loop={loop}
        muted={muted}
        volume={volume}
        playbackRate={playbackRate}
        trimBefore={trimBeforeMs ? Math.round((trimBeforeMs / 1000) * fps) : undefined}
        objectFit={fit}
        style={{ width: "100%", height: "100%" }}
      />
    </AbsoluteFill>
  );
};

export const VideoLoop: React.FC<{ src: string; dim?: number }> = ({ src, dim = 0.3 }) =>
  src ? (
    <AbsoluteFill>
      <VideoElement src={src} loop muted fit="cover" />
      <AbsoluteFill style={{ background: `rgba(0,0,0,${dim})` }} />
    </AbsoluteFill>
  ) : null;

export const StillElement: React.FC<{
  src: string;
  look?: "window" | "cutout" | "bleed";
  backdrop?: string;
}> = ({ src, look = "window", backdrop = "glaze" }) => {
  const frame = useCurrentFrame();
  const push = interpolate(frame, [0, 120], [1, 1.02], { extrapolateRight: "clamp" });
  if (!src) return null;
  if (look === "bleed") {
    return (
      <AbsoluteFill>
        <Img
          src={src}
          style={{ width: "100%", height: "100%", objectFit: "cover", transform: `scale(${push})` }}
        />
      </AbsoluteFill>
    );
  }
  return (
    <AbsoluteFill>
      <Backdrop name={backdrop} />
      <Stage>
        <div
          style={{
            position: "relative",
            width: `${WINDOW_FIT * 100}%`,
            height: `${WINDOW_FIT * 100}%`,
            transform: `scale(${push})`,
          }}
        >
          {look === "window" ? (
            <>
              <RimLight light={isLightBackdrop(backdrop)} />
              <WindowFrame>
                <Img src={src} style={{ width: "100%", height: "100%", objectFit: "cover" }} />
              </WindowFrame>
            </>
          ) : (
            <Img
              src={src}
              style={{
                width: "100%",
                height: "100%",
                objectFit: "contain",
                filter: "drop-shadow(0 24px 48px rgba(0,0,0,0.45))",
              }}
            />
          )}
        </div>
      </Stage>
    </AbsoluteFill>
  );
};
