import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { Easing } from "@opsis/runtime/easing.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import type { ReactNode } from "react";
import { useId } from "react";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "#video/lib/transitions/presentation.ts";

const SHAKE_STOPS = [-9 / 11, -3 / 11, 4 / 11, 8 / 11] as const;

const LENS_ZOOM_EASE_IN = Easing.bezier(0.54, 0, 0.82, 0.8);
const LENS_ZOOM_EASE_OUT = Easing.bezier(0.18, 0.2, 0.46, 1);

type LensZoomProps = ElementSettings["lens-zoom"];

const EXIT_ZOOM_SHARE = 0.9;
const EXIT_BLUR_SHARE = 0.75;
const EXIT_FOV_SHARE = 135 / 160;
const ANTICIPATE_PORTION = 0.35;
const BLUR_SCALE_GAIN = 1;
const BLUR_FADE = 1.5;

function hashed(seed: number, i: number): number {
  let t = Math.imul(i ^ seed, 0x27d4eb2d);
  t ^= t >>> 15;
  t = Math.imul(t, 0x85ebca6b);
  t ^= t >>> 13;
  return ((t >>> 0) / 4294967296) * 2 - 1;
}

function valueNoise(seed: number, t: number): number {
  const i = Math.floor(t);
  const f = t - i;
  const u = f * f * (3 - 2 * f);
  return hashed(seed, i) * (1 - u) + hashed(seed, i + 1) * u;
}

function fovHalf(fov: number): number {
  return (Math.min(fov, 178) / 2) * (Math.PI / 180);
}

function lensMagnification(r: number, fov: number): number {
  if (fov < 0.01) return 1;
  const theta = fovHalf(fov);
  const peak = Math.tan(theta) / theta;
  if (r <= 0) return peak;
  const a = r * theta;
  if (a >= Math.PI / 2 - 1e-3) return 1;
  const rSource = Math.tan(a) / Math.tan(theta);
  if (rSource <= 1e-6) return peak;
  return Math.max(1, r / rSource);
}

function radiusForMagnification(target: number, fov: number): number {
  let lo = 0;
  let hi = 1;
  for (let i = 0; i < 24; i++) {
    const mid = (lo + hi) / 2;
    if (lensMagnification(mid, fov) > target) lo = mid;
    else hi = mid;
  }
  return (lo + hi) / 2;
}

function ringClip(
  rInner: number,
  rOuter: number | null,
  w: number,
  h: number,
  cx: number,
  cy: number,
): string {
  function circle(r: number, clockwise: boolean) {
    const s = clockwise ? 1 : 0;
    return `M ${cx + r} ${cy} A ${r} ${r} 0 1 ${s} ${cx - r} ${cy} A ${r} ${r} 0 1 ${s} ${cx + r} ${cy} Z`;
  }
  const outer = rOuter === null ? `M 0 0 L ${w} 0 L ${w} ${h} L 0 ${h} Z` : circle(rOuter, true);
  return rInner > 0 ? `${outer} ${circle(rInner, false)}` : outer;
}

type HalfProps = {
  scene: ReactNode;
  scale: number;
  fov: number;
  blur: number;
  rotation: number;
  centerX: number;
  centerY: number;
  width: number;
  height: number;
  lensSteps: number;
  blurSamples: number;
  blurScaleGain: number;
  blurFade: number;
};

