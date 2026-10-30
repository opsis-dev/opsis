import { Easing, interpolateColors, spring } from "remotion";
import { clamp01, lerp, normalize, numberOr } from "#video/common/math.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export const radialBurstLength = 114;
const CYCLE = 120;
const easeInOut = Easing.bezier(0.65, 0, 0.25, 1);
const easeOut = Easing.bezier(0.16, 1, 0.3, 1);

export function getRadialBurstState(
  frame: number,
  {
    segments,
    radius,
    thickness,
    rotation,
    intensity,
    loop,
    width,
    height,
  }: Pick<
    ElementSettings["radial-burst"],
    "segments" | "radius" | "thickness" | "rotation" | "intensity" | "loop"
  > & {
    width: number;
    height: number;
  },
) {
  const w = Math.max(1, width);
  const h = Math.max(1, height);
  const unit = Math.min(w, h) / 720;
  const time = Math.max(0, numberOr(frame, 0));
  const local = loop ? time % CYCLE : time;
  const count = Math.round(Math.min(16, Math.max(4, numberOr(segments, 8))));
  const burstRadius = Math.min(280, Math.max(100, numberOr(radius, 210))) * unit;
  const stroke = Math.min(60, Math.max(8, numberOr(thickness, 30))) * unit;
  const ringRadius = burstRadius * 0.68;
  const assemble = easeInOut(normalize(local, 44, 76));
  const curl = easeInOut(normalize(local, 48, 78));
  const close = easeInOut(normalize(local, 72, 84));
  const anticipate = easeInOut(normalize(local, 86, 96));
  const exit = normalize(local, 96, radialBurstLength) ** 2.6;
  const launch = spring({
    frame: Math.max(0, local - 18),
    fps: 30,
    durationInFrames: 26,
    config: { damping: 13, stiffness: 170, mass: 0.8 },
  });
  const pop = spring({
    frame: Math.max(0, local - 2),
    fps: 30,
    durationInFrames: 16,
    config: { damping: 11, stiffness: 180, mass: 0.65 },
  });
  const ringStroke = lerp(stroke * 0.8, stroke * 1.1, anticipate);
  const exitStroke = lerp(ringStroke, stroke * 2.2, exit);
  const clearRadius = Math.hypot(w, h) / 2 + stroke * 2.2 + 2;
  const orbitRadius = lerp(lerp(28 * unit, burstRadius, launch), ringRadius, assemble);
  const finalRadius = lerp(ringRadius, ringRadius * 0.84, anticipate);
  const strength = Math.min(1.5, Math.max(0, numberOr(intensity, 1)));
  const energy = Math.sin(Math.PI * normalize(local, 18, 84));
  const tilt = easeOut(normalize(local, 18, 48)) * (1 - easeInOut(normalize(local, 96, 114)));

  return {
    frame: local,
    count,
    unit,
    strength,
    energy: Math.max(0, energy) * strength,
    planeScale: 1 - tilt * 0.46 * Math.min(1.5, strength),
    planeAngle: Math.sin(normalize(local, 18, 114) * Math.PI * 1.6) * 32 * strength,
    offsetX: Math.sin(Math.PI * normalize(local, 18, 114)) * 65 * unit * strength,
    offsetY: -Math.sin(Math.PI * normalize(local, 18, 114)) * 32 * unit * strength,
    dotRadius: 37 * unit * pop * (1 - easeOut(normalize(local, 18, 26))),
    dotSquash: 1 - Math.sin(Math.PI * normalize(local, 2, 18)) * 0.16,
    showSegments: local >= 18 && local < 84,
    showRing: local >= 84 && local < radialBurstLength,
    segmentOpacity: easeOut(normalize(local, 18, 23)),
    radius: local < 84 ? orbitRadius : lerp(finalRadius, clearRadius, exit),
    strokeWidth: local < 84 ? lerp(stroke, stroke * 0.8, assemble) : exitStroke,
    segmentLength: lerp(
      12 * unit,
      Math.min(82 * unit, burstRadius * 0.42),
      easeOut(normalize(local, 18, 36)),
    ),
    curl,
    arcAngle: ((Math.PI * 2) / count) * lerp(0.62, 1.002, close),
    rotation: numberOr(rotation, 135) * easeInOut(normalize(local, 34, 84)),
  };
}

