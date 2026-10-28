import React, { useLayoutEffect, useMemo } from "react";
import { useCurrentFrame, useVideoConfig } from "remotion";
import {
  AnimationMixer,
  Box3,
  Group,
  LoopOnce,
  LoopRepeat,
  Vector3,
  type AnimationAction,
  type Object3D,
} from "three";
import type { GLTF } from "three/addons/loaders/GLTFLoader.js";
import { clone } from "three/addons/utils/SkeletonUtils.js";
import { GroundShadow } from "#video/elements/scene/card-3d.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";
import { useModel } from "#video/three/load-model.ts";
import { elementPose } from "#video/three/motion.ts";
import { Posed, Stage3D } from "#video/three/stage.tsx";

const MODEL_SIZE = 2.2;
const SEED = 1;
const BOUNDS_SAMPLES = 12;

type Placed = {
  model: Group;
  floor: number;
  width: number;
  mixer: AnimationMixer | null;
  action: AnimationAction | null;
};

function animatedBounds(object: Object3D, mixer: AnimationMixer | null, length: number): Box3 {
  const box = new Box3();
  const samples = mixer && length > 0 ? BOUNDS_SAMPLES : 1;
  for (let i = 0; i < samples; i++) {
    mixer?.setTime((length * i) / Math.max(1, samples - 1));
    object.updateMatrixWorld(true);
    box.union(new Box3().setFromObject(object));
  }
  mixer?.setTime(0);
  return box;
}

function fitBox(object: Object3D, box: Box3, size: number): void {
  const center = box.getCenter(new Vector3());
  const extent = box.getSize(new Vector3());
  const scale = size / (Math.max(extent.x, extent.y, extent.z) || 1);
  object.scale.multiplyScalar(scale);
  object.position.sub(center.multiplyScalar(scale));
}

function place(gltf: GLTF, animation: string, loop: boolean, zoom: number): Placed {
  const scene = clone(gltf.scene);
  const fitted = new Group();
  fitted.add(scene);
  const model = new Group();
  model.add(fitted);
  const wanted = animation.trim().toLowerCase();
  const clip =
    gltf.animations.find((a) => a.name.trim().toLowerCase() === wanted) ?? gltf.animations[0];
  const mixer = clip ? new AnimationMixer(scene) : null;
  const action = clip && mixer ? mixer.clipAction(clip) : null;
  if (action) {
    action.setLoop(loop ? LoopRepeat : LoopOnce, Infinity);
    action.clampWhenFinished = true;
    action.play();
  }
  fitBox(fitted, animatedBounds(fitted, mixer, clip?.duration ?? 0), MODEL_SIZE * zoom);
  const box = animatedBounds(model, mixer, clip?.duration ?? 0);
  const size = box.getSize(new Vector3());
  return { model, floor: box.min.y, width: Math.max(size.x, size.z), mixer, action };
}

const Animated: React.FC<{ placed: Placed; timeS: number }> = ({ placed, timeS }) => {
  useLayoutEffect(() => {
    placed.mixer?.setTime(timeS);
  }, [placed, timeS]);
  return <primitive object={placed.model} />;
};

export const Model3D: React.FC<ElementSettings["model-3d"]> = ({
  src,
  motion,
  camera,
  speed,
  animation,
  animationSpeed,
  loop,
  zoom,
  lighting,
  shadow,
  background,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const gltf = useModel(src || null);
  const placed = useMemo(
    () => (gltf ? place(gltf, animation, loop, zoom) : null),
    [gltf, animation, loop, zoom],
  );
  useLayoutEffect(
    () => () => {
      placed?.mixer?.stopAllAction();
      if (placed?.mixer) placed.mixer.uncacheRoot(placed.mixer.getRoot());
    },
    [placed],
  );
  const pose = elementPose(motion, camera, { frame, fps, speed, seed: SEED });
  if (!placed) return null;
  return (
    <Stage3D lighting={lighting} background={background}>
      {shadow ? <GroundShadow width={placed.width} y={placed.floor} /> : null}
      <Posed pose={pose}>
        <Animated placed={placed} timeS={(frame / fps) * animationSpeed} />
      </Posed>
    </Stage3D>
  );
};
