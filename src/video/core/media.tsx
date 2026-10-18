import { Video } from "@remotion/media";
import React from "react";
import { AbsoluteFill, Img, useCurrentFrame, useVideoConfig } from "remotion";
import { DEFAULT_BACKDROP } from "../../shared/backdrops";
import { msToFrames } from "../../shared/timeline/time";
import { isLightBackdrop } from "../camera/lib/backdrop";
import { WINDOW_FIT } from "../camera/lib/style";
import { RimLight, WindowFrame } from "../camera/window-frame";
import { zoomOver } from "./anim";
import { Backdrop, Backgrounded } from "./backgrounds";
import { stillLayout, type Box, type Shot, type StillLook } from "./still-layout";

type Fit = "contain" | "cover" | "fill";

const KEN_BURNS_SCALE = 1.06;

export const ImageElement: React.FC<{
  src: string;
  fit?: Fit;
  background?: string;
  kenBurns?: boolean;
}> = ({ src, fit = "contain", background, kenBurns }) => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  if (!src) return null;
  const scale = kenBurns ? zoomOver(frame, durationInFrames, KEN_BURNS_SCALE) : 1;
  return (
    <Backgrounded background={background}>
      <Img
        src={src}
        style={{ width: "100%", height: "100%", objectFit: fit, transform: `scale(${scale})` }}
      />
    </Backgrounded>
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
      <VideoElement src={src} loop />
      <AbsoluteFill style={{ background: `rgba(0,0,0,${dim})` }} />
    </AbsoluteFill>
  ) : null;

const PUSH_IN_FRAMES = 120;
const PUSH_IN_SCALE = 1.02;

const PushIn: React.FC<{ origin?: string; children: React.ReactNode }> = ({ origin, children }) => {
  const frame = useCurrentFrame();
  const push = zoomOver(frame, PUSH_IN_FRAMES, PUSH_IN_SCALE);
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

type StillLookProps = { src: string; box: Box; backdrop: string };

const BleedStill: React.FC<StillLookProps> = ({ src, box }) => (
  <PushIn origin="50% 0%">
    <Img src={src} style={placed(box)} />
  </PushIn>
);

const OnBackdrop: React.FC<{ backdrop: string; children: React.ReactNode }> = ({
  backdrop,
  children,
}) => (
  <AbsoluteFill>
    <Backdrop name={backdrop} />
    <PushIn>{children}</PushIn>
  </AbsoluteFill>
);

const CutoutStill: React.FC<StillLookProps> = ({ src, box, backdrop }) => (
  <OnBackdrop backdrop={backdrop}>
    <Img
      src={src}
      style={{ ...placed(box), filter: "drop-shadow(0 24px 48px rgba(0,0,0,0.45))" }}
    />
  </OnBackdrop>
);

const WindowStill: React.FC<StillLookProps> = ({ src, box, backdrop }) => (
  <OnBackdrop backdrop={backdrop}>
    <div style={placed(box)}>
      <RimLight light={isLightBackdrop(backdrop)} />
      <WindowFrame>
        <Img
          src={src}
          style={{ width: "100%", height: "100%", objectFit: "cover", objectPosition: "top" }}
        />
      </WindowFrame>
    </div>
  </OnBackdrop>
);

const STILL_LOOKS: Record<StillLook, React.FC<StillLookProps>> = {
  bleed: BleedStill,
  cutout: CutoutStill,
  window: WindowStill,
};

export const StillElement: React.FC<{
  src: string;
  look?: StillLook;
  backdrop?: string;
  shot?: Shot;
}> = ({ src, look = "window", backdrop = DEFAULT_BACKDROP, shot }) => {
  const { width, height } = useVideoConfig();
  if (!src) return null;
  const box = stillLayout(look, shot ?? { width, height, scale: 1 }, { width, height }, WINDOW_FIT);
  const Look = STILL_LOOKS[look];
  return <Look src={src} box={box} backdrop={backdrop} />;
};
