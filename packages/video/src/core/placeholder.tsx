import {
  PLACEHOLDER_COLOR,
  PLACEHOLDER_SUBTITLE_COLOR,
  PLACEHOLDER_TITLE_COLOR,
} from "@opsis/core/elements/colors";
import { MAIN_FONT } from "@opsis/core/fonts";
import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";

const SWEEP_FRAMES = 90;
const SWEEP_FROM_PCT = -30;
const SWEEP_TO_PCT = 130;
const SWEEP_BAND_PCT = 20;
const SWEEP_SHINE = "rgba(255,255,255,0.05)";
const BORDER = "3px dashed rgba(255,255,255,0.18)";
const CARD_RADIUS_PX = 28;
const CARD_GAP_PX = 18;
const TITLE_SIZE_PX = 56;
const TITLE_WEIGHT = 650;
const SUBTITLE_SIZE_PX = 30;

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
        background: PLACEHOLDER_COLOR,
        alignItems: "center",
        justifyContent: "center",
        fontFamily: MAIN_FONT,
      }}
    >
      <div
        style={{
          width: "72%",
          height: "64%",
          borderRadius: CARD_RADIUS_PX,
          border: BORDER,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          gap: CARD_GAP_PX,
          background: `linear-gradient(100deg, transparent ${sweep - SWEEP_BAND_PCT}%, ${SWEEP_SHINE} ${sweep}%, transparent ${sweep + SWEEP_BAND_PCT}%)`,
        }}
      >
        <div
          style={{
            color: PLACEHOLDER_TITLE_COLOR,
            fontSize: TITLE_SIZE_PX,
            fontWeight: TITLE_WEIGHT,
          }}
        >
          {title}
        </div>
        {subtitle ? (
          <div
            style={{
              color: PLACEHOLDER_SUBTITLE_COLOR,
              fontSize: SUBTITLE_SIZE_PX,
              maxWidth: "80%",
              textAlign: "center",
            }}
          >
            {subtitle}
          </div>
        ) : null}
      </div>
    </AbsoluteFill>
  );
};
