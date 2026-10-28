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
const GLARE = { angle: 115, strongest: 0.16, travel: 120 };

const Glare: React.FC<{ turn: number }> = ({ turn }) => (
  <AbsoluteFill
    style={{
      pointerEvents: "none",
      background: `linear-gradient(${GLARE.angle}deg, rgba(255,255,255,${GLARE.strongest}) 0%, rgba(255,255,255,0) 45%)`,
      backgroundSize: "200% 100%",
      backgroundPositionX: `${50 + turn * GLARE.travel}%`,
      opacity: Math.min(1, Math.abs(turn) * 3),
    }}
  />
);

const OnScreen: React.FC<{ quad: Point2[]; turn: number; children?: React.ReactNode }> = ({
  quad,
  turn,
  children,
}) => {
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
      <Glare turn={turn} />
    </div>
  );
};

export const Screen3D: React.FC<ElementSettings["screen-3d"] & { children?: React.ReactNode }> = ({
  device,
  deviceColor,
  move,
  camera,
  endFaceOn,
  lighting,
  background,
  children,
}) => {
  const frame = useCurrentFrame();
  const { width, height, durationInFrames } = useVideoConfig();
  const screen = useMemo(
    () => ({ width: SCREEN_WIDTH, height: SCREEN_WIDTH / (width / height) }),
    [width, height],
  );
  const distance = screenDistance(screen.height, STAGE_FOV);
  const pose = screenPoseAt(move, frame, durationInFrames, endFaceOn, camera);
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
      <OnScreen quad={quad} turn={pose.rotation[1]}>
        {children}
      </OnScreen>
    ) : null;
  return (
    <Stage3D lighting={lighting} background={background} distance={distance} overlay={overlay}>
      <Posed pose={pose}>
        <DeviceBody device={device} screen={screen} color={deviceColor} />
      </Posed>
    </Stage3D>
  );
};
