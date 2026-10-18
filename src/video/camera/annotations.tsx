import React from "react";
import { Easing, interpolate } from "remotion";
import type { AnnotationKind } from "../../shared/compile/recording";
import { TEXT_COLOR } from "../../shared/elements/colors";
import { MAIN_FONT } from "../../shared/fonts";
import { percent } from "../../shared/format";
import { rectCenter, type Viewport } from "../../shared/schema/actions";
import { fadeWindow, isFadedOut } from "../core/anim";
import type { Annotation } from "./action-log";
import { useDesignScale } from "./lib/design-scale";

const FADE = {
  in: 0.18,
  out: 0.18,
  easeIn: Easing.out(Easing.cubic),
  easeOut: Easing.in(Easing.cubic),
};

const POP_SCALE = 1.2;
const SPOTLIGHT_RADIUS = 12;
const HIGHLIGHT = { radius: 10, border: 3, fill: "12%", growFrom: 0.96 } as const;
const TOOLTIP = { rise: 8, fontSize: 18, padY: 8, padX: 14, radius: 10, arrow: 8 } as const;
const CALLOUT = {
  width: 140,
  height: 110,
  stroke: 5,
  headShownFrom: 0.9,
  textLeft: 120,
  textBottom: 96,
  fontSize: 30,
} as const;

type MarkProps = {
  annotation: Annotation;
  opacity: number;
  designScale: number;
  counter: number;
  viewport: Viewport;
};

const pct = (v: number, of: number): string => percent(v / of);

const boxOf = ({ annotation, viewport }: MarkProps) => ({
  left: pct(annotation.rect.x - annotation.padding, viewport.width),
  top: pct(annotation.rect.y - annotation.padding, viewport.height),
  width: pct(annotation.rect.w + annotation.padding * 2, viewport.width),
  height: pct(annotation.rect.h + annotation.padding * 2, viewport.height),
});

const Spotlight: React.FC<MarkProps> = (props) => (
  <div
    style={{
      position: "absolute",
      ...boxOf(props),
      borderRadius: SPOTLIGHT_RADIUS * props.designScale,
      boxShadow: `0 0 0 9999px ${props.annotation.color}`,
      opacity: props.opacity,
    }}
  />
);

const Highlight: React.FC<MarkProps> = (props) => {
  const { annotation, opacity, designScale: k, counter } = props;
  return (
    <div
      style={{
        position: "absolute",
        ...boxOf(props),
        borderRadius: HIGHLIGHT.radius * k,
        border: `${HIGHLIGHT.border * k * counter}px solid ${annotation.color}`,
        background: `color-mix(in srgb, ${annotation.color} ${HIGHLIGHT.fill}, transparent)`,
        opacity,
        transform: `scale(${interpolate(opacity, [0, 1], [HIGHLIGHT.growFrom, 1])})`,
      }}
    />
  );
};

const Tooltip: React.FC<MarkProps> = ({
  annotation,
  opacity,
  designScale: k,
  counter,
  viewport,
}) => (
  <div
    style={{
      position: "absolute",
      left: pct(rectCenter(annotation.rect).x, viewport.width),
      top: pct(annotation.rect.y - annotation.padding, viewport.height),
      transform: `translate(-50%, -100%) translateY(${(1 - opacity) * TOOLTIP.rise * k}px) scale(${counter * POP_SCALE})`,
      transformOrigin: "50% 100%",
      opacity,
    }}
  >
    <div
      style={{
        background: annotation.color,
        color: TEXT_COLOR,
        fontFamily: MAIN_FONT,
        fontWeight: 600,
        fontSize: TOOLTIP.fontSize * k,
        padding: `${TOOLTIP.padY * k}px ${TOOLTIP.padX * k}px`,
        borderRadius: TOOLTIP.radius * k,
        whiteSpace: "nowrap",
        boxShadow: `0 ${6 * k}px ${18 * k}px rgba(0,0,0,0.25)`,
      }}
    >
      {annotation.text}
    </div>
    <div
      style={{
        width: 0,
        height: 0,
        margin: "0 auto",
        borderLeft: `${TOOLTIP.arrow * k}px solid transparent`,
        borderRight: `${TOOLTIP.arrow * k}px solid transparent`,
        borderTop: `${TOOLTIP.arrow * k}px solid ${annotation.color}`,
      }}
    />
  </div>
);

const Callout: React.FC<MarkProps> = ({
  annotation,
  opacity,
  designScale: k,
  counter,
  viewport,
}) => (
  <div
    style={{
      position: "absolute",
      left: pct(annotation.rect.x + annotation.rect.w, viewport.width),
      top: pct(annotation.rect.y, viewport.height),
      transform: `scale(${counter * POP_SCALE})`,
      transformOrigin: "0 100%",
      opacity,
    }}
  >
    <svg
      width={CALLOUT.width * k}
      height={CALLOUT.height * k}
      viewBox={`0 0 ${CALLOUT.width} ${CALLOUT.height}`}
      style={{ position: "absolute", left: 0, bottom: 0, overflow: "visible" }}
    >
      <path
        d="M130 10 C 90 20, 40 40, 12 96"
        fill="none"
        stroke={annotation.color}
        strokeWidth={CALLOUT.stroke}
        strokeLinecap="round"
        pathLength={1}
        strokeDasharray={1}
        strokeDashoffset={1 - opacity}
      />
      <path
        d="M4 82 L12 98 L28 88"
        fill="none"
        stroke={annotation.color}
        strokeWidth={CALLOUT.stroke}
        strokeLinecap="round"
        strokeLinejoin="round"
        opacity={opacity > CALLOUT.headShownFrom ? 1 : 0}
      />
    </svg>
    <div
      style={{
        position: "absolute",
        left: CALLOUT.textLeft * k,
        bottom: CALLOUT.textBottom * k,
        color: annotation.color,
        fontFamily: MAIN_FONT,
        fontWeight: 800,
        fontSize: CALLOUT.fontSize * k,
        whiteSpace: "nowrap",
        transform: "rotate(-4deg)",
      }}
    >
      {annotation.text}
    </div>
  </div>
);

const MARKS: Record<AnnotationKind, React.FC<MarkProps>> = {
  spotlight: Spotlight,
  highlight: Highlight,
  tooltip: Tooltip,
  callout: Callout,
};

export const Annotations: React.FC<{
  annotations: Annotation[];
  viewport: Viewport;
  timeS: number;
  cameraScale: number;
}> = ({ annotations, viewport, timeS, cameraScale }) => {
  const k = useDesignScale();
  const counter = 1 / Math.max(1, cameraScale);

  return (
    <div style={{ position: "absolute", inset: 0, pointerEvents: "none" }}>
      {annotations.map((annotation) => {
        const opacity = fadeWindow(timeS, annotation.fromS, annotation.toS, FADE);
        if (isFadedOut(opacity)) return null;
        const Mark = MARKS[annotation.kind];
        return (
          <Mark
            key={annotation.id}
            annotation={annotation}
            opacity={opacity}
            designScale={k}
            counter={counter}
            viewport={viewport}
          />
        );
      })}
    </div>
  );
};