function getSegmentPoints({
  radius,
  segmentLength,
  arcAngle,
  curl,
}: Pick<ReturnType<typeof getRadialBurstState>, "radius" | "segmentLength" | "arcAngle" | "curl">) {
  const half = arcAngle / 2;
  const x = radius * Math.cos(half);
  const y = radius * Math.sin(half);
  const tangent = (4 / 3) * Math.tan(arcAngle / 4) * radius;
  const arc = [
    [x, -y],
    [x + tangent * Math.sin(half), -y + tangent * Math.cos(half)],
    [x + tangent * Math.sin(half), y - tangent * Math.cos(half)],
    [x, y],
  ];
  const angle = (curl * Math.PI) / 2;
  return [-0.5, -1 / 6, 1 / 6, 0.5].map((offset, index) => {
    const px = radius + segmentLength * offset * Math.cos(angle);
    const py = segmentLength * offset * Math.sin(angle);
    return [lerp(px, arc[index][0], curl), lerp(py, arc[index][1], curl)];
  });
}

export function getRadialBurstSegmentPath(state: Parameters<typeof getSegmentPoints>[0]) {
  const points = getSegmentPoints(state).map((point) =>
    point.map((value) => value.toFixed(3)).join(" "),
  );
  return `M ${points[0]} C ${points[1]} ${points[2]} ${points[3]}`;
}

export function getRadialBurstRibbonPath(
  state: ReturnType<typeof getRadialBurstState>,
  index: number,
  twist = 110,
) {
  const energy = state.energy;
  const points = getSegmentPoints({
    ...state,
    radius: state.radius * (1 + Math.sin(index * 2.4) * energy * 0.15),
    segmentLength: state.segmentLength * (1 + energy * 2.8),
  });
  const bend =
    (Math.min(180, Math.max(-180, numberOr(twist, 110))) / 110) * energy * 145 * state.unit;
  points[1][1] += bend * 0.55;
  points[2][1] += bend;
  points[3][1] += bend * 1.25;
  points[2][0] += energy * 38 * state.unit;
  const upper: string[] = [];
  const lower: string[] = [];
  function format(x: number, y: number) {
    return `${x.toFixed(3)} ${y.toFixed(3)}`;
  }
  let firstWidth = 0;
  let lastWidth = 0;
  for (let sample = 0; sample <= 36; sample++) {
    const t = sample / 36;
    const q = 1 - t;
    const x =
      q ** 3 * points[0][0] +
      3 * q * q * t * points[1][0] +
      3 * q * t * t * points[2][0] +
      t ** 3 * points[3][0];
    const y =
      q ** 3 * points[0][1] +
      3 * q * q * t * points[1][1] +
      3 * q * t * t * points[2][1] +
      t ** 3 * points[3][1];
    const dx =
      3 * q * q * (points[1][0] - points[0][0]) +
      6 * q * t * (points[2][0] - points[1][0]) +
      3 * t * t * (points[3][0] - points[2][0]);
    const dy =
      3 * q * q * (points[1][1] - points[0][1]) +
      6 * q * t * (points[2][1] - points[1][1]) +
      3 * t * t * (points[3][1] - points[2][1]);
    const length = Math.max(0.001, Math.hypot(dx, dy));
    const ddx =
      6 * q * (points[2][0] - 2 * points[1][0] + points[0][0]) +
      6 * t * (points[3][0] - 2 * points[2][0] + points[1][0]);
    const ddy =
      6 * q * (points[2][1] - 2 * points[1][1] + points[0][1]) +
      6 * t * (points[3][1] - 2 * points[2][1] + points[1][1]);
    const curvature = Math.abs(dx * ddy - dy * ddx) / length ** 3;
    const swell = Math.sin(Math.PI * t) ** 1.4;
    const halfWidth = Math.min(
      (state.strokeWidth / 2) * (1 + energy * (2.4 * swell - 0.65 * t)),
      0.72 / Math.max(curvature, 0.00001),
    );
    if (sample === 0) firstWidth = halfWidth;
    if (sample === 36) lastWidth = halfWidth;
    upper.push(format(x - (dy / length) * halfWidth, y + (dx / length) * halfWidth));
    lower.push(format(x + (dy / length) * halfWidth, y - (dx / length) * halfWidth));
  }
  return `M ${upper.join(" L ")} A ${lastWidth} ${lastWidth} 0 0 0 ${lower[36]} L ${lower.slice(0, -1).toReversed().join(" L ")} A ${firstWidth} ${firstWidth} 0 0 0 ${upper[0]} Z`;
}

