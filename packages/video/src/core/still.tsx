import React from "react";
import { AbsoluteFill, Img, useCurrentFrame, useVideoConfig } from "remotion";
import { isLightBackdrop } from "#video/camera/lib/backdrop.ts";
import { WINDOW_FIT } from "#video/camera/lib/style.ts";
import { RimLight } from "#video/camera/stage.tsx";
import { WindowFrame } from "#video/camera/window-frame.tsx";
import { DEFAULT_BACKDROP } from "#video/common/backdrops.ts";
import { zoomOver } from "#video/core/anim.ts";
import { stillLayout, type Box, type Shot, type StillLook } from "#video/core/still-layout.ts";
import { Backdrop } from "#video/elements/background/backdrop.tsx";

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
