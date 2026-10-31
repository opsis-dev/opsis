import { interpolate } from "@opsis/runtime/interpolate.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { useMemo } from "react";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings, WipeFrom } from "#video/generated/spec.ts";
import type { TransitionPresentation } from "#video/lib/transitions/presentation.ts";
import { transitionPhase } from "#video/lib/transitions/transition-timing.ts";

type FrostedGlassWipeProps = ElementSettings["frosted-glass-wipe"];

const AXIS: Record<
  WipeFrom,
  { vertical: boolean; gradient: string; near: "left" | "right" | "top" | "bottom" }
> = {
  "from-left": { vertical: false, gradient: "to right", near: "left" },
  "from-right": { vertical: false, gradient: "to left", near: "right" },
  "from-top": { vertical: true, gradient: "to bottom", near: "top" },
  "from-bottom": { vertical: true, gradient: "to top", near: "bottom" },
};

function FrostedGlassWipePresentation({
  children,
  presentationProgress,
  presentationDirection,
  passedProps: { blur, panelWidth, direction, frostColor, edgeColor, glow: glowSize },
}: React.ComponentProps<NonNullable<TransitionPresentation<FrostedGlassWipeProps>["component"]>>) {
  const phase = transitionPhase(presentationProgress, presentationDirection, {
    lead: 1,
  });
  const { vertical, gradient, near } = AXIS[direction];
  const swept = 1 - phase.displace;

  const revealStyle = useMemo(() => {
    if (!phase.isEntering) {
      return undefined;
    }

    const hidden = `${(1 - swept) * 100}%`;

    return {
      clipPath: vertical
        ? direction === "from-top"
          ? `inset(0 0 ${hidden} 0)`
          : `inset(${hidden} 0 0 0)`
        : direction === "from-left"
          ? `inset(0 ${hidden} 0 0)`
          : `inset(0 0 0 ${hidden})`,
    };
  }, [direction, phase.isEntering, swept, vertical]);

  const panelStyle = useMemo(() => {
    const widthPct = panelWidth * 100;
    const offset = swept * (100 + widthPct) - widthPct;
    const glow = interpolate(swept, [0, 0.15, 0.85, 1], [0, 1, 1, 0], CLAMPED);
    const edge = `1px solid color-mix(in srgb, ${edgeColor} ${(glow * 100).toFixed(1)}%, transparent)`;

    return {
      position: "absolute" as const,
      ...(vertical
        ? {
            left: 0,
            right: 0,
            top: `${offset}%`,
            height: `${widthPct}%`,
            [near === "top" ? "borderBottom" : "borderTop"]: edge,
          }
        : {
            top: 0,
            bottom: 0,
            left: `${offset}%`,
            width: `${widthPct}%`,
            [near === "left" ? "borderRight" : "borderLeft"]: edge,
          }),
      backdropFilter: `blur(${blur}px)`,
      WebkitBackdropFilter: `blur(${blur}px)`,
      background: `linear-gradient(${gradient}, transparent, ${frostColor}, transparent)`,
      boxShadow: `0 0 ${(glowSize * glow).toFixed(1)}px rgba(255,255,255,${(0.18 * glow).toFixed(3)})`,
      opacity: glow,
      pointerEvents: "none" as const,
    };
  }, [blur, edgeColor, frostColor, glowSize, gradient, near, panelWidth, swept, vertical]);

  if (!phase.isEntering) {
    return <AbsoluteFill>{children}</AbsoluteFill>;
  }

  return (
    <AbsoluteFill>
      <AbsoluteFill style={revealStyle}>{children}</AbsoluteFill>
      <div style={panelStyle} />
    </AbsoluteFill>
  );
}

export function frostedGlassWipe(
  props: FrostedGlassWipeProps,
): TransitionPresentation<FrostedGlassWipeProps> {
  return {
    component: FrostedGlassWipePresentation,
    props,
  };
}
