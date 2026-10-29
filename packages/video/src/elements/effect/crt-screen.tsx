import { AbsoluteFill } from "remotion";
import type { ElementSettings } from "#video/generated/spec.ts";
import {
  type CanvasFilterProps,
  makeCanvasFilter,
  makeFilterShader,
  rgbUniform,
} from "#video/lib/transitions/canvas-presentation.tsx";

type CrtScreenProps = ElementSettings["crt-screen"];

const FRAGMENT_SHADER = `#version 300 es
precision highp float;

uniform sampler2D u_scene;
uniform float u_frame;
uniform float u_curvature;
uniform float u_scanlines;
uniform float u_mask_scale;
uniform float u_vignette;
uniform vec2 u_size;
uniform float u_scanline_count;
uniform float u_mask;
uniform float u_glow;
uniform float u_flicker;
uniform vec3 u_bezel;

in vec2 v_uv;
out vec4 outColor;

const float PI = 3.141592653589793;
const float REFERENCE_ASPECT = 16.0 / 9.0;
const float REFERENCE_HEIGHT = 1080.0;

vec2 warpTube(vec2 uv, float k, float aspect) {
  vec2 p = uv * 2.0 - 1.0;
  float x2 = p.x * p.x;
  float y2 = p.y * p.y;
  p.x *= 1.0 + k * 0.11 * (REFERENCE_ASPECT / aspect) * y2;
  p.y *= 1.0 + k * 0.17 * x2;
  p /= 1.0 + k * 0.045;
  return p * 0.5 + 0.5;
}

vec3 phosphorMask(float x, float scale) {
  float sub = mod(floor(x / max(scale, 1.0)), 3.0);
  vec3 lit = vec3(
    step(sub, 0.5),
    step(0.5, sub) * step(sub, 1.5),
    step(1.5, sub)
  );
  return mix(vec3(1.0 - u_mask), vec3(1.0), lit);
}

void main() {
  float aspect = u_size.x / max(u_size.y, 1.0);
  vec2 uv = warpTube(v_uv, u_curvature, aspect);

  vec2 edge =
    smoothstep(vec2(0.0), vec2(0.005), uv) *
    smoothstep(vec2(0.0), vec2(0.005), 1.0 - uv);
  float inside = edge.x * edge.y;

  vec4 base = texture(u_scene, uv);

  vec3 spread = vec3(0.0);
  for (int i = -2; i <= 2; i++) {
    spread += texture(u_scene, uv + vec2(float(i) * 0.0018, 0.0)).rgb;
  }
  spread /= 5.0;
  vec3 bloom = max(spread - 0.5, vec3(0.0));

  vec3 color = base.rgb;
  color *= phosphorMask(v_uv.x * u_size.x * (REFERENCE_HEIGHT / max(u_size.y, 1.0)), u_mask_scale);
  color += bloom * u_glow;

  float scan = mix(1.0, 0.5 + 0.5 * cos(uv.y * u_scanline_count * PI * 2.0), 0.55);
  color *= mix(1.0, scan, clamp(u_scanlines, 0.0, 1.5));

  vec2 vc = uv * 2.0 - 1.0;
  float radius = length(vc * vec2(0.86, 1.0));
  color *= mix(1.0, smoothstep(1.6, 0.35, radius), clamp(u_vignette, 0.0, 1.5));

  color *= 1.0 + u_flicker * sin(u_frame * PI / 30.0);
  color = mix(u_bezel, color, inside);

  outColor = vec4(clamp(color, 0.0, 1.0), max(base.a, 1.0 - inside));
}`;

const shader = makeFilterShader<CrtScreenProps>(
  FRAGMENT_SHADER,
  ({ gl, uniform, passedProps, width, height }) => {
    const {
      curvature,
      scanlines,
      maskScale,
      vignette,
      scanlineCount,
      mask,
      glow,
      flicker,
      bezelColor,
    } = passedProps;
    gl.uniform1f(uniform("u_curvature"), curvature);
    gl.uniform1f(uniform("u_scanlines"), scanlines);
    gl.uniform1f(uniform("u_mask_scale"), maskScale);
    gl.uniform1f(uniform("u_vignette"), vignette);
    gl.uniform1f(uniform("u_scanline_count"), scanlineCount);
    gl.uniform1f(uniform("u_mask"), mask);
    gl.uniform1f(uniform("u_glow"), glow);
    gl.uniform1f(uniform("u_flicker"), flicker);
    gl.uniform3fv(uniform("u_bezel"), rgbUniform(bezelColor));
    gl.uniform2f(uniform("u_size"), width, height);
  },
);

function CrtScreenFallback({ children, scanlines, vignette }: CrtScreenProps & CanvasFilterProps) {
  return (
    <AbsoluteFill>
      <AbsoluteFill style={{ filter: "saturate(1.1) contrast(1.06)" }}>{children}</AbsoluteFill>
      <AbsoluteFill
        style={{
          background:
            "repeating-linear-gradient(to bottom, rgba(0, 0, 0, 0.34) 0px, rgba(0, 0, 0, 0.34) 1.5px, transparent 1.5px, transparent 3px)",
          opacity: Math.min(scanlines, 1),
        }}
      />
      <AbsoluteFill
        style={{
          background:
            "radial-gradient(ellipse 82% 78% at 50% 50%, transparent 45%, rgba(0, 0, 0, 0.72) 100%)",
          opacity: Math.min(vignette, 1),
        }}
      />
    </AbsoluteFill>
  );
}

export const CrtScreen = makeCanvasFilter<CrtScreenProps>({
  shader,
  fallback: CrtScreenFallback,
});
