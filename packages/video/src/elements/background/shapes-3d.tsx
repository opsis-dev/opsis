import { noise3D } from "@remotion/noise";
import { useEffect, useMemo } from "react";
import { random, useCurrentFrame, useVideoConfig } from "remotion";
import { MeshStandardMaterial, SphereGeometry, TorusGeometry, type BufferGeometry } from "three";
import { RoundedBoxGeometry } from "three/addons/geometries/RoundedBoxGeometry.js";
import { cameraAt } from "#video/common/camera-path.ts";
import type { ElementSettings, ShapeKind } from "#video/generated/spec.ts";
import { viewPose } from "#video/three/motion.ts";
import { Posed, STAGE_FOV, Stage3D } from "#video/three/stage.tsx";

const FOV_HALF = (30 * Math.PI) / 360;
const STAGE_DISTANCE = 1.15 / Math.sin(FOV_HALF);
const DEPTH = { near: 0.6, far: -3.5 };
const SIZE = { min: 0.14, max: 0.42 };
const DRIFT = { pace: 0.006, spin: 0.012 };
const SPREAD = 1.05;

type Placed = { kind: ShapeKind; color: number; x: number; y: number; z: number; size: number };

type Layout = {
  count: number;
  colors: number;
  seed: number;
  aspect: number;
  kinds: ShapeKind[];
  depth: number;
  size: number;
};

function place({ count, colors, seed, aspect, kinds, depth, size }: Layout): Placed[] {
  return Array.from({ length: count }, (_, i) => {
    function r(key: string) {
      return random(`${seed}:${i}:${key}`);
    }
    const z = (DEPTH.far + r("z") * (DEPTH.near - DEPTH.far)) * depth;
    const halfH = (STAGE_DISTANCE - z) * Math.tan(FOV_HALF) * SPREAD;
    return {
      kind: kinds[Math.floor(r("kind") * kinds.length) % kinds.length],
      color: colors ? i % colors : 0,
      x: (r("x") * 2 - 1) * halfH * aspect,
      y: (r("y") * 2 - 1) * halfH,
      z,
      size: (SIZE.min + r("size") * (SIZE.max - SIZE.min)) * size,
    };
  });
}

function geometryFor(kind: ShapeKind): BufferGeometry {
  switch (kind) {
    case "sphere":
      return new SphereGeometry(1, 48, 32);
    case "ring":
      return new TorusGeometry(0.8, 0.3, 32, 96);
    case "box":
      return new RoundedBoxGeometry(1.4, 1.4, 1.4, 6, 0.3);
  }
}

export function Shapes3D({
  count,
  palette,
  seed,
  speed,
  camera,
  lighting,
  background,
  depth,
  size,
  drift: reach,
  spin,
  roughness,
  metalness,
  kinds,
}: ElementSettings["shapes-3d"]) {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();
  const placed = useMemo(
    () =>
      place({
        count,
        colors: palette.length,
        seed,
        aspect: width / height,
        kinds,
        depth,
        size,
      }),
    [count, palette.length, seed, width, height, kinds, depth, size],
  );
  const parts = useMemo(
    () => ({
      geometries: {
        sphere: geometryFor("sphere"),
        ring: geometryFor("ring"),
        box: geometryFor("box"),
      } satisfies Record<ShapeKind, BufferGeometry>,
      materials: palette.map((color) => new MeshStandardMaterial({ color, roughness, metalness })),
    }),
    [palette, roughness, metalness],
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
    <Stage3D lighting={lighting} background={background} fov={STAGE_FOV}>
      <Posed pose={viewPose(cameraAt(camera, frame))}>
        {placed.map((p, i) => {
          function drift(axis: string) {
            return noise3D(`${seed}:${axis}`, i * 7.3, 0, t * DRIFT.pace) * reach;
          }
          function turn(axis: string) {
            return noise3D(`${seed}:r${axis}`, i * 3.1, 0, t * DRIFT.spin * spin) * Math.PI;
          }
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
}
