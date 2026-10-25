import type { CoreElementProps } from "@opsis/core/elements/core";
import React, { useEffect, useMemo } from "react";
import { staticFile, useCurrentFrame, useVideoConfig } from "remotion";
import {
  CanvasTexture,
  ExtrudeGeometry,
  MeshBasicMaterial,
  MeshStandardMaterial,
  PlaneGeometry,
  type Texture,
} from "three";
import { useTexture } from "./load";
import { elementPose } from "./motion";
import { faceGeometry, fittedSize, roundedRect } from "./shapes";
import { Posed, Stage3D } from "./stage";

const SAMPLE_SCREEN = "placeholders/screen-wide.svg";
const CARD_SIZE = 2.2;
const FACE_GAP = 0.002;
const BODY_ROUGHNESS = 0.45;
const FACE_ROUGHNESS = 0.6;
const SHADOW = {
  spread: 1.25,
  drop: 0.18,
  back: 0.6,
  opacity: 0.55,
  px: 256,
  floorDepth: 0.6,
  floorGap: 0.01,
};
const SEED = 1;

type CardLook = { color?: string; metalness: number };

type CardProps = {
  texture: Texture;
  longest: number;
  radius: number;
  depth: number;
  look: CardLook;
};

const imageSize = (texture: Texture) => {
  const image: unknown = texture.image;
  const w = image instanceof HTMLImageElement ? image.naturalWidth || image.width : 1;
  const h = image instanceof HTMLImageElement ? image.naturalHeight || image.height : 1;
  return { w: w || 1, h: h || 1 };
};

export const Card: React.FC<CardProps> = ({ texture, longest, radius, depth, look }) => {
  const { w, h } = imageSize(texture);
  const { width, height } = fittedSize(w, h, longest);
  const corner = radius * longest;
  const parts = useMemo(() => {
    const shape = roundedRect(width, height, corner);
    const body = new ExtrudeGeometry(shape, {
      depth,
      bevelEnabled: true,
      bevelThickness: depth * 0.2,
      bevelSize: Math.min(corner, depth) * 0.2,
      bevelSegments: 4,
      curveSegments: 16,
    });
    body.translate(0, 0, -depth / 2);
    const face = faceGeometry(shape, width, height);
    const bodyMaterial = new MeshStandardMaterial({
      metalness: look.metalness,
      roughness: BODY_ROUGHNESS,
      ...(look.color ? { color: look.color } : {}),
    });
    const faceMaterial = new MeshStandardMaterial({ map: texture, roughness: FACE_ROUGHNESS });
    return { body, face, bodyMaterial, faceMaterial };
  }, [texture, width, height, corner, depth, look]);
  useEffect(
    () => () => {
      parts.body.dispose();
      parts.face.dispose();
      parts.bodyMaterial.dispose();
      parts.faceMaterial.dispose();
    },
    [parts],
  );
  const front = depth / 2 + depth * 0.2 + FACE_GAP;
  return (
    <group>
      <mesh geometry={parts.body} material={parts.bodyMaterial} />
      <mesh geometry={parts.face} material={parts.faceMaterial} position={[0, 0, front]} />
    </group>
  );
};

function shadowTexture(): CanvasTexture {
  const canvas = document.createElement("canvas");
  canvas.width = SHADOW.px;
  canvas.height = SHADOW.px;
  const ctx = canvas.getContext("2d");
  if (ctx) {
    const half = SHADOW.px / 2;
    const glow = ctx.createRadialGradient(half, half, 0, half, half, half);
    glow.addColorStop(0, "rgba(0,0,0,0.55)");
    glow.addColorStop(1, "rgba(0,0,0,0)");
    ctx.fillStyle = glow;
    ctx.fillRect(0, 0, SHADOW.px, SHADOW.px);
  }
  return new CanvasTexture(canvas);
}

function useShadowPlane(width: number, height: number) {
  const parts = useMemo(
    () => ({
      geometry: new PlaneGeometry(width * SHADOW.spread, height * SHADOW.spread),
      material: new MeshBasicMaterial({
        map: shadowTexture(),
        transparent: true,
        depthWrite: false,
        opacity: SHADOW.opacity,
      }),
    }),
    [width, height],
  );
  useEffect(
    () => () => {
      parts.geometry.dispose();
      parts.material.map?.dispose();
      parts.material.dispose();
    },
    [parts],
  );
  return parts;
}

const SoftShadow: React.FC<{ width: number; height: number }> = ({ width, height }) => {
  const parts = useShadowPlane(width, height);
  return (
    <mesh
      geometry={parts.geometry}
      material={parts.material}
      position={[0, -SHADOW.drop, -SHADOW.back]}
    />
  );
};

export const GroundShadow: React.FC<{ width: number; y: number }> = ({ width, y }) => {
  const parts = useShadowPlane(width, width * SHADOW.floorDepth);
  return (
    <mesh
      geometry={parts.geometry}
      material={parts.material}
      position={[0, y - SHADOW.floorGap, 0]}
      rotation={[-Math.PI / 2, 0, 0]}
    />
  );
};

export const Card3D: React.FC<CoreElementProps["core/card-3d"]> = ({
  src,
  cornerRadius,
  tilt,
  depth,
  motion,
  camera,
  speed,
  lighting,
  background,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const texture = useTexture(src || staticFile(SAMPLE_SCREEN));
  const look = useMemo(() => ({ metalness: 0 }), []);
  const pose = elementPose(motion, camera, { frame, fps, speed, seed: SEED, tiltDeg: tilt });
  if (!texture) return null;
  const { w, h } = imageSize(texture);
  const size = fittedSize(w, h, CARD_SIZE);
  return (
    <Stage3D lighting={lighting} background={background}>
      <SoftShadow width={size.width} height={size.height} />
      <Posed pose={pose}>
        <Card
          texture={texture}
          longest={CARD_SIZE}
          radius={cornerRadius}
          depth={depth}
          look={look}
        />
      </Posed>
    </Stage3D>
  );
};
