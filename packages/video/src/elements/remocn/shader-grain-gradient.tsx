import {
  defaultObjectSizing,
  getShaderNoiseTexture,
  GrainGradientShapes,
  grainGradientFragmentShader,
  ShaderFitOptions,
} from "@paper-design/shaders";
import {
  getShaderColorFromString,
  ShaderMount,
  type GrainGradientProps,
} from "@paper-design/shaders-react";
import { useCallback, useState } from "react";
import {
  continueRender,
  delayRender,
} from "remotion";
import { useAuthoredConfig, useAuthoredFrame } from "../../core/authored-clock";

const NEUTRAL_COLORS = ["#3a3a52", "#4a4a68", "#5a5a7e"];

const ORDERED_CLAMP_SHADER = grainGradientFragmentShader.replaceAll(
  "clamp(0., 1., length(",
  "min(1., length(",
);

export interface ShaderGrainGradientProps
  extends Omit<GrainGradientProps, "frame" | "ref"> {}

export function ShaderGrainGradient({
  speed = 1,
  colors = NEUTRAL_COLORS,
  colorBack = "#12121a",
  softness = 0.6,
  intensity = 0.2,
  noise = 0.15,
  shape = "corners",
  scale = defaultObjectSizing.scale,
  rotation = defaultObjectSizing.rotation,
  originX = defaultObjectSizing.originX,
  originY = defaultObjectSizing.originY,
  offsetX = defaultObjectSizing.offsetX,
  offsetY = defaultObjectSizing.offsetY,
  worldWidth = defaultObjectSizing.worldWidth,
  worldHeight = defaultObjectSizing.worldHeight,
  fit = "cover",
  className,
  ...rest
}: ShaderGrainGradientProps) {
  const frame = useAuthoredFrame();
  const { fps, width, height } = useAuthoredConfig();

  const [handle] = useState(() => delayRender("shader-grain-gradient"));
  const gate = useCallback(
    (element: HTMLDivElement | null) => {
      if (!element) return;
      requestAnimationFrame(() =>
        requestAnimationFrame(() => continueRender(handle)),
      );
    },
    [handle],
  );

  return (
    <div
      ref={gate}
      className={className}
      style={{ position: "absolute", inset: 0 }}
    >
      <ShaderMount
        speed={0}
        frame={(frame / fps) * speed * 1000}
        fragmentShader={ORDERED_CLAMP_SHADER}
        uniforms={{
          u_colorBack: getShaderColorFromString(colorBack),
          u_colors: colors.map(getShaderColorFromString),
          u_colorsCount: colors.length,
          u_softness: softness,
          u_intensity: intensity,
          u_noise: noise,
          u_shape: GrainGradientShapes[shape],
          u_noiseTexture: getShaderNoiseTexture(),
          u_fit: ShaderFitOptions[fit],
          u_scale: scale,
          u_rotation: rotation,
          u_offsetX: offsetX,
          u_offsetY: offsetY,
          u_originX: originX,
          u_originY: originY,
          u_worldWidth: worldWidth,
          u_worldHeight: worldHeight,
        }}
        width={width}
        height={height}
        {...rest}
      />
    </div>
  );
}
