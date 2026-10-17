import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { MAIN_FONT } from "../../shared/fonts";

export const Placeholder: React.FC<{ title?: string; subtitle?: string }> = ({
  title = "Not captured yet",
  subtitle,
}) => {
  const frame = useCurrentFrame();
  const sweep = interpolate(frame % 90, [0, 90], [-30, 130]);
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
          background: `linear-gradient(100deg, transparent ${sweep - 20}%, rgba(255,255,255,0.05) ${sweep}%, transparent ${sweep + 20}%)`,
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
