import React, { useMemo } from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import type { ElementSettings } from "#video/generated/spec.ts";
import { DeviceBody } from "#video/three/devices.tsx";
import {
  facesCamera,
  quadMatrix,
  screenDistance,
  screenQuad,
  type Point2,
} from "#video/three/project.ts";
import { screenPoseAt } from "#video/three/screen-moves.ts";
import { Posed, STAGE_FOV, Stage3D } from "#video/three/stage.tsx";

const SCREEN_WIDTH = 2;
const GLARE = { angle: 115, travel: 120 };

function Glare({ turn, strength }: { turn: number; strength: number }) {
  return (
    <AbsoluteFill
      style={{
        pointerEvents: "none",
        background: `linear-gradient(${GLARE.angle}deg, rgba(255,255,255,${strength}) 0%, rgba(255,255,255,0) 45%)`,
        backgroundSize: "200% 100%",
        backgroundPositionX: `${50 + turn * GLARE.travel}%`,
        opacity: Math.min(1, Math.abs(turn) * 3),
      }}
    />
  );
}

function OnScreen({
  quad,
  turn,
  glare,
  children,
}: {
  quad: Point2[];
  turn: number;
  glare: number;
  children?: React.ReactNode;
}) {
  const { width, height } = useVideoConfig();
  return (
    <div
      style={{
        position: "absolute",
        left: 0,
        top: 0,
        width,
        height,
        overflow: "hidden",
        transformOrigin: "0 0",
        transform: quadMatrix(width, height, quad),
      }}
    >
      {children}
      <Glare turn={turn} strength={glare} />
    </div>
  );
}

export function Screen3D({
  device,
  deviceColor,
  move,
  camera,
  endFaceOn,
  lighting,
  background,
  bezelColor,
  metalness,
  roughness,
  glare,
  moveEndsAt,
  children,
}: ElementSettings["screen-3d"] & { children?: React.ReactNode }) {
  const frame = useCurrentFrame();
  const { width, height, durationInFrames } = useVideoConfig();
  const screen = useMemo(
    () => ({ width: SCREEN_WIDTH, height: SCREEN_WIDTH / (width / height) }),
    [width, height],
  );
  const distance = screenDistance(screen.height, STAGE_FOV);
  const pose = screenPoseAt(move, frame, durationInFrames, endFaceOn, moveEndsAt, camera);
  const quad = screenQuad({
    fovDeg: STAGE_FOV,
    distance,
    width,
    height,
    pose,
    screen: { ...screen, z: 0 },
  });
  const overlay =
    quad && facesCamera(quad) ? (
      <OnScreen quad={quad} turn={pose.rotation[1]} glare={glare}>
        {children}
      </OnScreen>
    ) : null;
  return (
    <Stage3D
      lighting={lighting}
      background={background}
      distance={distance}
      overlay={overlay}
      fov={STAGE_FOV}
    >
      <Posed pose={pose}>
        <DeviceBody
          device={device}
          screen={screen}
          color={deviceColor}
          bezelColor={bezelColor}
          metalness={metalness}
          roughness={roughness}
        />
      </Posed>
    </Stage3D>
  );
}