export function RadialBurst({
  segments,
  radius,
  thickness,
  rotation,
  intensity,
  twist,
  echoes,
  color,
  accentColor,
  backgroundColor,
  speed,
  loop,
  echoOpacity,
  x,
  y,
}: ElementSettings["radial-burst"]) {
  const frame = useAuthoredFrame();
  const { width, height, fps } = useAuthoredConfig();
  const time = frame * (30 / fps) * Math.max(0, speed);
  const options = {
    segments,
    radius,
    thickness,
    rotation,
    intensity,
    loop,
    width,
    height,
  };
  const state = getRadialBurstState(time, options);
  const path = getRadialBurstSegmentPath(state);
  const echoCount = Math.round(Math.min(5, Math.max(0, echoes)));
  function plane(pose: typeof state) {
    return `translate(${pose.offsetX} ${pose.offsetY}) rotate(${pose.planeAngle}) scale(1 ${pose.planeScale})`;
  }
  function ribbons(pose: typeof state, outline: boolean) {
    return Array.from({ length: pose.count }, (_, index) => (
      <path
        key={index}
        d={getRadialBurstRibbonPath(pose, index, twist)}
        transform={`rotate(${(index * 360) / pose.count + pose.rotation - 90 + Math.sin(index * 1.7) * pose.energy * 16})`}
        fill={
          outline
            ? "none"
            : index % 3 === 0
              ? interpolateColors(clamp01(pose.energy), [0, 1], [color, accentColor])
              : color
        }
        stroke={outline ? color : "none"}
        strokeWidth={1.5 * pose.unit}
      />
    ));
  }

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        overflow: "hidden",
        backgroundColor,
      }}
    >
      <svg
        width="100%"
        height="100%"
        viewBox={`0 0 ${width} ${height}`}
        role="img"
        aria-label="Geometric radial burst"
        style={{ display: "block" }}
      >
        <g transform={`translate(${width * x} ${height * y})`}>
          {state.strength > 0 && state.frame >= 18 && state.frame < radialBurstLength
            ? Array.from({ length: echoCount }, (_, index) => {
                const lag = (echoCount - index) * 2.2;
                const echo = getRadialBurstState(Math.max(18, state.frame - lag), {
                  ...options,
                  loop: false,
                });
                const opacity =
                  echoOpacity *
                  (1 - index / (echoCount + 1)) *
                  Math.min(1, state.strength) *
                  (1 - easeOut(normalize(state.frame, 107, 114)));
                return (
                  <g key={index} transform={plane(echo)} opacity={opacity}>
                    {echo.showSegments ? (
                      ribbons(echo, true)
                    ) : echo.showRing ? (
                      <circle
                        r={echo.radius}
                        fill="none"
                        stroke={index % 2 ? color : accentColor}
                        strokeWidth={echo.strokeWidth * (0.22 + index * 0.07)}
                      />
                    ) : null}
                  </g>
                );
              })
            : null}
          <g transform={plane(state)}>
            {state.dotRadius > 0.001 ? (
              <circle
                r={state.dotRadius}
                fill={color}
                transform={`scale(${1 / state.dotSquash} ${state.dotSquash})`}
              />
            ) : null}
            {state.showSegments && state.strength > 0 ? (
              <g opacity={state.segmentOpacity}>{ribbons(state, false)}</g>
            ) : state.showSegments ? (
              <g
                fill="none"
                stroke={color}
                strokeWidth={state.strokeWidth}
                strokeLinecap="round"
                opacity={state.segmentOpacity}
                transform={`rotate(${state.rotation - 90})`}
              >
                {Array.from({ length: state.count }, (_, index) => (
                  <path key={index} d={path} transform={`rotate(${(index * 360) / state.count})`} />
                ))}
              </g>
            ) : null}
            {state.showRing ? (
              <circle r={state.radius} fill="none" stroke={color} strokeWidth={state.strokeWidth} />
            ) : null}
          </g>
        </g>
      </svg>
    </div>
  );
}
