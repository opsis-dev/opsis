import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { MAIN_FONT } from "../../shared/fonts";

const SWEEP_FRAMES = 90;
const SWEEP_FROM_PCT = -30;
const SWEEP_TO_PCT = 130;
const SWEEP_BAND_PCT = 20;

export const Placeholder: React.FC<{ title?: string; subtitle?: string }> = ({
  title = "Not captured yet",
  subtitle,
}) => {
  const frame = useCurrentFrame();
  const sweep = interpolate(
    frame % SWEEP_FRAMES,
    [0, SWEEP_FRAMES],
    [SWEEP_FROM_PCT, SWEEP_TO_PCT],
  );
  return (
    <AbsoluteFill
      style={{
        background: "#0c0c10",
        alignItems: "center",
        justifyContent: "center",
        fontFamily: MAIN_FONT,
      }}
    >
      <div
        style={{
          width: "72%",
          height: "64%",
          borderRadius: 28,
          border: "3px dashed rgba(255,255,255,0.18)",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          gap: 18,
          background: `linear-gradient(100deg, transparent ${sweep - SWEEP_BAND_PCT}%, rgba(255,255,255,0.05) ${sweep}%, transparent ${sweep + SWEEP_BAND_PCT}%)`,
        }}
      >
        <div style={{ color: "#e4e4e7", fontSize: 56, fontWeight: 650 }}>{title}</div>
        {subtitle ? (
          <div style={{ color: "#a1a1aa", fontSize: 30, maxWidth: "80%", textAlign: "center" }}>
            {subtitle}
          </div>
        ) : null}
      </div>
    </AbsoluteFill>
  );
};
