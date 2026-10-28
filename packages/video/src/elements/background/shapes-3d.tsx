import { noise3D } from "@remotion/noise";
import React, { useEffect, useMemo } from "react";
import { random, useCurrentFrame, useVideoConfig } from "remotion";
import { MeshStandardMaterial, SphereGeometry, TorusGeometry, type BufferGeometry } from "three";
import { RoundedBoxGeometry } from "three/addons/geometries/RoundedBoxGeometry.js";
import { cameraAt } from "#video/common/camera-path.ts";
import type { CoreElementProps } from "#video/generated/spec.ts";
import { viewPose } from "#video/three/motion.ts";
import { Posed, Stage3D } from "#video/three/stage.tsx";

const FOV_HALF = (30 * Math.PI) / 360;
const STAGE_DISTANCE = 1.15 / Math.sin(FOV_HALF);
const DEPTH = { near: 0.6, far: -3.5 };
const SIZE = { min: 0.14, max: 0.42 };
const DRIFT = { reach: 0.45, pace: 0.006, spin: 0.012 };
const SPREAD = 1.05;
const ROUGHNESS = 0.4;
const METALNESS = 0.1;

type Kind = "sphere" | "ring" | "box";
const KINDS: Kind[] = ["sphere", "ring", "box"];

type Placed = { kind: Kind; color: number; x: number; y: number; z: number; size: number };

function place(count: number, colors: number, seed: number, aspect: number): Placed[] {
  return Array.from({ length: count }, (_, i) => {
    const r = (key: string) => random(`${seed}:${i}:${key}`);
    const z = DEPTH.far + r("z") * (DEPTH.near - DEPTH.far);
    const halfH = (STAGE_DISTANCE - z) * Math.tan(FOV_HALF) * SPREAD;
    return {
      kind: KINDS[Math.floor(r("kind") * KINDS.length) % KINDS.length],
      color: colors ? i % colors : 0,
      x: (r("x") * 2 - 1) * halfH * aspect,
      y: (r("y") * 2 - 1) * halfH,
      z,
      size: SIZE.min + r("size") * (SIZE.max - SIZE.min),
    };
  });
}

function geometryFor(kind: Kind): BufferGeometry {
  switch (kind) {
    case "sphere":
      return new SphereGeometry(1, 48, 32);
    case "ring":
      return new TorusGeometry(0.8, 0.3, 32, 96);
    case "box":
      return new RoundedBoxGeometry(1.4, 1.4, 1.4, 6, 0.3);
  }
}

export const Shapes3D: React.FC<CoreElementProps["shapes-3d"]> = ({
  count,
  palette,
  seed,
  speed,
  camera,
  lighting,
  background,
}) => {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();
  const placed = useMemo(
    () => place(count, palette.length, seed, width / height),
    [count, palette.length, seed, width, height],
  );
  const parts = useMemo(
    () => ({
      geometries: {
        sphere: geometryFor("sphere"),
        ring: geometryFor("ring"),
        box: geometryFor("box"),
      } satisfies Record<Kind, BufferGeometry>,
      materials: palette.map(
        (color) => new MeshStandardMaterial({ color, roughness: ROUGHNESS, metalness: METALNESS }),
      ),
    }),
    [palette],
  );
  useEffect(
    () => () => {
      for (const g of Object.values(parts.geometries)) g.dispose();
      for (const m of parts.materials) m.dispose();
    },
    [parts],
  );
  const t = frame * speed;
  return (
    <Stage3D lighting={lighting} background={background}>
      <Posed pose={viewPose(cameraAt(camera, frame))}>
        {placed.map((p, i) => {
          const drift = (axis: string) =>
            noise3D(`${seed}:${axis}`, i * 7.3, 0, t * DRIFT.pace) * DRIFT.reach;
          const turn = (axis: string) =>
            noise3D(`${seed}:r${axis}`, i * 3.1, 0, t * DRIFT.spin) * Math.PI;
          const material = parts.materials[p.color];
          if (!material) return null;
          return (
            <mesh
              key={i}
              geometry={parts.geometries[p.kind]}
              material={material}
              position={[p.x + drift("x"), p.y + drift("y"), p.z + drift("z")]}
              rotation={[turn("x"), turn("y"), turn("z")]}
              scale={p.size}
            />
          );
        })}
      </Posed>
    </Stage3D>
  );
};
