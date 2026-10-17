import React from "react";
import { AbsoluteFill, Img, interpolate, useCurrentFrame, useVideoConfig } from "remotion";
import { Video } from "@remotion/media";
import { Backdrop } from "../camera/demo-clip";
import { RimLight, WindowFrame } from "../camera/window-frame";
import { WINDOW_FIT } from "../camera/lib/style";
import { isLightBackdrop } from "../camera/lib/backdrop";
import {
  bleedLayout,
  cutoutLayout,
  windowLayout,
  type Box,
  type Shot,
  type StillLook,
} from "./still-layout";
import { msToFrames } from "../../shared/timeline/time";

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

export const StillElement: React.FC<{
  src: string;
  look?: StillLook;
  backdrop?: string;
  shot?: Shot;
}> = ({ src, look = "window", backdrop = "glaze", shot }) => {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();
  const push = interpolate(frame, [0, 120], [1, 1.02], { extrapolateRight: "clamp" });
  if (!src) return null;
  const canvas = { width, height };
  const size = shot ?? { width, height, scale: 1 };
  const place = (box: Box): React.CSSProperties => ({
    position: "absolute",
    left: box.x,
    top: box.y,
    width: box.width,
    height: box.height,
  });
  if (look === "bleed") {
    return (
      <AbsoluteFill style={{ transform: `scale(${push})`, transformOrigin: "50% 0%" }}>
        <Img src={src} style={place(bleedLayout(size, canvas))} />
      </AbsoluteFill>
    );
  }
  if (look === "cutout") {
    const { img } = cutoutLayout(size, canvas);
    return (
      <AbsoluteFill>
        <Backdrop name={backdrop} />
        <AbsoluteFill style={{ transform: `scale(${push})` }}>
          <Img
            src={src}
            style={{ ...place(img), filter: "drop-shadow(0 24px 48px rgba(0,0,0,0.45))" }}
          />
        </AbsoluteFill>
      </AbsoluteFill>
    );
  }
  const box = windowLayout(size, canvas, WINDOW_FIT);
  return (
    <AbsoluteFill>
      <Backdrop name={backdrop} />
      <AbsoluteFill style={{ transform: `scale(${push})` }}>
        <div style={place(box)}>
          <RimLight light={isLightBackdrop(backdrop)} />
          <WindowFrame>
            <Img
              src={src}
              style={{ width: "100%", height: "100%", objectFit: "cover", objectPosition: "top" }}
            />
          </WindowFrame>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