function TransitionHalf({
  scene,
  scale,
  fov,
  blur,
  rotation,
  centerX,
  centerY,
  width,
  height,
  lensSteps,
  blurSamples,
  blurScaleGain,
  blurFade,
}: HalfProps) {
  const origin = `${centerX}px ${centerY}px`;
  const base = scale / 100;

  const peak = lensMagnification(0, fov);
  const steps = peak <= 1.001 ? 0 : lensSteps;
  const rings = Array.from({ length: steps }, (_, i) => {
    const s = 1 + ((peak - 1) * (i + 0.5)) / steps;
    return { s, r: radiusForMagnification(s, fov) };
  });

  const samples = blur <= 0.05 ? 1 : Math.max(2, Math.round((blurSamples * blur) / 60));
  const weights = Array.from({ length: samples }, (_, j) => (1 - j / samples) ** blurFade);

  const groups: {
    key: number;
    alpha: number;
    copies: { key: string; clip: string; k: number }[];
  }[] = [];
  let running = 0;
  for (let j = samples - 1; j >= 0; j--) {
    running += weights[j];
    const g = samples === 1 ? 1 : 1 + (blur / 100) * blurScaleGain * (j / (samples - 1));
    const halfWidth = (width / 2) * g;
    const copies: { key: string; clip: string; k: number }[] = [];

    copies.push({
      key: "base",
      clip: ringClip(
        rings.length ? rings[0].r * halfWidth : 0,
        null,
        width,
        height,
        centerX,
        centerY,
      ),
      k: g * base,
    });

    for (let i = 0; i < rings.length; i++) {
      const inner = i + 1 < rings.length ? rings[i + 1].r * halfWidth : 0;
      copies.push({
        key: `${i}`,
        clip: ringClip(inner, rings[i].r * halfWidth, width, height, centerX, centerY),
        k: g * rings[i].s * base,
      });
    }

    groups.push({ key: j, alpha: weights[j] / running, copies });
  }

  return (
    <div style={{ position: "absolute", inset: 0 }}>
      {groups.map(({ key, alpha, copies }) => (
        <div key={key} style={{ position: "absolute", inset: 0, opacity: alpha }}>
          {copies.map(({ key: ck, clip, k }) => (
            <div
              key={ck}
              style={{
                position: "absolute",
                inset: 0,
                clipPath: `path("${clip}")`,
                rotate: `${rotation}deg`,
                scale: `${k}`,
                transformOrigin: origin,
              }}
            >
              {scene}
            </div>
          ))}
        </div>
      ))}
    </div>
  );
}

interface LensZoomStageProps extends HalfProps {
  shakeEnvelope: number;
  shakeFrequency: number;
  shakeCyclesPerUnit: number;
  shakeTranslatePx: number;
  shakeRotateDeg: number;
  redScale: number;
  greenScale: number;
  blueScale: number;
  letterboxAspect: number;
  background: string;
}

function LensZoomStage({
  shakeEnvelope,
  shakeFrequency,
  shakeCyclesPerUnit,
  shakeTranslatePx,
  shakeRotateDeg,
  redScale,
  greenScale,
  blueScale,
  letterboxAspect,
  background,
  ...half
}: LensZoomStageProps) {
  const frame = useFrame();
  const { fps } = useFilm();
  const filterId = useId();
  const { width, height, centerX, centerY } = half;

  const core = <TransitionHalf {...half} />;

  const split = redScale !== greenScale || blueScale !== greenScale;
  const channels: { id: string; scale: number; matrix: string }[] = [
    {
      id: "r",
      scale: redScale,
      matrix: "1 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0",
    },
    {
      id: "g",
      scale: greenScale,
      matrix: "0 0 0 0 0  0 1 0 0 0  0 0 0 0 0  0 0 0 1 0",
    },
    {
      id: "b",
      scale: blueScale,
      matrix: "0 0 0 0 0  0 0 0 0 0  0 0 1 0 0  0 0 0 1 0",
    },
  ];

  const composited = split ? (
    <div style={{ position: "absolute", inset: 0, backgroundColor: "#000000" }}>
      {channels.map((c) => (
        <div
          key={c.id}
          style={{
            position: "absolute",
            inset: 0,
            mixBlendMode: "screen",
            filter: `url(#${filterId}-${c.id})`,
            scale: `${c.scale}`,
            transformOrigin: `${centerX}px ${centerY}px`,
          }}
        >
          {core}
        </div>
      ))}
    </div>
  ) : (
    core
  );

  const t = (frame / fps) * shakeFrequency * shakeCyclesPerUnit;
  function octave(seed: number) {
    return valueNoise(seed, t) + 0.45 * valueNoise(seed + 977, t * 2.6);
  }
  const shakeX = octave(11) * shakeTranslatePx * shakeEnvelope;
  const shakeY = octave(23) * shakeTranslatePx * shakeEnvelope;
  const shakeR = octave(41) * shakeRotateDeg * shakeEnvelope;

  const barHeight = letterboxAspect > 0 ? Math.max(0, (height - width / letterboxAspect) / 2) : 0;

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        width,
        height,
        overflow: "hidden",
        backgroundColor: background,
      }}
    >
      {split ? (
        <svg width={0} height={0} style={{ position: "absolute" }} aria-hidden>
          <defs>
            {channels.map((c) => (
              <filter key={c.id} id={`${filterId}-${c.id}`} colorInterpolationFilters="sRGB">
                <feColorMatrix type="matrix" values={c.matrix} />
              </filter>
            ))}
          </defs>
        </svg>
      ) : null}

      <div
        style={{
          position: "absolute",
          inset: 0,
          translate: `${shakeX}px ${shakeY}px`,
          rotate: `${shakeR}deg`,
          transformOrigin: `${centerX}px ${centerY}px`,
        }}
      >
        {composited}
      </div>

      {barHeight > 0 ? (
        <>
          <div
            style={{
              position: "absolute",
              left: 0,
              right: 0,
              top: 0,
              height: barHeight,
              backgroundColor: background,
            }}
          />
          <div
            style={{
              position: "absolute",
              left: 0,
              right: 0,
              bottom: 0,
              height: barHeight,
              backgroundColor: background,
            }}
          />
        </>
      ) : null}
    </div>
  );
}

