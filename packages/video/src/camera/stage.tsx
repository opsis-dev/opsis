import React from "react";
import { AbsoluteFill } from "remotion";
import { useDesignScale } from "#video/camera/lib/design-scale.ts";
import { ink, WINDOW_RADIUS } from "#video/camera/lib/style.ts";

export const Stage: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <AbsoluteFill
    style={{
      justifyContent: "center",
      alignItems: "center",
      overflow: "hidden",
    }}
  >
    {children}
  </AbsoluteFill>
);

const GLOW_RADIUS = 60;
const GLOW_BLUR = 90;
const GLOW_SCALE = "0.97, 0.96";
const RIM_RADIUS = WINDOW_RADIUS + 1;

const RIM_LOOKS = {
  light: { glow: "rgba(30,20,10,0.30)", lift: "2.5%", rim: ink(0.16) },
  dark: { glow: "rgba(255,255,255,0.10)", lift: "2%", rim: "rgba(255,255,255,0.14)" },
} as const;

export const RimLight: React.FC<{ light?: boolean }> = ({ light = false }) => {
  const k = useDesignScale();
  const look = RIM_LOOKS[light ? "light" : "dark"];
  return (
    <>
      <div
        style={{
          position: "absolute",
          inset: 0,
          borderRadius: GLOW_RADIUS * k,
          background: look.glow,
          transform: `translateY(${look.lift}) scale(${GLOW_SCALE})`,
          filter: `blur(${GLOW_BLUR * k}px)`,
          pointerEvents: "none",
        }}
      />
      <div
        style={{
          position: "absolute",
          inset: -k,
          borderRadius: RIM_RADIUS * k,
          boxShadow: `0 0 0 ${k}px ${look.rim}`,
          pointerEvents: "none",
        }}
      />
    </>
  );
};
