import { useMemo } from "react";
import { spring } from "remotion";
import { MAIN_FONT } from "@/shared/fonts";
import { useAuthoredConfig, useAuthoredFrame } from "@/video/core/authored-clock";

const COLUMN_EM = 0.7;
const LETTER_SPACING_EM = -0.03;

function columnWidths(
  from: string,
  to: string,
  fontSize: number,
  fontWeight: number,
  fontFamily: string,
): number[] {
  const fixed = () => Array.from(from, () => COLUMN_EM * fontSize);
  if (typeof document === "undefined") return fixed();
  const ctx = document.createElement("canvas").getContext("2d");
  if (!ctx) return fixed();
  ctx.font = `${fontWeight} ${fontSize}px ${fontFamily}`;
  ctx.letterSpacing = `${LETTER_SPACING_EM * fontSize}px`;
  return Array.from(from, (char, i) =>
    Math.max(ctx.measureText(char).width, ctx.measureText(to[i] ?? "").width),
  );
}

export interface SlotMachineRollProps {
  fontFamily?: string;
  from?: string;
  to?: string;
  fontSize?: number;
  color?: string;
  fontWeight?: number;
  speed?: number;
  className?: string;
}

export function SlotMachineRoll({
  fontFamily = MAIN_FONT,
  from = "$99",
  to = "$199",
  fontSize = 120,
  color = "#171717",
  fontWeight = 700,
  speed = 1,
  className,
}: SlotMachineRollProps) {
  const frame = useAuthoredFrame() * speed;
  const { fps } = useAuthoredConfig();

  const len = Math.max(from.length, to.length);
  const paddedFrom = from.padStart(len, " ");
  const paddedTo = to.padStart(len, " ");
  const widths = useMemo(
    () => columnWidths(paddedFrom, paddedTo, fontSize, fontWeight, fontFamily),
    [paddedFrom, paddedTo, fontSize, fontWeight, fontFamily],
  );

  const columns = [];
  for (let i = 0; i < len; i++) {
    const raw = spring({
      frame: frame - i * 4,
      fps,
      config: { damping: 14 },
    });
    const t = Math.max(0, Math.min(1, raw));
    columns.push(
      <span
        key={i}
        style={{
          display: "inline-block",
          overflow: "hidden",
          height: "1.1em",
          lineHeight: "1.1em",
          verticalAlign: "top",
          width: widths[i],
          textAlign: "center",
        }}
      >
        <span
          style={{
            display: "flex",
            flexDirection: "column",
            translate: `0 ${-t * fontSize * 1.1}px`,
          }}
        >
          <span style={{ height: "1.1em", whiteSpace: "pre" }}>{paddedFrom[i]}</span>
          <span style={{ height: "1.1em", whiteSpace: "pre" }}>{paddedTo[i]}</span>
        </span>
      </span>,
    );
  }

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: "white",
      }}
    >
      <span
        className={className}
        style={{
          fontSize,
          fontWeight,
          color,
          letterSpacing: `${LETTER_SPACING_EM}em`,
          fontFamily,
          lineHeight: 1.1,
        }}
      >
        {columns}
      </span>
    </div>
  );
}
