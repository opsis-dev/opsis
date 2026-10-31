import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill, Img } from "@opsis/runtime/parts.tsx";
import React from "react";
import { isLightBackdrop } from "#video/camera/lib/backdrop.ts";
import { RimLight } from "#video/camera/stage.tsx";
import { WindowFrame } from "#video/camera/window-frame.tsx";
import { zoomOver } from "#video/core/anim.ts";
import { stillLayout, type Box, type StillLook } from "#video/core/still-layout.ts";
import { Backdrop } from "#video/elements/background/backdrop.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";

type StillProps = ElementSettings["still"];

function PushIn({
  origin,
  scale,
  frames,
  children,
}: {
  origin?: string;
  scale: number;
  frames: number;
  children: React.ReactNode;
}) {
  const frame = useFrame();
  const push = zoomOver(frame, frames, scale);
  return (
    <AbsoluteFill style={{ transform: `scale(${push})`, transformOrigin: origin }}>
      {children}
    </AbsoluteFill>
  );
}

function placed(box: Box): React.CSSProperties {
  return {
    position: "absolute",
    left: box.x,
    top: box.y,
    width: box.width,
    height: box.height,
  };
}

type StillLookProps = { box: Box; props: StillProps };

function BleedStill({ box, props }: StillLookProps) {
  return (
    <PushIn
      origin={props.align === "middle" ? "50% 50%" : "50% 0%"}
      scale={props.pushIn}
      frames={props.pushInFrames}
    >
      <Img src={props.src} style={placed(box)} />
    </PushIn>
  );
}

function OnBackdrop({ props, children }: { props: StillProps; children: React.ReactNode }) {
  return (
    <AbsoluteFill>
      <Backdrop name={props.backdrop} />
      <PushIn scale={props.pushIn} frames={props.pushInFrames}>
        {children}
      </PushIn>
    </AbsoluteFill>
  );
}

function CutoutStill({ box, props }: StillLookProps) {
  return (
    <OnBackdrop props={props}>
      <Img
        src={props.src}
        style={{
          ...placed(box),
          filter: `drop-shadow(0 24px 48px rgba(0,0,0,${props.shadow}))`,
        }}
      />
    </OnBackdrop>
  );
}

function WindowStill({ box, props }: StillLookProps) {
  return (
    <OnBackdrop props={props}>
      <div style={placed(box)}>
        <RimLight light={isLightBackdrop(props.backdrop)} />
        <WindowFrame chrome={props.frame !== undefined} windowStyle={props.frame}>
          <Img
            src={props.src}
            style={{ width: "100%", height: "100%", objectFit: "cover", objectPosition: "top" }}
          />
        </WindowFrame>
      </div>
    </OnBackdrop>
  );
}

const STILL_LOOKS: Record<StillLook, (props: StillLookProps) => React.ReactNode> = {
  bleed: BleedStill,
  cutout: CutoutStill,
  window: WindowStill,
};

export function StillElement(props: StillProps) {
  const { width, height } = useFilm();
  if (!props.src) return null;
  const shot = props.shot ?? { width, height, scale: 1 };
  const box = stillLayout(props.look, shot, { width, height }, props.fit, props.align);
  const Look = STILL_LOOKS[props.look];
  return <Look box={box} props={props} />;
}
