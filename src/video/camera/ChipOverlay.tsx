import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import { settle } from "./lib/push";
import { chipsAt, type ReelChip } from "./lib/chips";
import type { AnnotationStyle } from "./lib/style";

export type ChipOverlayProps = {
  chips: ReelChip[];
  style: AnnotationStyle;
  totalS: number;
};

export const ChipOverlay: React.FC<ChipOverlayProps> = ({ chips, style }) => {
  const frame = useCurrentFrame();
  const { fps, height } = useVideoConfig();
  const t = frame / fps;
  const live = chipsAt(chips, t);
  if (live.length === 0) return <AbsoluteFill />;

  const pillH = height * style.heightFrac;

  return (
    <AbsoluteFill>
      {live.map((c) => {
        const inP = 1 - settle((t - c.fromS) / Math.max(style.wipeS, 1e-6));
        const outP = 1 - settle((c.toS - t) / Math.max(style.exitS, 1e-6));
        const p = Math.min(inP, outP);
        const fromLeft = c.x < 0.5;
        const hidden = (1 - p) * 100;
        return (
          <div
            key={`${c.text}-${c.fromS}`}
            style={{
              position: "absolute",
              left: `${c.x * 100}%`,
              top: `${c.y * 100}%`,
              transform: `translate(-50%, -50%) scale(${1 + (style.oversize - 1) * (1 - p)})`,
              clipPath: fromLeft ? `inset(0 ${hidden}% 0 0)` : `inset(0 0 0 ${hidden}%)`,
              height: pillH,
              display: "flex",
              alignItems: "center",
              padding: `0 ${pillH * 0.38}px`,
              background: style.fill,
              color: style.ink,
              borderRadius: pillH * style.radiusFrac,
              fontSize: (pillH * style.capRatio) / 0.715,
              fontFamily: '"Inter", ui-sans-serif, system-ui, sans-serif',
              fontWeight: 500,
              letterSpacing: "-0.01em",
              whiteSpace: "nowrap",
            }}
          >
            {c.text}
          </div>
        );
      })}
    </AbsoluteFill>
  );
};
