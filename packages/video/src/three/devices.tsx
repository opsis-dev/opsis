import type { Device } from "@opsis/core/elements/screen-3d";
import React, { useEffect, useMemo } from "react";
import {
  BoxGeometry,
  ExtrudeGeometry,
  MeshStandardMaterial,
  PlaneGeometry,
  ShapeGeometry,
  type BufferGeometry,
} from "three";
import { RoundedBoxGeometry } from "three/addons/geometries/RoundedBoxGeometry.js";
import type { Vec3 } from "./motion";
import { roundedRect } from "./shapes";

const BEZEL_COLOR = "#0a0a0c";
const BODY = { roughness: 0.35, metalness: 0.6 };

type Part = { geometry: BufferGeometry; position: Vec3 };

type Build = { front: { width: number; height: number; radius: number }; parts: Part[] };

type ScreenSize = { width: number; height: number };

function slab(width: number, height: number, radius: number, depth: number): ExtrudeGeometry {
  const geometry = new ExtrudeGeometry(roundedRect(width, height, radius), {
    depth,
    bevelEnabled: true,
    bevelThickness: depth * 0.15,
    bevelSize: Math.min(radius, depth) * 0.3,
    bevelSegments: 3,
    curveSegments: 12,
  });
  geometry.translate(0, 0, -depth * 1.15);
  return geometry;
}

const front = (width: number, height: number, radius: number) => ({ width, height, radius });

function partsFor(device: Device, { width: w, height: h }: ScreenSize): Build {
  const unit = Math.min(w, h);
  switch (device) {
    case "floating": {
      const edge = unit * 0.015;
      return {
        front: front(w + edge * 2, h + edge * 2, unit * 0.04),
        parts: [
          {
            geometry: slab(w + edge * 2, h + edge * 2, unit * 0.04, unit * 0.03),
            position: [0, 0, -0.003],
          },
        ],
      };
    }
    case "phone": {
      const edge = unit * 0.05;
      return {
        front: front(w + edge * 2, h + edge * 2, unit * 0.12),
        parts: [
          {
            geometry: slab(w + edge * 2, h + edge * 2, unit * 0.12, unit * 0.07),
            position: [0, 0, -0.003],
          },
        ],
      };
    }
    case "monitor": {
      const bezel = unit * 0.05;
      const neck = h * 0.32;
      const bottom = -h / 2 - bezel;
      return {
        front: front(w + bezel * 2, h + bezel * 2, unit * 0.03),
        parts: [
          {
            geometry: slab(w + bezel * 2, h + bezel * 2, unit * 0.03, unit * 0.06),
            position: [0, 0, -0.003],
          },
          {
            geometry: new BoxGeometry(w * 0.12, neck, unit * 0.05),
            position: [0, bottom - neck / 2 + unit * 0.02, -unit * 0.12],
          },
          {
            geometry: new RoundedBoxGeometry(w * 0.36, unit * 0.04, unit * 0.45, 4, unit * 0.015),
            position: [0, bottom - neck + unit * 0.02, -unit * 0.1],
          },
        ],
      };
    }
    case "laptop": {
      const bezel = unit * 0.045;
      const deck = h * 0.72;
      const thick = unit * 0.035;
      const hinge = -h / 2 - bezel;
      return {
        front: front(w + bezel * 2, h + bezel * 2, unit * 0.03),
        parts: [
          {
            geometry: slab(w + bezel * 2, h + bezel * 2, unit * 0.03, unit * 0.025),
            position: [0, 0, -0.003],
          },
          {
            geometry: new RoundedBoxGeometry(w + bezel * 2.4, thick, deck, 4, thick * 0.4),
            position: [0, hinge - thick / 2, deck / 2 - unit * 0.03],
          },
        ],
      };
    }
  }
}

export const DeviceBody: React.FC<{ device: Device; screen: ScreenSize; color: string }> = ({
  device,
  screen,
  color,
}) => {
  const build = useMemo(() => partsFor(device, screen), [device, screen]);
  const materials = useMemo(
    () => ({
      body: new MeshStandardMaterial({ color, ...BODY }),
      bezel: new MeshStandardMaterial({ color: BEZEL_COLOR, roughness: 0.15 }),
    }),
    [color],
  );
  const faces = useMemo(
    () => ({
      bezel: new ShapeGeometry(
        roundedRect(build.front.width, build.front.height, build.front.radius),
        12,
      ),
      glass: new PlaneGeometry(screen.width, screen.height),
    }),
    [build, screen],
  );
  useEffect(
    () => () => {
      for (const p of build.parts) p.geometry.dispose();
      faces.bezel.dispose();
      faces.glass.dispose();
      materials.body.dispose();
      materials.bezel.dispose();
    },
    [build, faces, materials],
  );
  return (
    <group>
      <mesh geometry={faces.glass} material={materials.bezel} position={[0, 0, -0.001]} />
      <mesh geometry={faces.bezel} material={materials.bezel} position={[0, 0, -0.002]} />
      {build.parts.map((p, i) => (
        <mesh key={i} geometry={p.geometry} material={materials.body} position={p.position} />
      ))}
    </group>
  );
};
