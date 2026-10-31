import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { useHeldLoad } from "@opsis/runtime/holds.ts";
import { useEffect, useMemo } from "react";
import { Box3, Group, Matrix4, Mesh, MeshStandardMaterial, Vector3 } from "three";
import { TextGeometry } from "three/addons/geometries/TextGeometry.js";
import { Font, type FontData } from "three/addons/loaders/FontLoader.js";
import type { ElementSettings, PreparedTypeface } from "#video/generated/spec.ts";
import { elementPose } from "#video/three/motion.ts";
import { FONT_UNREADABLE, loadOutlines, wrapLines } from "#video/three/outlines.ts";
import { fitToSize } from "#video/three/shapes.ts";
import { Posed, Stage3D } from "#video/three/stage.tsx";

const TEXT_SIZE = 2.6;
const LETTER_EM = 0.6;
const DEPTH_EM = 0.6;
const BEVEL = { thickness: 0.05, size: 0.035, segments: 4 };
const CURVE_SEGMENTS = 10;
const ITALIC_SHEAR = 0.2;
const SEED = 1;

type TextLook = {
  text: string;
  depth: number;
  bevel: number;
  color: string;
  metalness: number;
  align: "center" | "left" | "right";
  lineWidth: number;
  slant: boolean;
  lineHeight: number;
  roughness: number;
};

function textModel(data: FontData, look: TextLook): Group {
  const font = new Font(data);
  function advance(char: string) {
    return (data.glyphs[char] ?? data.glyphs["?"] ?? data.glyphs[" "])?.ha ?? 0;
  }
  function widthOf(s: string) {
    return Array.from(s).reduce((sum, c) => sum + advance(c), 0) / data.resolution;
  }
  const lines = wrapLines(look.text, look.lineWidth * LETTER_EM, widthOf);
  const material = new MeshStandardMaterial({
    color: look.color,
    metalness: look.metalness,
    roughness: look.roughness,
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
    mesh.position.set(x, -i * look.lineHeight, 0);
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

function outlinesFor(url: string) {
  return url ? loadOutlines(url) : Promise.reject(new Error(FONT_UNREADABLE));
}

export function Text3D({
  text,
  fontFamily,
  typeface = "",
  typefaceItalic = false,
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
  size,
  lineHeight,
  roughness,
  lens,
  tilt,
  endFacing,
  motionAmount,
  growIn,
  reverse,
}: ElementSettings["text-3d"] & Partial<PreparedTypeface>) {
  const frame = useFrame();
  const { fps } = useFilm();
  const loaded = useHeldLoad(
    `Preparing 3D text in ${fontFamily || "the heading font"}`,
    typeface,
    outlinesFor,
  );
  const outlines = loaded?.key === typeface ? loaded.value : null;
  const slant = italic && !typefaceItalic;
  const model = useMemo(
    () =>
      outlines
        ? textModel(outlines, {
            text,
            depth,
            bevel,
            color,
            metalness,
            align,
            lineWidth,
            slant,
            lineHeight,
            roughness,
          })
        : null,
    [
      outlines,
      text,
      depth,
      bevel,
      color,
      metalness,
      align,
      lineWidth,
      slant,
      lineHeight,
      roughness,
    ],
  );
  useEffect(() => (model ? () => dispose(model) : undefined), [model]);
  const pose = elementPose(motion, camera, {
    frame,
    fps,
    speed,
    seed: SEED,
    tiltDeg: tilt,
    endFacing,
    amount: motionAmount,
    reverse,
    growIn,
  });
  const box = useMemo(() => {
    if (!model) return undefined;
    const extent = new Box3().setFromObject(model).getSize(new Vector3());
    return { width: extent.x / size, height: extent.y / size };
  }, [model, size]);
  if (!model) return null;
  return (
    <Stage3D lighting={lighting} background={background} box={box} fov={lens}>
      <Posed pose={pose}>
        <primitive object={model} />
      </Posed>
    </Stage3D>
  );
}
