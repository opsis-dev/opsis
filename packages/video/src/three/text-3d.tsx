import type { CoreElementProps } from "@opsis/core/elements/core";
import { firstFamily } from "@opsis/core/fonts";
import React, { useEffect, useMemo } from "react";
import { useCurrentFrame, useVideoConfig } from "remotion";
import { Box3, Group, Matrix4, Mesh, MeshStandardMaterial, Vector3 } from "three";
import { TextGeometry } from "three/addons/geometries/TextGeometry.js";
import { Font, type FontData } from "three/addons/loaders/FontLoader.js";
import { useHeldLoad } from "../core/loading";
import { fontFileFor } from "../fonts/registry";
import { elementPose } from "./motion";
import { FONT_UNREADABLE, loadOutlines, wrapLines } from "./outlines";
import { fitToSize } from "./shapes";
import { Posed, Stage3D } from "./stage";

const HEADING_FONT = "df-heading";
const TEXT_SIZE = 2.6;
const LETTER_EM = 0.6;
const LINE_HEIGHT_EM = 1.15;
const DEPTH_EM = 0.6;
const BEVEL = { thickness: 0.05, size: 0.035, segments: 4 };
const CURVE_SEGMENTS = 10;
const ITALIC_SHEAR = 0.2;
const ROUGHNESS = 0.3;
const SEED = 1;
const KEY_GAP = "\u0000";

type TextLook = {
  text: string;
  depth: number;
  bevel: number;
  color: string;
  metalness: number;
  align: "center" | "left" | "right";
  lineWidth: number;
  slant: boolean;
};

function textModel(data: FontData, look: TextLook): Group {
  const font = new Font(data);
  const advance = (char: string) =>
    (data.glyphs[char] ?? data.glyphs["?"] ?? data.glyphs[" "])?.ha ?? 0;
  const widthOf = (s: string) =>
    Array.from(s).reduce((sum, c) => sum + advance(c), 0) / data.resolution;
  const lines = wrapLines(look.text, look.lineWidth * LETTER_EM, widthOf);
  const material = new MeshStandardMaterial({
    color: look.color,
    metalness: look.metalness,
    roughness: ROUGHNESS,
  });
  const shapes = lines.map((line) => {
    if (!line.trim()) return null;
    const geometry = new TextGeometry(line, {
      font,
      size: 1,
      depth: look.depth * DEPTH_EM,
      curveSegments: CURVE_SEGMENTS,
      bevelEnabled: look.bevel > 0,
      bevelThickness: look.bevel * BEVEL.thickness,
      bevelSize: look.bevel * BEVEL.size,
      bevelSegments: BEVEL.segments,
    });
    if (look.slant) geometry.applyMatrix4(new Matrix4().makeShear(0, 0, ITALIC_SHEAR, 0, 0, 0));
    geometry.computeBoundingBox();
    return geometry;
  });
  const widths = shapes.map((g) =>
    g?.boundingBox ? g.boundingBox.max.x - g.boundingBox.min.x : 0,
  );
  const widest = Math.max(0, ...widths);
  const text = new Group();
  shapes.forEach((geometry, i) => {
    const box = geometry?.boundingBox;
    if (!geometry || !box) return;
    const x =
      look.align === "left"
        ? -widest / 2 - box.min.x
        : look.align === "right"
          ? widest / 2 - box.max.x
          : -(box.min.x + box.max.x) / 2;
    const mesh = new Mesh(geometry, material);
    mesh.position.set(x, -i * LINE_HEIGHT_EM, 0);
    text.add(mesh);
  });
  const model = new Group();
  model.add(text);
  fitToSize(text, TEXT_SIZE);
  return model;
}

function dispose(model: Group): void {
  model.traverse((child) => {
    if (!(child instanceof Mesh)) return;
    child.geometry.dispose();
    if (child.material instanceof MeshStandardMaterial) child.material.dispose();
  });
}

const outlinesFor = (key: string) => {
  const [url = "", weight = ""] = key.split(KEY_GAP);
  if (!url) return Promise.reject(new Error(FONT_UNREADABLE));
  return loadOutlines(url, Number(weight));
};

export const Text3D: React.FC<CoreElementProps["core/text-3d"]> = ({
  text,
  fontFamily,
  fontWeight,
  italic,
  depth,
  bevel,
  color,
  metalness,
  align,
  lineWidth,
  motion,
  camera,
  speed,
  lighting,
  background,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const family = (fontFamily && firstFamily(fontFamily)) || HEADING_FONT;
  const file = fontFileFor(family, fontWeight, italic);
  const key = [file?.url ?? "", fontWeight].join(KEY_GAP);
  const loaded = useHeldLoad(`Preparing 3D text in ${family}`, key, outlinesFor);
  const outlines = loaded?.key === key ? loaded.value : null;
  const slant = italic && !file?.italic;
  const model = useMemo(
    () =>
      outlines
        ? textModel(outlines, { text, depth, bevel, color, metalness, align, lineWidth, slant })
        : null,
    [outlines, text, depth, bevel, color, metalness, align, lineWidth, slant],
  );
  useEffect(() => (model ? () => dispose(model) : undefined), [model]);
  const pose = elementPose(motion, camera, { frame, fps, speed, seed: SEED });
  const box = useMemo(() => {
    if (!model) return undefined;
    const size = new Box3().setFromObject(model).getSize(new Vector3());
    return { width: size.x, height: size.y };
  }, [model]);
  if (!model) return null;
  return (
    <Stage3D lighting={lighting} background={background} box={box}>
      <Posed pose={pose}>
        <primitive object={model} />
      </Posed>
    </Stage3D>
  );
};
