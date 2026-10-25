import type { Lighting } from "@opsis/core/elements/three";
import { useThree } from "@react-three/fiber";
import { ThreeCanvas } from "@remotion/three";
import React, { useEffect, useMemo } from "react";
import { useVideoConfig } from "remotion";
import { PMREMGenerator } from "three";
import { RoomEnvironment } from "three/addons/environments/RoomEnvironment.js";
import { Backgrounded } from "#video/core/backgrounds.tsx";
import { boxDistance, fitDistance, type Pose, type Vec3 } from "#video/three/motion.ts";

const FOV = 30;
const SUBJECT_RADIUS = 1.15;
const ROOM_BLUR = 0.04;

type Light = { position: Vec3; intensity: number };

type LightingLook = {
  ambient: number;
  sky?: number;
  lights: Light[];
  reflections: number;
};

const LOOKS: Record<Lighting, LightingLook> = {
  studio: {
    ambient: 0.35,
    lights: [
      { position: [4, 5, 6], intensity: 2.2 },
      { position: [-5, 2, 4], intensity: 0.7 },
      { position: [0, 4, -6], intensity: 1.2 },
    ],
    reflections: 0.8,
  },
  soft: {
    ambient: 0.6,
    sky: 0.9,
    lights: [{ position: [2, 4, 6], intensity: 0.9 }],
    reflections: 0.5,
  },
  dramatic: {
    ambient: 0.08,
    lights: [
      { position: [5, 6, 3], intensity: 3.2 },
      { position: [-4, 1, -5], intensity: 1.6 },
    ],
    reflections: 0.35,
  },
};

const Reflections: React.FC = () => {
  const gl = useThree((state) => state.gl);
  const map = useMemo(() => {
    const pmrem = new PMREMGenerator(gl);
    const room = new RoomEnvironment();
    const texture = pmrem.fromScene(room, ROOM_BLUR).texture;
    room.dispose();
    pmrem.dispose();
    return texture;
  }, [gl]);
  useEffect(() => () => map.dispose(), [map]);
  return <primitive object={map} attach="environment" />;
};

const Lights: React.FC<{ lighting: Lighting }> = ({ lighting }) => {
  const look = LOOKS[lighting];
  return (
    <>
      <ambientLight intensity={look.ambient} />
      {look.sky ? <hemisphereLight intensity={look.sky} /> : null}
      {look.lights.map((light, i) => (
        <directionalLight key={i} position={light.position} intensity={light.intensity} />
      ))}
      <Reflections />
    </>
  );
};

export const Posed: React.FC<{ pose: Pose; children: React.ReactNode }> = ({ pose, children }) => (
  <group rotation={pose.rotation} position={pose.position} scale={pose.scale}>
    {children}
  </group>
);

export const STAGE_FOV = FOV;

export const Stage3D: React.FC<{
  lighting: Lighting;
  background?: string;
  radius?: number;
  distance?: number;
  box?: { width: number; height: number };
  overlay?: React.ReactNode;
  children: React.ReactNode;
}> = ({
  lighting,
  background,
  radius = SUBJECT_RADIUS,
  distance: fixed,
  box,
  overlay,
  children,
}) => {
  const { width, height } = useVideoConfig();
  const aspect = width / height;
  const distance =
    fixed ?? Math.max(fitDistance(radius, FOV, aspect), box ? boxDistance(box, FOV, aspect) : 0);
  return (
    <Backgrounded background={background}>
      <ThreeCanvas
        key={`${width}x${height}:${lighting}:${distance}`}
        width={width}
        height={height}
        gl={{ antialias: true, alpha: true, preserveDrawingBuffer: true }}
        camera={{ fov: FOV, position: [0, 0, distance], near: 0.1, far: distance * 10 }}
        scene={{ environmentIntensity: LOOKS[lighting].reflections }}
      >
        <Lights lighting={lighting} />
        {children}
      </ThreeCanvas>
      {overlay}
    </Backgrounded>
  );
};
