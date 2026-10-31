import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { useLayoutEffect, useRef } from "react";
import type { ElementSettings } from "#video/generated/spec.ts";
import { createQuadSurface } from "#video/lib/shaders/quad.ts";
import { parseColor } from "#video/lib/ui-kit/color.ts";

const MAX_COLOURS = 4;

const FRAGMENT = `#version 300 es
precision highp float;
in vec2 v_uv;
out vec4 outColor;
uniform vec2 uResolution;
uniform float uProgress;
uniform float uSeed;
uniform vec3 uColors[${MAX_COLOURS}];
uniform int uCount;

float hash(float n) {
  return fract(sin(n * 12.9898 + uSeed * 78.233) * 43758.5453);
}

float valueNoise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  vec2 u = f * f * (3.0 - 2.0 * f);
  float a = hash(i.x + i.y * 57.0);
  float b = hash(i.x + 1.0 + i.y * 57.0);
  float c = hash(i.x + (i.y + 1.0) * 57.0);
  float d = hash(i.x + 1.0 + (i.y + 1.0) * 57.0);
  return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

float fbm(vec2 p) {
  float sum = 0.0;
  float amp = 0.5;
  for (int i = 0; i < 4; i++) {
    sum += valueNoise(p) * amp;
    p *= 2.03;
    amp *= 0.5;
  }
  return sum;
}

void main() {
  float aspect = uResolution.x / uResolution.y;
  vec2 p = vec2(v_uv.x * aspect, v_uv.y);
  vec3 tint = vec3(0.0);
  float share = 0.0;
  float cover = 0.0;
  for (int i = 0; i < ${MAX_COLOURS}; i++) {
    if (i >= uCount) break;
    float fi = float(i);
    float lag = fi * 0.12;
    float t = clamp((uProgress - lag) / (1.0 - lag * 0.5), 0.0, 1.0);
    float bloom = smoothstep(0.0, 0.45, t) * (1.0 - smoothstep(0.55, 1.0, t));
    vec2 from = vec2(hash(fi + 1.0) < 0.5 ? -0.2 : aspect + 0.2, hash(fi + 2.0));
    vec2 to = vec2(aspect * hash(fi + 3.0), hash(fi + 4.0));
    vec2 centre = mix(from, to, smoothstep(0.0, 1.0, t));
    vec2 warp = vec2(
      fbm(p * 1.6 + vec2(fi * 3.1, uProgress * 1.5)),
      fbm(p * 1.6 + vec2(uProgress * 1.2, fi * 5.7))
    ) - 0.5;
    float d = length(p + warp * 0.35 - centre);
    float radius = mix(0.35, 0.95, hash(fi + 5.0)) * (0.6 + 0.6 * bloom);
    float glow = exp(-(d * d) / (radius * radius)) * bloom;
    float lead = glow * glow * glow;
    tint += uColors[i] * lead;
    share += lead;
    cover = 1.0 - (1.0 - cover) * (1.0 - glow);
  }
  vec3 colour = share > 0.000001 ? tint / share : vec3(0.0);
  outColor = vec4(colour * cover, cover);
}`;

type Surface = ReturnType<typeof createQuadSurface>;

type Leak = {
  width: number;
  height: number;
  progress: number;
  seed: number;
  colors: string[];
};

function drawLeak(surface: Surface, { width, height, progress, seed, colors }: Leak) {
  const { gl, uniform } = surface;
  surface.begin(width, height);
  gl.uniform2f(uniform("uResolution"), width, height);
  gl.uniform1f(uniform("uProgress"), progress);
  gl.uniform1f(uniform("uSeed"), seed);
  const shown = colors.slice(0, MAX_COLOURS);
  const rgb = new Float32Array(MAX_COLOURS * 3);
  shown.forEach((colour, i) => {
    const { r, g, b } = parseColor(colour);
    rgb.set([r, g, b], i * 3);
  });
  gl.uniform3fv(uniform("uColors"), rgb);
  gl.uniform1i(uniform("uCount"), shown.length);
  surface.draw();
}

export function LightLeakElement({ seed, colors, strength, blend }: ElementSettings["light-leak"]) {
  const frame = useFrame();
  const { width, height, durationInFrames } = useFilm();
  const canvas = useRef<HTMLCanvasElement>(null);
  const surface = useRef<Surface | null>(null);
  const progress = durationInFrames > 1 ? frame / (durationInFrames - 1) : 1;
  useLayoutEffect(() => {
    const target = canvas.current;
    if (!target) return;
    const made = createQuadSurface(target, FRAGMENT);
    surface.current = made;
    return () => {
      made.cleanup();
      surface.current = null;
    };
  }, []);
  useLayoutEffect(() => {
    const drawn = surface.current;
    if (drawn) drawLeak(drawn, { width, height, progress, seed, colors });
  }, [width, height, progress, seed, colors]);
  return (
    <AbsoluteFill style={{ opacity: strength / 100, mixBlendMode: blend }}>
      <canvas
        ref={canvas}
        width={width}
        height={height}
        style={{ width: "100%", height: "100%" }}
      />
    </AbsoluteFill>
  );
}
