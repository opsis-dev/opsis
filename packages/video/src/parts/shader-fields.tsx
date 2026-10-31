import { holdRender } from "@opsis/runtime/holds.ts";
import {
  defaultObjectSizing,
  getShaderNoiseTexture,
  GrainGradientShapes,
  grainGradientFragmentShader,
  ShaderFitOptions,
} from "@paper-design/shaders";
import {
  Dithering,
  GodRays,
  getShaderColorFromString,
  MeshGradient,
  PerlinNoise,
  ShaderMount,
  SmokeRing,
  Swirl,
  Warp,
  type DitheringProps,
  type GodRaysProps,
  type MeshGradientProps,
  type PerlinNoiseProps,
  type SmokeRingProps,
  type SwirlProps,
  type WarpProps,
} from "@paper-design/shaders-react";
import { useCallback, useState } from "react";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";

const FILL = { position: "absolute", inset: 0 } as const;

const ORDERED_CLAMP_SHADER = grainGradientFragmentShader.replaceAll(
  "clamp(0., 1., length(",
  "min(1., length(",
);

type Field<P> = Omit<P, "frame" | "ref" | "speed" | "fit" | "width" | "height"> & {
  speed: number;
};

function useShaderClock(label: string, speed: number) {
  const frame = useAuthoredFrame();
  const { fps, width, height } = useAuthoredConfig();
  const [release] = useState(() => holdRender(label));
  const gate = useCallback(
    (element: HTMLDivElement | null) => {
      if (!element) return;
      requestAnimationFrame(() => requestAnimationFrame(() => release()));
    },
    [release],
  );
  return { gate, ms: (frame / fps) * speed * 1000, width, height };
}

export function DitheringField({ speed, ...props }: Field<DitheringProps>) {
  const { gate, ms, width, height } = useShaderClock("shader-dithering", speed);
  return (
    <div ref={gate} style={FILL}>
      <Dithering speed={0} frame={ms} {...props} fit="cover" width={width} height={height} />
    </div>
  );
}

export function GodRaysField({ speed, ...props }: Field<GodRaysProps>) {
  const { gate, ms, width, height } = useShaderClock("shader-god-rays", speed);
  return (
    <div ref={gate} style={FILL}>
      <GodRays speed={0} frame={ms} {...props} fit="cover" width={width} height={height} />
    </div>
  );
}

export function MeshGradientField({ speed, ...props }: Field<MeshGradientProps>) {
  const { gate, ms, width, height } = useShaderClock("shader-mesh-gradient", speed);
  return (
    <div ref={gate} style={FILL}>
      <MeshGradient speed={0} frame={ms} {...props} fit="cover" width={width} height={height} />
    </div>
  );
}

export function PerlinNoiseField({ speed, ...props }: Field<PerlinNoiseProps>) {
  const { gate, ms, width, height } = useShaderClock("shader-perlin-noise", speed);
  return (
    <div ref={gate} style={FILL}>
      <PerlinNoise speed={0} frame={ms} {...props} fit="cover" width={width} height={height} />
    </div>
  );
}

export function SmokeRingField({ speed, ...props }: Field<SmokeRingProps>) {
  const { gate, ms, width, height } = useShaderClock("shader-smoke-ring", speed);
  return (
    <div ref={gate} style={FILL}>
      <SmokeRing speed={0} frame={ms} {...props} fit="cover" width={width} height={height} />
    </div>
  );
}

export function SwirlField({ speed, ...props }: Field<SwirlProps>) {
  const { gate, ms, width, height } = useShaderClock("shader-swirl", speed);
  return (
    <div ref={gate} style={FILL}>
      <Swirl speed={0} frame={ms} {...props} fit="cover" width={width} height={height} />
    </div>
  );
}

export function WarpField({ speed, ...props }: Field<WarpProps>) {
  const { gate, ms, width, height } = useShaderClock("shader-warp", speed);
  return (
    <div ref={gate} style={FILL}>
      <Warp speed={0} frame={ms} {...props} fit="cover" width={width} height={height} />
    </div>
  );
}

export type GrainGradientFieldProps = {
  speed: number;
  colors: string[];
  colorBack: string;
  softness: number;
  intensity: number;
  noise: number;
  shape: keyof typeof GrainGradientShapes;
  scale?: number;
  rotation?: number;
  offsetX?: number;
  offsetY?: number;
};

export function GrainGradientField({
  speed,
  colors,
  colorBack,
  softness,
  intensity,
  noise,
  shape,
  scale = defaultObjectSizing.scale,
  rotation = defaultObjectSizing.rotation,
  offsetX = defaultObjectSizing.offsetX,
  offsetY = defaultObjectSizing.offsetY,
}: GrainGradientFieldProps) {
  const { gate, ms, width, height } = useShaderClock("shader-grain-gradient", speed);
  return (
    <div ref={gate} style={FILL}>
      <ShaderMount
        speed={0}
        frame={ms}
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
          u_fit: ShaderFitOptions.cover,
          u_scale: scale,
          u_rotation: rotation,
          u_offsetX: offsetX,
          u_offsetY: offsetY,
          u_originX: defaultObjectSizing.originX,
          u_originY: defaultObjectSizing.originY,
          u_worldWidth: defaultObjectSizing.worldWidth,
          u_worldHeight: defaultObjectSizing.worldHeight,
        }}
        width={width}
        height={height}
      />
    </div>
  );
}
