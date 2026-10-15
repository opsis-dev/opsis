import React from "react";
import { Easing, interpolate } from "remotion";
import type { AnnotationKind } from "../../shared/compile/recording";
import { useDesignScale } from "./WindowFrame";

export type Annotation = {
  kind: AnnotationKind;
  rect: { x: number; y: number; w: number; h: number };
  fromS: number;
  toS: number;
  text: string;
  color: string;
  padding: number;
};

const FADE_S = 0.18;

function envelope(timeS: number, fromS: number, toS: number): number {
  if (timeS <= fromS || timeS >= toS) return 0;
  const fadeIn = interpolate(timeS, [fromS, fromS + FADE_S], [0, 1], {
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });
  const fadeOut = interpolate(timeS, [toS - FADE_S, toS], [1, 0], {
    extrapolateLeft: "clamp",
    easing: Easing.in(Easing.cubic),
  });
  return Math.min(fadeIn, fadeOut);
}

export const Annotations: React.FC<{
  annotations: Annotation[];
  viewport: { width: number; height: number };
  timeS: number;
  cameraScale: number;
}> = ({ annotations, viewport, timeS, cameraScale }) => {
  const k = useDesignScale();
  const counter = 1 / Math.max(1, cameraScale);
  const pct = (v: number, of: number) => `${(v / of) * 100}%`;

  return (
    <div style={{ position: "absolute", inset: 0, pointerEvents: "none" }}>
      {annotations.map((a, i) => {
        const o = envelope(timeS, a.fromS, a.toS);
        if (o <= 0.001) return null;
        const pad = a.padding;
        const box = {
          left: pct(a.rect.x - pad, viewport.width),
          top: pct(a.rect.y - pad, viewport.height),
          width: pct(a.rect.w + pad * 2, viewport.width),
          height: pct(a.rect.h + pad * 2, viewport.height),
        };
        const grow = interpolate(o, [0, 1], [0.96, 1]);

        if (a.kind === "spotlight") {
          return (
            <div
              key={i}
              style={{
                position: "absolute",
                ...box,
                borderRadius: 12 * k,
                boxShadow: `0 0 0 9999px ${a.color}`,
                opacity: o,
              }}
            />
          );
        }

        if (a.kind === "highlight") {
          return (
            <div
              key={i}
              style={{
                position: "absolute",
                ...box,
                borderRadius: 10 * k,
                border: `${3 * k * counter}px solid ${a.color}`,
                background: `color-mix(in srgb, ${a.color} 12%, transparent)`,
                opacity: o,
                transform: `scale(${grow})`,
              }}
            />
          );
        }

        if (a.kind === "tooltip") {
          return (
            <div
              key={i}
              style={{
                position: "absolute",
                left: pct(a.rect.x + a.rect.w / 2, viewport.width),
                top: pct(a.rect.y - pad, viewport.height),
                transform: `translate(-50%, -100%) translateY(${(1 - o) * 8 * k}px) scale(${counter * 1.2})`,
                transformOrigin: "50% 100%",
                opacity: o,
              }}
            >
              <div
                style={{
                  background: a.color,
                  color: "#fafafa",
                  fontFamily: "Inter, system-ui, sans-serif",
                  fontWeight: 600,
                  fontSize: 18 * k,
                  padding: `${8 * k}px ${14 * k}px`,
                  borderRadius: 10 * k,
                  whiteSpace: "nowrap",
                  boxShadow: `0 ${6 * k}px ${18 * k}px rgba(0,0,0,0.25)`,
                }}
              >
                {a.text}
              </div>
              <div
                style={{
                  width: 0,
                  height: 0,
                  margin: "0 auto",
                  borderLeft: `${8 * k}px solid transparent`,
                  borderRight: `${8 * k}px solid transparent`,
                  borderTop: `${8 * k}px solid ${a.color}`,
                }}
              />
            </div>
          );
        }

        const draw = interpolate(o, [0, 1], [0, 1]);
        const len = 160;
        return (
          <div
            key={i}
            style={{
              position: "absolute",
              left: pct(a.rect.x + a.rect.w, viewport.width),
              top: pct(a.rect.y, viewport.height),
              transform: `scale(${counter * 1.2})`,
              transformOrigin: "0 100%",
              opacity: o,
            }}
          >
            <svg
              width={140 * k}
              height={110 * k}
              viewBox="0 0 140 110"
              style={{ position: "absolute", left: 0, bottom: 0, overflow: "visible" }}
            >
              <path
                d="M130 10 C 90 20, 40 40, 12 96"
                fill="none"
                stroke={a.color}
                strokeWidth={5}
                strokeLinecap="round"
                strokeDasharray={len}
                strokeDashoffset={len * (1 - draw)}
              />
              <path
                d="M4 82 L12 98 L28 88"
                fill="none"
                stroke={a.color}
                strokeWidth={5}
                strokeLinecap="round"
                strokeLinejoin="round"
                opacity={draw > 0.9 ? 1 : 0}
              />
            </svg>
            <div
              style={{
                position: "absolute",
                left: 120 * k,
                bottom: 96 * k,
                color: a.color,
                fontFamily: "Inter, system-ui, sans-serif",
                fontWeight: 800,
                fontSize: 30 * k,
                whiteSpace: "nowrap",
                transform: `rotate(-4deg)`,
              }}
            >
              {a.text}
            </div>
          </div>
        );
      })}
    </div>
  );
};
