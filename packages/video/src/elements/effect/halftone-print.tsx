import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { useAuthoredConfig } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import {
  type CanvasFilterProps,
  makeCanvasFilter,
  makeFilterShader,
  rgbUniform,
} from "#video/lib/transitions/canvas-presentation.tsx";

type NormalizedProps = {
  cell: number;
  angle: number;
  shift: number;
  paper: string;
  inks: [string, string, string];
};

const FRAGMENT_SHADER = `#version 300 es
precision highp float;

uniform sampler2D u_scene;
uniform float u_aspect;
uniform float u_cell;
uniform float u_angle;
uniform float u_shift;
uniform vec3 u_paper;
uniform vec3 u_cyan_absorb;
uniform vec3 u_magenta_absorb;
uniform vec3 u_yellow_absorb;

in vec2 v_uv;
out vec4 outColor;


vec2 toSquare(vec2 uv) {
  return vec2(uv.x * u_aspect, uv.y);
}

vec2 fromSquare(vec2 p) {
  return vec2(p.x / u_aspect, p.y);
}

float screenInk(float angle, vec2 drift, vec3 pick) {
  vec2 p = toSquare(v_uv);
  float s = sin(angle);
  float c = cos(angle);
  vec2 r = vec2(p.x * c - p.y * s, p.x * s + p.y * c);

  vec2 center = (floor(r / u_cell) + 0.5) * u_cell;
  vec2 unrotated =
    vec2(center.x * c + center.y * s, -center.x * s + center.y * c);

  vec4 texel = texture(u_scene, fromSquare(unrotated + drift));
  vec3 straight = texel.rgb / max(texel.a, 0.004);
  float ink = clamp(1.0 - dot(straight, pick), 0.0, 1.0) * texel.a;

  float radius = sqrt(ink) * u_cell * 0.72;
  float aa = u_cell * 0.045;
  float coverage = 1.0 - smoothstep(radius - aa, radius + aa, length(r - center));
  return coverage * step(0.002, ink);
}

void main() {
  vec4 source = texture(u_scene, v_uv);

  float cyan =
    screenInk(u_angle + 0.2618, u_shift * vec2(1.0, 0.0), vec3(1.0, 0.0, 0.0));
  float magenta =
    screenInk(u_angle + 1.3090, u_shift * vec2(-0.5, 0.866), vec3(0.0, 1.0, 0.0));
  float yellow =
    screenInk(u_angle, u_shift * vec2(-0.5, -0.866), vec3(0.0, 0.0, 1.0));

  vec3 color = u_paper;
  color *= 1.0 - cyan * u_cyan_absorb;
  color *= 1.0 - magenta * u_magenta_absorb;
  color *= 1.0 - yellow * u_yellow_absorb;

  outColor = vec4(color * source.a, source.a);
}`;

const shader = makeFilterShader<NormalizedProps>(
  FRAGMENT_SHADER,
  ({ gl, uniform, passedProps }) => {
    const { cell, angle, shift, paper, inks } = passedProps;
    function absorb(ink: string) {
      return rgbUniform(ink).map((c) => 1 - c);
    }
    gl.uniform3fv(uniform("u_cyan_absorb"), absorb(inks[0]));
    gl.uniform3fv(uniform("u_magenta_absorb"), absorb(inks[1]));
    gl.uniform3fv(uniform("u_yellow_absorb"), absorb(inks[2]));
    gl.uniform1f(uniform("u_cell"), cell);
    gl.uniform1f(uniform("u_angle"), angle);
    gl.uniform1f(uniform("u_shift"), shift);
    gl.uniform3fv(uniform("u_paper"), rgbUniform(paper));
  },
);

function HalftonePrintFallback({ children }: NormalizedProps & CanvasFilterProps) {
  return <AbsoluteFill>{children}</AbsoluteFill>;
}

const HalftonePrintCanvas = makeCanvasFilter<NormalizedProps>({
  shader,
  fallback: HalftonePrintFallback,
});

export function HalftonePrint({
  dotSize,
  angle,
  misregistration,
  paperTint,
  cyanInk,
  magentaInk,
  yellowInk,
  children,
}: ElementSettings["halftone-print"] & CanvasFilterProps) {
  const { height } = useAuthoredConfig();

  return (
    <HalftonePrintCanvas
      cell={Math.max(dotSize, 1) / height}
      angle={(angle * Math.PI) / 180}
      shift={misregistration / height}
      paper={paperTint}
      inks={[cyanInk, magentaInk, yellowInk]}
    >
      {children}
    </HalftonePrintCanvas>
  );
}
