import { Canvas, useThree } from "@react-three/fiber";
import React, { useCallback, useEffect, useLayoutEffect, useMemo, useRef } from "react";
import { useVideoConfig } from "remotion";
import { PMREMGenerator } from "three";
import { RoomEnvironment } from "three/addons/environments/RoomEnvironment.js";
import { holdRender } from "#video/core/loading.ts";
import type { Lighting } from "#video/generated/spec.ts";
import { Backgrounded } from "#video/lib/backgrounds/backgrounded.tsx";
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

function Reflections() {
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
}

function Lights({ lighting }: { lighting: Lighting }) {
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
}

type Drawn = (scene: object) => void;

function Draw({
  scene,
  width,
  height,
  onDrawn,
}: {
  scene: object;
  width: number;
  height: number;
  onDrawn: Drawn;
}) {
  const setSize = useThree((state) => state.setSize);
  const advance = useThree((state) => state.advance);
  useLayoutEffect(() => setSize(width, height), [setSize, width, height]);
  useLayoutEffect(() => {
    advance(performance.now());
    onDrawn(scene);
  }, [advance, onDrawn, scene]);
  return null;
}

function useDrawnScene(scene: object): Drawn {
  const drawn = useRef<object | null>(null);
  const pending = useRef<{ scene: object; release: () => void } | null>(null);
  useLayoutEffect(() => {
    if (drawn.current === scene) return;
    const release = holdRender("Drawing the 3D scene");
    pending.current = { scene, release };
    return release;
  }, [scene]);
  return useCallback((done: object) => {
    drawn.current = done;
    if (pending.current?.scene === done) pending.current.release();
  }, []);
}

export function Posed({ pose, children }: { pose: Pose; children: React.ReactNode }) {
  return (
    <group rotation={pose.rotation} position={pose.position} scale={pose.scale}>
      {children}
    </group>
  );
}

export const STAGE_FOV = FOV;

export function Stage3D({
  lighting,
  background,
  radius = SUBJECT_RADIUS,
  distance: fixed,
  box,
  overlay,
  fov,
  children,
}: {
  fov: number;
  lighting: Lighting;
  background?: string;
  radius?: number;
  distance?: number;
  box?: { width: number; height: number };
  overlay?: React.ReactNode;
  children: React.ReactNode;
}) {
  const { width, height } = useVideoConfig();
  const aspect = width / height;
  const distance =
    fixed ?? Math.max(fitDistance(radius, fov, aspect), box ? boxDistance(box, fov, aspect) : 0);
  const scene = useMemo(() => ({ children }), [children]);
  const onDrawn = useDrawnScene(scene);
  return (
    <Backgrounded background={background}>
      <Canvas
        key={`${width}x${height}:${lighting}:${distance}`}
        frameloop="never"
        resize={{ scroll: false }}
        style={{ width, height }}
        gl={{ antialias: true, alpha: true, preserveDrawingBuffer: true }}
        camera={{ fov, position: [0, 0, distance], near: 0.1, far: distance * 10 }}
        scene={{ environmentIntensity: LOOKS[lighting].reflections }}
      >
        <Lights lighting={lighting} />
        {children}
        <Draw scene={scene} width={width} height={height} onDrawn={onDrawn} />
      </Canvas>
      {overlay}
    </Backgrounded>
  );
}