function LensZoomPresentation({
  children,
  presentationDirection,
  presentationProgress,
  passedProps,
}: TransitionPresentationComponentProps<LensZoomProps>) {
  const { width, height } = useFilm();
  const {
    lensSteps,
    blurSamples,
    shakeAmount,
    shakeTranslatePx,
    colorSplit,
    zoom,
    zoomBlur,
    distortion,
    twist,
    pullBack,
    shakeSpeed,
    shakeRotation,
    fringe,
    letterbox,
    background,
  } = passedProps;
  const inScaleFrom = 100;
  const outScaleFrom = zoom * EXIT_ZOOM_SHARE;
  const outScaleTo = 100;
  const inFovFrom = 0;
  const outFovFrom = distortion * EXIT_FOV_SHARE;
  const outFovTo = 0;
  const inBlurFrom = 0;
  const outBlurFrom = zoomBlur * EXIT_BLUR_SHARE;
  const outBlurTo = 0;
  const inRotationFrom = 0;
  const outRotationFrom = 360 + twist;
  const outRotationTo = 360;
  const anticipatePortion = ANTICIPATE_PORTION;

  const exiting = presentationDirection === "exiting";

  if (exiting ? presentationProgress >= 0.5 : presentationProgress < 0.5) return null;

  const p = exiting
    ? interpolate(presentationProgress, [0, 0.5], [0, 1], { ...CLAMPED, easing: LENS_ZOOM_EASE_IN })
    : interpolate(presentationProgress, [0.5, 1], [0, 1], {
        ...CLAMPED,
        easing: LENS_ZOOM_EASE_OUT,
      });

  const anticipating = exiting && pullBack < inScaleFrom;

  const pFx = anticipating && exiting ? interpolate(p, [anticipatePortion, 1], [0, 1], CLAMPED) : p;

  function lerp(from: number, to: number) {
    return from + (to - from) * pFx;
  }

  const exitScale = anticipating
    ? interpolate(p, [0, anticipatePortion, 1], [inScaleFrom, pullBack, zoom], {
        ...CLAMPED,
        easing: Easing.inOut(Easing.ease),
      })
    : lerp(inScaleFrom, zoom);

  const shakeEnvelope =
    interpolate(
      presentationProgress,
      SHAKE_STOPS.map((s) => 0.5 + s / 2),
      [0, shakeAmount, shakeAmount, 0],
      CLAMPED,
    ) / 100;

  return (
    <LensZoomStage
      scene={children}
      scale={exiting ? exitScale : lerp(outScaleFrom, outScaleTo)}
      fov={exiting ? lerp(inFovFrom, distortion) : lerp(outFovFrom, outFovTo)}
      blur={exiting ? lerp(inBlurFrom, zoomBlur) : lerp(outBlurFrom, outBlurTo)}
      rotation={exiting ? lerp(inRotationFrom, twist) : lerp(outRotationFrom, outRotationTo)}
      centerX={width / 2}
      centerY={height / 2}
      width={width}
      height={height}
      lensSteps={lensSteps}
      blurSamples={blurSamples}
      blurScaleGain={BLUR_SCALE_GAIN}
      blurFade={BLUR_FADE}
      shakeEnvelope={shakeEnvelope}
      shakeFrequency={shakeSpeed}
      shakeCyclesPerUnit={1}
      shakeTranslatePx={shakeTranslatePx}
      shakeRotateDeg={shakeRotation}
      redScale={colorSplit ? 1 + fringe / 100 : 1}
      greenScale={1}
      blueScale={colorSplit ? 1 - fringe / 100 : 1}
      letterboxAspect={letterbox}
      background={background}
    />
  );
}

export function lensZoom(props: LensZoomProps): TransitionPresentation<LensZoomProps> {
  return {
    component: LensZoomPresentation,
    props,
  };
}
