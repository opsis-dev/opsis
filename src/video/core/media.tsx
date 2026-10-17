import React from "react";
import { AbsoluteFill, Img, interpolate, useCurrentFrame, useVideoConfig } from "remotion";
import { Video } from "@remotion/media";
import { Backdrop } from "../camera/demo-clip";
import { RimLight, WindowFrame } from "../camera/window-frame";
import { WINDOW_FIT } from "../camera/lib/style";
import { isLightBackdrop } from "../camera/lib/backdrop";
import { stillLayout, type Box, type Shot, type StillLook } from "./still-layout";
import { visibleBackground } from "./anim";
import { msToFrames } from "../../shared/timeline/time";
import { DEFAULT_BACKDROP } from "../../shared/backdrops";

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
    <AbsoluteFill style={{ background: visibleBackground(background) }}>
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
        trimBefore={trimBeforeMs ? msToFrames(trimBeforeMs, fps, 0) : undefined}
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

const PUSH_IN_FRAMES = 120;
const PUSH_IN_SCALE = 1.02;

const PushIn: React.FC<{ origin?: string; children: React.ReactNode }> = ({ origin, children }) => {
  const frame = useCurrentFrame();
  const push = interpolate(frame, [0, PUSH_IN_FRAMES], [1, PUSH_IN_SCALE], {
    extrapolateRight: "clamp",
  });
  return (
    <AbsoluteFill style={{ transform: `scale(${push})`, transformOrigin: origin }}>
      {children}
    </AbsoluteFill>
  );
};

const placed = (box: Box): React.CSSProperties => ({
  position: "absolute",
  left: box.x,
  top: box.y,
  width: box.width,
  height: box.height,
});

export const StillElement: React.FC<{
  src: string;
  look?: StillLook;
  backdrop?: string;
  shot?: Shot;
}> = ({ src, look = "window", backdrop = DEFAULT_BACKDROP, shot }) => {
  const { width, height } = useVideoConfig();
  if (!src) return null;
  const box = stillLayout(look, shot ?? { width, height, scale: 1 }, { width, height }, WINDOW_FIT);
  if (look === "bleed") {
    return (
      <PushIn origin="50% 0%">
        <Img src={src} style={placed(box)} />
      </PushIn>
    );
  }
  return (
    <AbsoluteFill>
      <Backdrop name={backdrop} />
      <PushIn>
        {look === "cutout" ? (
          <Img
            src={src}
            style={{ ...placed(box), filter: "drop-shadow(0 24px 48px rgba(0,0,0,0.45))" }}
          />
        ) : (
          <div style={placed(box)}>
            <RimLight light={isLightBackdrop(backdrop)} />
            <WindowFrame>
              <Img
                src={src}
                style={{ width: "100%", height: "100%", objectFit: "cover", objectPosition: "top" }}
              />
            </WindowFrame>
          </div>
        )}
      </PushIn>
    </AbsoluteFill>
  );
};
