import React, { useEffect, useMemo } from "react";
import { staticFile, useCurrentFrame, useVideoConfig } from "remotion";
import {
  Box3,
  CurvePath,
  DoubleSide,
  ExtrudeGeometry,
  Group,
  LineCurve3,
  Mesh,
  MeshStandardMaterial,
  TubeGeometry,
  Vector3,
  type BufferGeometry,
  type Object3D,
} from "three";
import type { SVGResult } from "three/addons/loaders/SVGLoader.js";
import type { CoreElementProps } from "../../shared/elements/core";
import { isRecord } from "../../shared/guards";
import { Card } from "./card-3d";
import { isSvg, useSvg, useTexture } from "./load";
import { elementPose } from "./motion";
import { fitToSize } from "./shapes";
import { Posed, Stage3D } from "./stage";

const SAMPLE_LOGO = "placeholders/logo-mark.svg";
const LOGO_SIZE = 2;
const ROUGHNESS = 0.35;
const BEVEL_SHARE = 0.08;
const CURVE_SEGMENTS = 24;
const TUBE_SIDES = 12;
const SEED = 1;

type Look = { depth: number; color?: string; metalness: number };

function svgBounds(svg: SVGResult): number {
  const box = new Box3();
  for (const path of svg.paths)
    for (const sub of path.subPaths)
      for (const p of sub.getPoints()) box.expandByPoint(new Vector3(p.x, p.y, 0));
  const size = box.getSize(new Vector3());
  return Math.max(size.x, size.y) || 1;
}

const material = (color: string, look: Look) =>
  new MeshStandardMaterial({
    color: look.color ?? color,
    metalness: look.metalness,
    roughness: ROUGHNESS,
    side: DoubleSide,
  });

function strokeTube(points: Vector3[], radius: number): TubeGeometry | null {
  if (points.length < 2) return null;
  const closed = points[0].distanceTo(points[points.length - 1]) < radius * 0.5;
  const ends = closed ? points.slice(0, -1) : points;
  const path = new CurvePath<Vector3>();
  for (let i = 0; i < ends.length - 1; i++) path.add(new LineCurve3(ends[i], ends[i + 1]));
  if (closed) path.add(new LineCurve3(ends[ends.length - 1], ends[0]));
  return new TubeGeometry(path, Math.max(8, ends.length * 8), radius, TUBE_SIDES, closed);
}

function svgModel(svg: SVGResult, look: Look): Group {
  const extent = svgBounds(svg);
  const depth = look.depth * extent * 0.5;
  const flipped = new Group();
  for (const path of svg.paths) {
    const style: Record<string, unknown> = isRecord(path.userData?.style)
      ? path.userData.style
      : {};
    const fill = typeof style.fill === "string" ? style.fill : "none";
    const stroke = typeof style.stroke === "string" ? style.stroke : "none";
    if (fill !== "none") {
      const geometry = new ExtrudeGeometry(path.toShapes(), {
        depth,
        curveSegments: CURVE_SEGMENTS,
        bevelEnabled: true,
        bevelThickness: depth * BEVEL_SHARE,
        bevelSize: depth * BEVEL_SHARE,
        bevelSegments: 3,
      });
      geometry.translate(0, 0, -depth / 2);
      flipped.add(new Mesh(geometry, material(`#${path.color.getHexString()}`, look)));
    }
    if (stroke !== "none") {
      const width = Number(style.strokeWidth) || 1;
      for (const sub of path.subPaths) {
        const tube = strokeTube(
          sub.getPoints(CURVE_SEGMENTS).map((p) => new Vector3(p.x, p.y, 0)),
          Math.max(width / 2, depth / 2),
        );
        if (tube) flipped.add(new Mesh(tube, material(stroke, look)));
      }
    }
  }
  flipped.scale.y = -1;
  const model = new Group();
  model.add(flipped);
  fitToSize(flipped, LOGO_SIZE);
  return model;
}

function dispose(object: Object3D): void {
  object.traverse((child) => {
    if (!(child instanceof Mesh)) return;
    const geometry: BufferGeometry = child.geometry;
    geometry.dispose();
    const materials: unknown[] = Array.isArray(child.material) ? child.material : [child.material];
    for (const m of materials) if (m instanceof MeshStandardMaterial) m.dispose();
  });
}

const SvgLogo: React.FC<{ svg: SVGResult; look: Look }> = ({ svg, look }) => {
  const model = useMemo(() => svgModel(svg, look), [svg, look]);
  useEffect(() => () => dispose(model), [model]);
  return <primitive object={model} />;
};

export const Logo3D: React.FC<CoreElementProps["core/logo-3d"]> = ({
  src,
  depth,
  color,
  metalness,
  motion,
  camera,
  speed,
  lighting,
  background,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const file = src || staticFile(SAMPLE_LOGO);
  const vector = isSvg(file);
  const svg = useSvg(vector ? file : null);
  const texture = useTexture(vector ? null : file);
  const look = useMemo(
    () => ({ depth, color: color || undefined, metalness }),
    [depth, color, metalness],
  );
  const pose = elementPose(motion, camera, { frame, fps, speed, seed: SEED });
  return (
    <Stage3D lighting={lighting} background={background}>
      <Posed pose={pose}>
        {svg ? <SvgLogo svg={svg} look={look} /> : null}
        {texture ? (
          <Card
            texture={texture}
            longest={LOGO_SIZE}
            radius={0.12}
            depth={depth * 0.4}
            look={look}
          />
        ) : null}
      </Posed>
    </Stage3D>
  );
};
