import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { spring } from "@opsis/runtime/spring.ts";
import { CLAMPED } from "#video/core/anim.ts";
import type { StepsLook } from "#video/elements/scene/steps-timeline/look.ts";
import { getSafeAreaPadding } from "#video/lib/layout/layout.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

type TimelineStep = {
  title: string;
  description?: string;
};

export type TimelineStepsProps = {
  steps: TimelineStep[];
  title?: string;
  eyebrow?: string;
  backgroundColor?: string;
  accentColor: string;
  look: StepsLook;
  holdSeconds?: number;
  speed?: number;
};

const T = {
  title: 0,
  start: 0.5,
  dwell: 0.78,
  travel: 0.42,
  exitFor: 0.42,
} as const;

function CheckGlyph({ size, color, progress }: { size: number; color: string; progress: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path
        d="M4.8 12.6l4.9 4.9 9.5-10.4"
        stroke={color}
        strokeWidth={2.8}
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeDasharray={22}
        strokeDashoffset={22 * (1 - progress)}
      />
    </svg>
  );
}

export function TimelineSteps({
  steps,
  title,
  eyebrow,
  backgroundColor,
  accentColor,
  look,
  holdSeconds,
  speed = 1,
}: TimelineStepsProps) {
  const frame = useFrame();
  const { fps, width, height } = useFilm();
  const palette = look.palette;
  const safe = getSafeAreaPadding({ width, height });

  const now = (frame / fps) * speed;
  function at(seconds: number) {
    return (seconds * fps) / speed;
  }
  function ease(from: number, to: number, easing = EASING.enter) {
    return interpolate(frame, [at(from), at(to)], [0, 1], { easing, ...CLAMPED });
  }

  const stage = {
    x: safe.paddingLeft,
    y: safe.paddingTop,
    w: width - safe.paddingLeft - safe.paddingRight,
    h: height - safe.paddingTop - safe.paddingBottom,
  };
  const portrait = height > width;
  const u = Math.min(stage.w / 1120, stage.h / 620);
  const list = steps.slice(0, 5);
  const count = list.length;

  function arrival(index: number) {
    return T.start + index * (T.dwell + T.travel);
  }
  function completion(index: number) {
    return arrival(index) + T.dwell;
  }
  const finished = completion(count - 1);

  const node = 60 * u;
  const headerH = title || eyebrow ? (title ? 108 * u : 44 * u) : 0;
  const railInset = portrait ? node / 2 + 4 * u : stage.w / count / 2;
  const railStart = portrait ? headerH + node * 0.9 : railInset;
  const railEnd = portrait ? stage.h - node * 0.6 : stage.w - railInset;
  const railSpan = Math.max(railEnd - railStart, 1);
  const hasDescription = list.some((step) => step.description);
  const copyDrop = node * 0.78 + (hasDescription ? 116 * u : 48 * u);
  const railAcross = portrait
    ? node / 2
    : headerH + Math.max(node / 2, (stage.h - headerH - (copyDrop + node / 2)) / 2 + node / 2);

  function nodeAt(index: number) {
    return railStart + (count === 1 ? railSpan / 2 : (index / (count - 1)) * railSpan);
  }

  const headIndex = (() => {
    if (now <= arrival(0)) {
      return 0;
    }
    for (let index = 0; index < count - 1; index += 1) {
      if (now < completion(index)) {
        return index;
      }
      if (now < arrival(index + 1)) {
        return interpolate(now, [completion(index), arrival(index + 1)], [index, index + 1], {
          easing: EASING.editorial,
          ...CLAMPED,
        });
      }
    }
    return count - 1;
  })();
  const headPos = nodeAt(headIndex);
  const railDrawn = ease(T.start - 0.35, T.start + 0.15);

  const titleIn = ease(T.title, T.title + 0.55);

  const exit =
    holdSeconds === undefined
      ? 0
      : interpolate(frame, [at(holdSeconds), at(holdSeconds + T.exitFor)], [0, 1], {
          easing: EASING.exit,
          ...CLAMPED,
        });

  return (
    <div
      style={{
        width,
        height,
        background: backgroundColor ?? palette.page,
        fontFamily: look.font,
        position: "relative",
        overflow: "hidden",
      }}
    >
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: `radial-gradient(ellipse 80% 50% at 50% ${
            portrait ? 30 : 40
          }%, color-mix(in srgb, ${accentColor} ${8 * look.glow}%, transparent), transparent 70%)`,
        }}
      />

      <div
        style={{
          position: "absolute",
          left: stage.x,
          top: stage.y,
          width: stage.w,
          height: stage.h,
          opacity: 1 - exit,
          translate: `0 ${exit * 30 * u}px`,
        }}
      >
        {eyebrow ? (
          <div
            style={{
              position: "absolute",
              left: 0,
              top: 0,
              color: accentColor,
              fontSize: 21 * u,
              fontWeight: 600,
              letterSpacing: look.eyebrowCaps ? "0.12em" : undefined,
              textTransform: look.eyebrowCaps ? "uppercase" : "none",
              opacity: titleIn,
              translate: `0 ${(1 - titleIn) * 10 * u}px`,
            }}
          >
            {eyebrow}
          </div>
        ) : null}
        {title ? (
          <h2
            style={{
              position: "absolute",
              left: 0,
              top: eyebrow ? 36 * u : 0,
              margin: 0,
              color: palette.fg,
              fontSize: 62 * look.textSize * u,
              fontWeight: 700,
              lineHeight: 1.05,
              letterSpacing: "-0.02em",
              opacity: titleIn,
              translate: `0 ${(1 - titleIn) * 16 * u}px`,
            }}
          >
            {title}
          </h2>
        ) : null}

        <div
          style={{
            position: "absolute",
            left: portrait ? railAcross - 1.5 * u : railStart,
            top: portrait ? railStart : railAcross - 1.5 * u,
            width: portrait ? 3 * u : railSpan,
            height: portrait ? railSpan : 3 * u,
            background: palette.border,
            borderRadius: 999,
            transformOrigin: portrait ? "center top" : "left center",
            transform: portrait ? `scaleY(${railDrawn})` : `scaleX(${railDrawn})`,
          }}
        />
        <div
          style={{
            position: "absolute",
            left: portrait ? railAcross - 1.5 * u : railStart,
            top: portrait ? railStart : railAcross - 1.5 * u,
            width: portrait ? 3 * u : Math.max(headPos - railStart, 0),
            height: portrait ? Math.max(headPos - railStart, 0) : 3 * u,
            background: accentColor,
            borderRadius: 999,
            boxShadow: `0 0 ${14 * u}px color-mix(in srgb, ${accentColor} 53%, transparent)`,
            opacity: railDrawn,
          }}
        />

        {list.map((step, index) => {
          const landed = ease(arrival(index), arrival(index) + 0.3);
          const work = interpolate(now, [arrival(index), completion(index)], [0, 1], {
            easing: EASING.editorial,
            ...CLAMPED,
          });
          const check = interpolate(now, [completion(index), completion(index) + 0.32], [0, 1], {
            easing: EASING.enter,
            ...CLAMPED,
          });
          const active = landed * (1 - check * 0.55);
          const pop = spring({
            frame: frame - at(arrival(index)),
            fps,
            config: { damping: 12, stiffness: 200, mass: 0.6 },
          });
          const along = nodeAt(index);
          const copyIn = ease(arrival(index) + 0.08, arrival(index) + 0.5);

          const ringRadius = node / 2 - 2 * u;
          const ringLength = 2 * Math.PI * ringRadius;

          return (
            <div key={step.title}>
              <div
                style={{
                  position: "absolute",
                  left: (portrait ? railAcross : along) - node / 2,
                  top: (portrait ? along : railAcross) - node / 2,
                  width: node,
                  height: node,
                  borderRadius: "50%",
                  background:
                    check > 0.4
                      ? `color-mix(in srgb, ${accentColor} 13%, transparent)`
                      : landed > 0.1
                        ? accentColor
                        : palette.window,
                  border: `${2 * u}px solid ${landed > 0.1 ? accentColor : palette.border}`,
                  boxShadow: `0 0 ${34 * u * active}px color-mix(in srgb, ${accentColor} 40%, transparent)`,
                  display: "grid",
                  placeItems: "center",
                  scale: `${interpolate(railDrawn, [0, 1], [0.75, 1]) + pop * 0.06}`,
                  color: check > 0.4 ? accentColor : landed > 0.1 ? palette.page : palette.dim,
                  fontSize: 26 * u,
                  fontWeight: 700,
                  fontVariantNumeric: "tabular-nums",
                }}
              >
                {check > 0.05 ? (
                  <CheckGlyph size={30 * u} color={accentColor} progress={check} />
                ) : (
                  index + 1
                )}
              </div>

              <svg
                width={node + 16 * u}
                height={node + 16 * u}
                style={{
                  position: "absolute",
                  left: (portrait ? railAcross : along) - (node + 16 * u) / 2,
                  top: (portrait ? along : railAcross) - (node + 16 * u) / 2,
                  opacity: landed * (1 - check),
                  transform: "rotate(-90deg)",
                }}
              >
                <circle
                  cx={(node + 16 * u) / 2}
                  cy={(node + 16 * u) / 2}
                  r={ringRadius + 6 * u}
                  fill="none"
                  stroke={accentColor}
                  strokeWidth={2.4 * u}
                  strokeLinecap="round"
                  strokeDasharray={ringLength}
                  strokeDashoffset={ringLength * (1 - work)}
                />
              </svg>

              <div
                style={{
                  position: "absolute",
                  left: portrait ? railAcross + node * 0.85 : along - stage.w / count / 2 + 10 * u,
                  top: portrait ? along - node * 0.34 : railAcross + node * 0.78,
                  width: portrait ? stage.w - railAcross - node * 0.85 : stage.w / count - 20 * u,
                  opacity: copyIn,
                  translate: `0 ${(1 - copyIn) * 14 * u}px`,
                }}
              >
                <div
                  style={{
                    color: check > 0.4 ? palette.dim : palette.fg,
                    fontSize: 30 * u,
                    fontWeight: 600,
                    lineHeight: 1.15,
                    letterSpacing: "-0.01em",
                  }}
                >
                  {step.title}
                </div>
                {step.description ? (
                  <div
                    style={{
                      marginTop: 9 * u,
                      color: palette.faint,
                      fontSize: 21 * u,
                      lineHeight: 1.35,
                    }}
                  >
                    {step.description}
                  </div>
                ) : null}
              </div>
            </div>
          );
        })}

        <div
          style={{
            position: "absolute",
            left: (portrait ? railAcross : headPos) - 7 * u,
            top: (portrait ? headPos : railAcross) - 7 * u,
            width: 14 * u,
            height: 14 * u,
            borderRadius: "50%",
            background: accentColor,
            boxShadow: `0 0 ${18 * u}px ${accentColor}`,
            opacity: railDrawn * (now >= finished ? 0 : Math.abs(headIndex % 1) > 0.02 ? 1 : 0),
          }}
        />
      </div>
    </div>
  );
}
